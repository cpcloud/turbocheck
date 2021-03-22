use crate::{
    dashboard::{Area, Dashboard, Location, Portal},
    error::Error,
    twilio_concurrent,
};
use chrono::prelude::{DateTime, Local};
use enumset::EnumSet;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use std::collections::{HashMap, HashSet};
use tracing::{info, warn};
use urlshortener::client::UrlShortener;

#[derive(typed_builder::TypedBuilder)]
pub(crate) struct Client {
    client: reqwest::Client,
    data_url: url::Url,

    #[builder(default = EnumSet::all())]
    areas: EnumSet<Area>,

    #[builder(default = Default::default())]
    site_filter: Option<regex::Regex>,

    #[builder(default = Default::default())]
    twilio_client: Option<twilio_concurrent::Client>,

    #[builder(default = Default::default())]
    was_available: HashSet<String>,

    #[builder(default = Default::default())]
    last_updated_at: HashMap<String, DateTime<Local>>,

    #[builder(default = UrlShortener::new().expect("failed to construct UrlShortener"))]
    url_shortener: UrlShortener,
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

impl Client {
    fn get_short_url(&self, url: &str) -> Result<String, Error> {
        self.url_shortener
            .generate(url, &urlshortener::providers::Provider::IsGd)
            .map_err(Error::GetShortUrl)
    }

    fn get_maps_short_url(&self, site: &str) -> Result<String, Error> {
        self.get_short_url(&format!(
            "https://www.google.com/maps/search/?api=1&query={}",
            utf8_percent_encode(&site, FRAGMENT).to_string()
        ))
    }

    async fn data(&self) -> Result<impl Iterator<Item = (Location, Portal)>, Error> {
        let areas = self.areas;
        let Dashboard { portals, locations } = self
            .client
            .get(self.data_url.clone())
            .send()
            .await
            .map_err(Error::GetData)?
            .json()
            .await
            .map_err(Error::ParseData)?;
        let portals = portals
            .into_iter()
            .map(move |portal| (portal.key.clone(), portal))
            .collect::<HashMap<_, _>>();
        Ok(locations.into_iter().filter_map(move |location| {
            if location.currently_giving_vaccinations && areas.contains(location.area) {
                let portal_key = location.portal_key.clone();
                Some((location, portals[&portal_key].clone()))
            } else {
                None
            }
        }))
    }

    async fn check_location_availability(
        &mut self,
        Location {
            site,
            updated_at,
            appointments,
            has_appointments,
            area,
            ..
        }: Location,
        Portal { url, .. }: Portal,
    ) -> Result<(), Error> {
        let site_matches_filter_pattern = self
            .site_filter
            .as_ref()
            .map(|pattern| pattern.is_match(&site))
            .unwrap_or(true);

        // if the site has available appointments
        if has_appointments {
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
                    "".into(), // these empty strings are adding one more newline in between sections
                    format!("Site: {}", &site),
                    "".into(),
                    format!("Area: {:?}", area),
                    format!("Sched: {}", self.get_short_url(&url.to_string())?),
                    format!("Map: {}", self.get_maps_short_url(&site)?),
                    "".into(),
                ]
                .into_iter()
                .chain(
                    appointments
                        .summary
                        .into_iter()
                        .map(|slot| format!("Times: {}", slot)),
                )
                .chain(vec![
                    "".into(),
                    format!("Appts Remaining: {}", appointments.count),
                    format!("Last Updated: {}", updated_at),
                ])
                .collect::<Vec<_>>();

                self.was_available.insert(site.clone());

                if site_matches_filter_pattern {
                    let (header, footer) = format_header_footer(&lines)?;
                    info!(message = header.as_str());
                    lines.iter().for_each(|line| info!(message = line.as_str()));
                    info!(message = footer.as_str());

                    if let Some(ref twilio_client) = self.twilio_client {
                        let text_message = lines.join("\n");
                        twilio_client
                            .send_to_many(&text_message)
                            .await
                            .map_err(Error::SendAvailableMessage)?;
                    }
                }
            }
        } else if self.was_available.remove(&site) && site_matches_filter_pattern {
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
        Ok(())
    }

    pub(crate) async fn check_availability(&mut self) -> Result<(), Error> {
        for (location, portal) in self.data().await? {
            self.check_location_availability(location, portal).await?;
        }
        Ok(())
    }
}
