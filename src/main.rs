use anyhow::Context;
use dashboard::Area;
use enumset::EnumSet;
use std::{
    iter::FromIterator,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use structopt::StructOpt;
use strum::VariantNames;
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
        default_value = "https://turbovax.global.ssl.fastly.net/dashboard"
    )]
    data_url: url::Url,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    ctrlc::set_handler(move || {
        running_clone.store(false, Ordering::SeqCst);
    })
    .context("setting ctrlc handler")?;

    let Opt {
        area,
        site_filter,
        twilio_config,
        duration_between_requests,
        log_filter,
        data_url,
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
        EnumSet::from_iter(area.into_iter())
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

    let mut interval = tokio::time::interval(duration_between_requests);

    while running.load(Ordering::SeqCst) {
        if let Err(error) = client.check_availability().await {
            error!(?error);
        }

        interval.tick().await;
    }

    Ok(())
}
