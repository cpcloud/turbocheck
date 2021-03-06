use anyhow::{anyhow, Context, Result};
use dashboard::{Area, PortalType};
use enumset::EnumSet;
use std::{path::PathBuf, time::Duration};
use structopt::StructOpt;
use strum::VariantNames;
use tokio::signal::unix;
use tracing::{error, info};
use tracing_subscriber::layer::SubscriberExt;

mod client;
mod dashboard;
mod error;
mod twilio_concurrent;

/// A command line client for https://turbovax.info data.
#[derive(Debug, structopt::StructOpt)]
struct Opt {
    /// Boroughs/regions to look for appointments in. Not specifying this argument searches all areas.
    #[structopt(short, long, possible_values = Area::VARIANTS)]
    area: Vec<Area>,

    /// Pattern of text to use for searching site names. Not specifying this argument results in all sites being displayed.
    #[structopt(short, long)]
    site_filter: Option<regex::Regex>,

    /// The type of location to filter for.
    #[structopt(short, long, possible_values = PortalType::VARIANTS)]
    location_type: Vec<PortalType>,

    /// Optional Twilio configuration. If this argument isn't provided, then text messaging functionality will be disabled.
    #[structopt(short, long)]
    twilio_config: Option<PathBuf>,

    /// The time to wait between requests to TurboVax.
    #[structopt(
        short,
        long,
        default_value = "1s",
        parse(try_from_str = humantime::parse_duration)
    )]
    duration_between_requests: Duration,

    /// Verbosity of logs.
    #[structopt(long, env = "RUST_LOG", default_value = "info")]
    log_filter: tracing_subscriber::filter::EnvFilter,

    /// The URL from which to fetch TurboVax data.
    #[structopt(
        short = "-u",
        long,
        default_value = crate::dashboard::DEFAULT_DATA_URL
    )]
    data_url: url::Url,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let Opt {
        area,
        site_filter,
        twilio_config,
        duration_between_requests,
        log_filter,
        data_url,
        location_type,
    } = Opt::from_args();

    tracing::subscriber::set_global_default(
        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer())
            .with(log_filter),
    )
    .context("setting tracing global default subscriber")?;

    // If no area was specified search all areas.
    let areas = if area.is_empty() {
        EnumSet::all()
    } else {
        area.into_iter().collect()
    };

    let location_types = if location_type.is_empty() {
        EnumSet::all()
    } else {
        location_type.into_iter().collect()
    };

    info!(
        message = "searching",
        ?areas,
        interval = %humantime::format_duration(duration_between_requests)
    );

    let request_client = reqwest::ClientBuilder::new()
        .build()
        .context("creating reqwest client")?;
    let mut client = client::Client::builder()
        .client(request_client.clone())
        .data_url(data_url)
        .areas(areas)
        .location_types(location_types)
        .site_filter(site_filter)
        .twilio_client(if let Some(twilio_config) = twilio_config {
            Some(
                twilio_concurrent::Client::builder()
                    .client(request_client)
                    .config(
                        toml::from_str(
                            &tokio::fs::read_to_string(twilio_config)
                                .await
                                .context("reading twilio config")?,
                        )
                        .context("parsing twilio config TOML")?,
                    )
                    .build(),
            )
        } else {
            None
        })
        .build();

    let mut sigterm = unix::signal(unix::SignalKind::terminate())?;
    let mut sigint = unix::signal(unix::SignalKind::interrupt())?;

    loop {
        let request = async {
            if let Err(error) = client.check_availability().await {
                error!(?error);
            }

            tokio::time::sleep(duration_between_requests).await;
        };

        tokio::select! {
            _ = sigint.recv() => return Err(anyhow!("got SIGINT")),
            _ = sigterm.recv() => return Err(anyhow!("got SIGTERM")),
            _ = request => continue,
        }
    }
}
