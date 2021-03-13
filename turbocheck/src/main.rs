use enumset::EnumSet;
use std::{iter::FromIterator, path::PathBuf, time::Duration};
use structopt::StructOpt;
use strum::VariantNames;
use tracing::{debug, error, info};
use tracing_subscriber::layer::SubscriberExt;

mod error;
mod turbovax;
mod vax_site;

use vax_site::Area;

#[derive(Debug, structopt::StructOpt)]
struct Opt {
    /// Boroughs/regions to look for appointments in. Passing no area argument searches all areas.
    #[structopt(long, possible_values = Area::VARIANTS)]
    area: Vec<Area>,

    /// Pattern of text to use for searching site names.
    #[structopt(long)]
    site_pattern: Option<regex::Regex>,

    /// Optional twilio configuration. If this argument isn't provided,
    /// then text messaging functionality will be disabled.
    #[structopt(long)]
    twilio_config: Option<PathBuf>,

    /// The time to wait between requests.
    #[structopt(
        long,
        default_value = "1s",
        parse(try_from_str = humantime::parse_duration)
    )]
    time_between_requests: Duration,

    /// Logging verbosity
    #[structopt(long, default_value = "INFO")]
    log_level: tracing_subscriber::filter::LevelFilter,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let Opt {
        twilio_config,
        area,
        time_between_requests,
        log_level,
        site_pattern,
    } = Opt::from_args();

    tracing::subscriber::set_global_default(
        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer().with_timer(
                tracing_subscriber::fmt::time::ChronoUtc::with_format("%Y-%m-%dT%H:%M:%S%.3f".into()),
            ))
            .with(log_level),
    )?;

    // If no area was specified search all areas.
    let areas = if area.is_empty() {
        EnumSet::all()
    } else {
        EnumSet::from_iter(area.into_iter())
    };

    info!(message = "searching", ?areas);

    let request_client = reqwest::ClientBuilder::new().build()?;
    let mut client = turbovax::TurboxVaxClient::builder()
        .client(request_client.clone())
        .areas(areas)
        .site_pattern(site_pattern)
        .twilio_client(if let Some(twilio_config) = twilio_config {
            let twilio_concurrent::TwilioConfig {
                account_sid,
                auth_token,
                sms_from,
                sms_to,
            } = toml::from_str(&tokio::fs::read_to_string(twilio_config).await?)?;
            Some(
                twilio_concurrent::Client::builder()
                    .client(request_client)
                    .account_sid(account_sid)
                    .auth_token(auth_token)
                    .sms_from(sms_from)
                    .sms_to(sms_to)
                    .build(),
            )
        } else {
            None
        })
        .build();

    loop {
        if let Err(error) = client.check_availability().await {
            error!(?error);
        }

        debug!(message = "sleep", ?time_between_requests);
        tokio::time::sleep(time_between_requests).await;
    }
}
