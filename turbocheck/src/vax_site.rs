use anyhow::Result;
use chrono::prelude::{DateTime, Local};
use serde::Deserialize;

#[derive(
    Debug,
    serde::Deserialize,
    strum_macros::EnumString,
    strum_macros::EnumVariantNames,
    enumset::EnumSetType,
)]
#[strum(serialize_all = "kebab_case")]
pub(crate) enum Area {
    Manhattan,
    Queens,
    Brooklyn,
    Bronx,

    #[serde(rename = "Staten Island")]
    StatenIsland,

    Upstate,

    #[serde(rename = "Long Island")]
    LongIsland,
}

#[derive(serde::Deserialize)]
pub(crate) struct Site {
    /// The name of the vaccination site.
    #[serde(rename = "site_name")]
    pub(crate) site: String,

    /// Whether or not the site is currently active.
    pub(crate) is_active: bool,

    /// The URL people can use to get an appointment.
    pub(crate) url: url::Url,

    /// The last time the site was updated, in local time.
    pub(crate) updated_at: DateTime<Local>,

    /// The number of available appointments.
    pub(crate) appointment_count: usize,

    /// Available appointment times.
    #[serde(deserialize_with = "appointment_times_deserialize")]
    pub(crate) appointment_times: Vec<String>,

    /// Whether or not any appointments are available at all.
    pub(crate) is_available: bool,

    /// The borough/New York State area in which appointments are available.
    pub(crate) area: Area,

    /// Not entirely clear what this field is used for.
    #[serde(rename = "last_available_at")]
    pub(crate) _last_available_at: DateTime<Local>,
}

// XXX: Hopefully this doesn't ever change to a different character!
const APPOINTMENT_TIMES_SPLIT_CHAR: char = ';';

fn appointment_times_deserialize<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    Ok(String::deserialize(deserializer)?
        .split(APPOINTMENT_TIMES_SPLIT_CHAR)
        .map(ToOwned::to_owned)
        .collect())
}

#[derive(serde::Deserialize)]
pub(crate) struct Data {
    pub(crate) feed: Feed,
}

#[derive(serde::Deserialize)]
pub(crate) struct Feed {
    #[serde(rename = "entry")]
    pub(crate) entries: Vec<Entry>,
}

#[derive(serde::Deserialize)]
pub(crate) struct Entry {
    pub(crate) content: Content,
}

#[derive(serde::Deserialize)]
pub(crate) struct Content {
    #[serde(rename = "$t", deserialize_with = "site_deserialize")]
    pub(crate) site: Site,
}

// TODO: `serde_with` in theory could work, but it appears to not work unless you impl both Deserialize _and_ Serialize.
// Way overkill to impl an ultimately unused Serialize just to avoid writing two functions.
fn site_deserialize<'de, D>(deserializer: D) -> Result<Site, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    serde_json::from_str(&String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
}
