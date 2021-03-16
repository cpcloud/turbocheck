use crate::{
    error::Error,
    vax_site::{Area, Data, Site},
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
    site_pattern: Option<regex::Regex>,

    data_uris: Vec<String>,
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
    async fn unified_data(&self) -> Result<impl Iterator<Item = Site>, Error> {
        let areas = self.areas;
        let mut iters = vec![];
        for uri in self.data_uris.iter().cloned() {
            iters.push(
                self.client
                    .get(uri)
                    .send()
                    .await
                    .map_err(Error::GetData)?
                    .json::<Data>()
                    .await
                    .map_err(Error::ParseData)?
                    .feed
                    .entries
                    .into_iter()
                    .filter_map(move |entry| {
                        let site = entry.content.site;
                        if site.is_active && areas.contains(site.area) {
                            Some(site)
                        } else {
                            None
                        }
                    }),
            );
        }
        Ok(iters.into_iter().flatten())
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
        for Site {
            site,
            url,
            updated_at,
            appointment_count,
            is_available,
            appointment_times,
            area,
            ..
        } in self.unified_data().await?
        {
            let desired_site = self
                .site_pattern
                .as_ref()
                .map(|pattern| pattern.is_match(&site))
                .unwrap_or(true);
            // if the site has available appointments
            if is_available {
                let newly_available = !self.was_available.contains(&site);

                // compute whether the site's last updated time is more recent than the currently
                // stored updated time
                let updated_recently = updated_at
                    > *self
                        .last_updated_at
                        .entry(site.clone())
                        .or_insert(updated_at);

                // always set the latest known update time for the site
                self.last_updated_at.insert(site.clone(), updated_at);

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
                        format!("Site: {}", &site),
                        "".into(),
                        format!("Area: {:?}", area),
                        format!("Sched: {}", url),
                        format!("Map: {}", self.get_maps_short_url(&site)?),
                        "".into(),
                    ];

                    self.was_available.insert(site.clone());

                    let body_lines = lines
                        .into_iter()
                        .chain(
                            appointment_times
                                .into_iter()
                                .map(|s| format!("Times: {}", s)),
                        )
                        .chain(std::iter::once("".into()))
                        .chain(
                            vec![
                                format!("Appts Remaining: {}", appointment_count),
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
            } else if self.was_available.remove(&site) && desired_site {
                let message = format!(
                    "{updated_at} {area:?}: {site} appts no longer available",
                    updated_at = updated_at,
                    area = area,
                    site = site,
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
