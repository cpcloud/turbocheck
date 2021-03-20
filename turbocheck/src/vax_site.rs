use chrono::prelude::{DateTime, Local};
use url::Url;

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

#[derive(serde::Deserialize)]
pub(crate) struct Appointments {
    /// Number of available appointments.
    pub(crate) count: usize,

    /// Appointment summary, including times.
    pub(crate) summary: String,
}

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
    #[serde(rename = "last_available_at")]
    pub(crate) _last_available_at: DateTime<Local>,

    /// Human readable portal name.
    #[serde(rename = "portal_name")]
    pub(crate) _portal_name: String,

    /// Portal key.
    pub(crate) portal: String,

    /// The borough/New York State area in which appointments are available.
    pub(crate) area: Area,

    /// Available appointments.
    pub(crate) appointments: Appointments,

    /// URL containing more information
    #[serde(rename = "info_url")]
    pub(crate) _info_url: Option<Url>,
}

#[derive(serde::Deserialize)]
pub(crate) struct Dashboard {
    /// Sequence of portals.
    pub(crate) portals: Vec<Portal>,

    /// Sequence of locations.
    pub(crate) locations: Vec<Location>,
}
