use chrono::prelude::{DateTime, Local};
use serde::Deserialize;
use url::Url;

/// The borough or New York state area where a vaccine appointment is being given.
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

    #[serde(rename(deserialize = "Staten Island"))]
    StatenIsland,

    Upstate,

    #[serde(rename(deserialize = "Long Island"))]
    LongIsland,

    #[serde(rename(deserialize = "Multiple locations"))]
    Multiple,

    #[serde(rename(deserialize = "Mid-Hudson"))]
    MidHudson,
}

/// Appointment summary information.
#[derive(serde::Deserialize)]
pub(crate) struct Appointments {
    /// Number of available appointments.
    pub(crate) count: usize,

    /// Appointment summary, including times.
    #[serde(deserialize_with = "deserialize_appointment_times")]
    pub(crate) summary: Vec<String>,
}

const APPOINTMENT_TIMES_SEPARATOR: char = ';';

fn deserialize_appointment_times<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    Ok(String::deserialize(deserializer)?
        .split(APPOINTMENT_TIMES_SEPARATOR)
        .map(ToOwned::to_owned)
        .collect())
}

/// Vaccine appointment portal information.
#[derive(serde::Deserialize, Clone)]
pub(crate) struct Portal {
    /// Full name of the portal.
    pub(crate) name: String,

    /// Optional short name for the portal.
    pub(crate) short_name: Option<String>,

    /// Key used in locations data to reference the portal.
    pub(crate) key: String,

    /// The URL people can use to get an appointment.
    pub(crate) url: Url,
}

/// Vaccine location information.
#[derive(serde::Deserialize)]
pub(crate) struct Location {
    /// The name of the vaccination site.
    #[serde(rename(deserialize = "name"))]
    pub(crate) site: String,

    /// Whether or not the site is currently active.
    #[serde(rename(deserialize = "active"))]
    pub(crate) currently_giving_vaccinations: bool,

    /// Whether or not any appointments are available at all.
    #[serde(rename(deserialize = "available"))]
    pub(crate) has_appointments: bool,

    /// The last time the site was updated, in local time.
    pub(crate) updated_at: DateTime<Local>,

    /// When information was last available.
    #[serde(rename(deserialize = "last_available_at"))]
    pub(crate) _last_available_at: DateTime<Local>,

    /// Human readable portal name.
    #[serde(rename(deserialize = "portal_name"))]
    pub(crate) _portal_name: String,

    /// Portal key.
    #[serde(rename(deserialize = "portal_key"))]
    pub(crate) portal_key: String,

    /// The borough/New York State area in which appointments are available.
    pub(crate) area: Area,

    /// Information about available appointments.
    pub(crate) appointments: Appointments,

    /// URL containing more information
    #[serde(rename(deserialize = "info_url"))]
    pub(crate) _info_url: Option<Url>,
}

/// Aggregate portal + location information.
#[derive(serde::Deserialize)]
pub(crate) struct Dashboard {
    /// Sequence of portals containing information about where (on the internet) to schedule an appointment.
    pub(crate) portals: Vec<Portal>,

    /// Sequence of locations.
    pub(crate) locations: Vec<Location>,
}
