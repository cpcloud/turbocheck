use crate::{
    error::Error,
    vax_site::{Appointments, Area, Dashboard, Location, Portal},
};
use chrono::prelude::{DateTime, Local};
use enumset::EnumSet;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use std::collections::{HashMap, HashSet};
use tracing::{info, instrument, warn};
use urlshortener::client::UrlShortener;

#[derive(typed_builder::TypedBuilder)]
pub(crate) struct TurboxVaxClient {
    client: reqwest::Client,

    #[builder(default = Default::default())]
    twilio_client: Option<twilio_concurrent::Client>,

    #[builder(default = Default::default())]
    was_available: HashSet<String>,

    #[builder(default = Default::default())]
    last_updated_at: HashMap<String, DateTime<Local>>,

    #[builder(default = EnumSet::all())]
    areas: EnumSet<Area>,

    #[builder(default = UrlShortener::new().unwrap())]
    url_shortener: UrlShortener,

    #[builder(default = Default::default())]
    site_filter: Option<regex::Regex>,

    data_uri: String,
}

/// https://url.spec.whatwg.org/#fragment-percent-encode-set
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

const HEADER_TITLE: &str = " BEGIN ";
const HEADER_TITLE_LENGTH: usize = HEADER_TITLE.len();

const FOOTER_TITLE: &str = " END ";
const FOOTER_TITLE_LENGTH: usize = FOOTER_TITLE.len();

fn format_header_footer(lines: &[impl AsRef<str>]) -> Result<(String, String), Error> {
    let max_line_len = lines
        .iter()
        .map(|s| s.as_ref().len())
        .max()
        .ok_or(Error::GetMaxMessageLineLength)?;

    let header_delta = max_line_len - HEADER_TITLE_LENGTH;
    let left_header = "-".repeat(header_delta / 2);
    let right_header = "-".repeat(header_delta - left_header.len());
    let header = format!("{}{}{}", left_header, HEADER_TITLE, right_header);

    let footer_delta = max_line_len - FOOTER_TITLE_LENGTH;
    let left_footer = "-".repeat(footer_delta / 2);
    let right_footer = "-".repeat(footer_delta - left_footer.len());
    let footer = format!("{}{}{}", left_footer, FOOTER_TITLE, right_footer);
    Ok((header, footer))
}

impl TurboxVaxClient {
    async fn data(&self) -> Result<impl Iterator<Item = (Location, Portal)>, Error> {
        let areas = self.areas;
        let data = self
            .client
            .get(&self.data_uri)
            .send()
            .await
            .map_err(Error::GetData)?
            .json::<Dashboard>()
            .await
            .map_err(Error::ParseData)?;
        let portals = data
            .portals
            .into_iter()
            .map(move |portal| (portal.key.clone(), portal))
            .collect::<HashMap<_, _>>();
        let locations = data.locations;
        Ok(locations.into_iter().filter_map(move |location| {
            if location.active && areas.contains(location.area) {
                let portal = location.portal.clone();
                Some((location, portals[&portal].clone()))
            } else {
                None
            }
        }))
    }

    fn get_maps_short_url(&self, site: &str) -> Result<String, Error> {
        self.url_shortener
            .generate(
                &http::uri::Uri::builder()
                    .scheme("https")
                    .authority("www.google.com")
                    .path_and_query(format!(
                        "/maps/search/?api=1&query={}",
                        utf8_percent_encode(&site, FRAGMENT).to_string()
                    ))
                    .build()
                    .map_err(Error::BuildMapsUri)?
                    .to_string(),
                &urlshortener::providers::Provider::IsGd,
            )
            .map_err(Error::GetShortUrl)
    }

    #[instrument(
        name = "TurboxVaxClient::check_availability",
        skip(self),
        level = "debug"
    )]
    pub(crate) async fn check_availability(&mut self) -> Result<(), Error> {
        for (
            Location {
                name,
                updated_at,
                appointments: Appointments { count, summary },
                available,
                area,
                ..
            },
            Portal { url, .. },
        ) in self.data().await?
        {
            let desired_site = self
                .site_filter
                .as_ref()
                .map(|pattern| pattern.is_match(&name))
                .unwrap_or(true);
            // if the site has available appointments
            if available {
                let newly_available = !self.was_available.contains(&name);

                // compute whether the site's last updated time is more recent than the currently
                // stored updated time
                let updated_recently = updated_at
                    > *self
                        .last_updated_at
                        .entry(name.clone())
                        .or_insert(updated_at);

                // always set the latest known update time for the site
                self.last_updated_at.insert(name.clone(), updated_at);

                // if the site is newly available *or* if the appointment times for the site
                // have been updated recently
                if newly_available || updated_recently {
                    let lines = vec![
                        format!(
                            "{updated_at} {area:?}: appointments available!",
                            updated_at = updated_at,
                            area = area,
                        ),
                        "".into(),
                        format!("Site: {}", &name),
                        "".into(),
                        format!("Area: {:?}", area),
                        format!("Sched: {}", url),
                        format!("Map: {}", self.get_maps_short_url(&name)?),
                        "".into(),
                    ];

                    self.was_available.insert(name.clone());

                    let body_lines = lines
                        .into_iter()
                        .chain(std::iter::once(format!("Times: {}", summary)))
                        .chain(std::iter::once("".into()))
                        .chain(
                            vec![
                                format!("Appts Remaining: {}", count),
                                format!("Last Updated: {}", updated_at),
                            ]
                            .into_iter(),
                        )
                        .collect::<Vec<_>>();

                    if desired_site {
                        let (header, footer) = format_header_footer(&body_lines)?;
                        info!(message = header.as_str());
                        for line in &body_lines {
                            info!(message = line.as_str());
                        }
                        info!(message = footer.as_str());

                        if let Some(ref twilio_client) = self.twilio_client {
                            twilio_client
                                .send_to_many(&body_lines.join("\n"))
                                .await
                                .map_err(Error::SendAvailableMessage)?;
                        }
                    }
                }
            } else if self.was_available.remove(&name) && desired_site {
                let message = format!(
                    "{updated_at} {area:?}: {site} appts no longer available",
                    updated_at = updated_at,
                    area = area,
                    site = name,
                );
                warn!(message = message.as_str());

                if let Some(ref twilio_client) = self.twilio_client {
                    twilio_client
                        .send_to_many(&message)
                        .await
                        .map_err(Error::SendUnavailableMessage)?;
                }
            }
        }
        Ok(())
    }
}
