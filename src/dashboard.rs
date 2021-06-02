use chrono::prelude::{DateTime, Local};
use serde::Deserialize;
use url::Url;

pub(crate) const DEFAULT_DATA_URL: &str = "https://turbovax.global.ssl.fastly.net/dashboard";

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

    Unknown,
}

/// Appointment summary information.
#[derive(Debug, serde::Deserialize)]
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
    Ok(
        Option::deserialize(deserializer)?.map_or_else(Default::default, |summary: &str| {
            summary
                .split(APPOINTMENT_TIMES_SEPARATOR)
                .map(ToOwned::to_owned)
                .collect()
        }),
    )
}

#[derive(
    Debug,
    serde::Deserialize,
    strum_macros::EnumString,
    strum_macros::EnumVariantNames,
    enumset::EnumSetType,
)]
#[strum(serialize_all = "kebab_case")]
#[serde(rename_all(deserialize = "lowercase"))]
pub(crate) enum PortalType {
    Government,
    Clinic,
    Pharmacy,
}

/// Vaccine appointment portal information.
#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct Portal {
    /// Full name of the portal.
    pub(crate) name: String,

    /// Optional short name for the portal.
    pub(crate) short_name: Option<String>,

    /// Key used in locations data to reference the portal.
    pub(crate) key: String,

    /// The URL people can use to get an appointment.
    pub(crate) url: Url,

    /// Whether to show the name in the displayed card.
    #[serde(rename(deserialize = "show_name_in_card"))]
    pub(crate) _show_name_in_card: bool,

    #[serde(rename(deserialize = "type"))]
    pub(crate) r#type: PortalType,
}

/// Vaccine location information.
#[derive(Debug, serde::Deserialize)]
pub(crate) struct Location {
    /// The unique id of the vaccination site.
    pub(crate) id: String,

    /// The name of the vaccination site.
    pub(crate) name: String,

    /// Whether or not the site is currently active.
    pub(crate) active: bool,

    /// Whether or not any appointments are available at all.
    pub(crate) available: Option<bool>,

    /// The last time the site was updated, in local time.
    pub(crate) updated_at: Option<DateTime<Local>>,

    /// When information was last available.
    #[serde(rename(deserialize = "last_available_at"))]
    pub(crate) _last_available_at: Option<DateTime<Local>>,

    /// Portal key.
    #[serde(rename(deserialize = "portal"))]
    pub(crate) portal: String,

    /// The borough/New York State area in which appointments are available.
    #[serde(deserialize_with = "deserialize_area")]
    pub(crate) area: Area,

    /// The address of the site.
    pub(crate) formatted_address: Option<String>,

    /// Information about available appointments.
    pub(crate) appointments: Appointments,
}

fn deserialize_area<'de, D>(deserializer: D) -> Result<Area, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    Ok(Option::deserialize(deserializer)?.unwrap_or(Area::Unknown))
}

/// Aggregate portal + location information.
#[derive(Debug, serde::Deserialize)]
pub(crate) struct Dashboard {
    /// Sequence of portals containing information about where (on the internet) to schedule an appointment.
    pub(crate) portals: Vec<Portal>,

    /// Sequence of locations.
    pub(crate) locations: Vec<Location>,

    #[serde(rename(deserialize = "last_updated_at"))]
    pub(crate) _last_updated_at: DateTime<Local>,
}
