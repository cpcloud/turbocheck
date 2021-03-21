use chrono::prelude::{DateTime, Local};
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
}

/// Appointment summary information.
#[derive(serde::Deserialize)]
pub(crate) struct Appointments {
    /// Number of available appointments.
    pub(crate) count: usize,

    /// Appointment summary, including times.
    pub(crate) summary: String,
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
    pub(crate) name: String,

    /// Whether or not the site is currently active.
    pub(crate) active: bool,

    /// Whether or not any appointments are available at all.
    pub(crate) available: bool,

    /// The last time the site was updated, in local time.
    pub(crate) updated_at: DateTime<Local>,

    /// Not entirely clear what this field is used for.
    #[serde(rename(deserialize = "last_available_at"))]
    pub(crate) _last_available_at: DateTime<Local>,

    /// Human readable portal name.
    #[serde(rename(deserialize = "portal_name"))]
    pub(crate) _portal_name: String,

    /// Portal key.
    pub(crate) portal: String,

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
