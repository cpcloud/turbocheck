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
use tracing::{debug, error, info};
use tracing_subscriber::layer::SubscriberExt;
use vax_site::Area;

mod error;
mod turbovax;
mod vax_site;

#[derive(Debug, structopt::StructOpt)]
struct Opt {
    /// Boroughs/regions to look for appointments in. Not specifying this argument searches all areas.
    #[structopt(short, long, possible_values = Area::VARIANTS)]
    area: Vec<Area>,

    /// Pattern of text to use for searching site names. Not specifying this argument results in all sites being displayed.
    #[structopt(short, long)]
    site_pattern: Option<regex::Regex>,

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
    #[structopt(long, default_value = "info")]
    log_level: tracing_subscriber::filter::EnvFilter,

    /// Log timestamp format
    #[structopt(long, default_value = "%Y-%m-%dT%H:%M:%S%.3f")]
    log_timestamp_format: String,

    #[structopt(short = "-u", long)]
    turbovax_data_uris: Vec<String>,
}

const DEFAULT_DATA_URIS: [&str; 2] = [
    "https://spreadsheets.google.com/feeds/cells/1NNZJWI7BYlajdBcqkEpOXXq6EZZyMd-zSIGKHNgS99w/4/public/full?alt=json",
    "https://spreadsheets.google.com/feeds/cells/1ORaOxzA1hKSd7w-iNOj6uS2MaG01l6ff1bHQ0dt2MIA/4/public/full?alt=json"
];

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let Opt {
        area,
        site_pattern,
        twilio_config,
        duration_between_requests,
        log_level,
        log_timestamp_format,
        mut turbovax_data_uris,
    } = Opt::from_args();

    tracing::subscriber::set_global_default(
        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer().with_timer(
                tracing_subscriber::fmt::time::ChronoUtc::with_format(log_timestamp_format),
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

    turbovax_data_uris.extend(DEFAULT_DATA_URIS.iter().map(|&data_uri| data_uri.into()));
    turbovax_data_uris.dedup();

    let request_client = reqwest::ClientBuilder::new().build()?;
    let mut client = turbovax::TurboxVaxClient::builder()
        .client(request_client.clone())
        .areas(areas)
        .site_pattern(site_pattern)
        .data_uris(turbovax_data_uris)
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

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    ctrlc::set_handler(move || {
        running_clone.store(false, Ordering::SeqCst);
    })?;

    while running.load(Ordering::SeqCst) {
        if let Err(error) = client.check_availability().await {
            error!(?error);
        }

        debug!(message = "sleep", ?duration_between_requests);
        tokio::time::sleep(duration_between_requests).await;
    }

    Ok(())
}
