use chrono::{DateTime, TimeZone, Utc};
use serde::{de, Deserialize, Deserializer};
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TrainingCenterDatabase {
    #[serde(rename = "Activities")]
    pub activities: Activities,
}

#[derive(Debug, Deserialize)]
pub struct Activities {
    #[serde(rename = "Activity")]
    pub activity: Vec<Activity>,
}

#[derive(Debug, Deserialize)]
pub struct Activity {
    #[serde(rename = "Lap")]
    pub lap: Vec<Lap>,
}

#[derive(Debug, Deserialize)]
pub struct Lap {
    #[serde(rename = "Track")]
    pub track: Vec<Track>,
}

#[derive(Debug, Deserialize)]
pub struct Track {
    #[serde(rename = "Trackpoint")]
    pub trackpoints: Vec<Trackpoint>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Trackpoint {
    #[serde(rename = "Time", deserialize_with = "deserialize_datetime_opt")]
    pub time: Option<DateTime<Utc>>,

    #[serde(rename = "HeartRateBpm", deserialize_with = "deserialize_hr_opt")]
    pub heart_rate_bpm: Option<u16>,

    // #[serde(
    //     default,
    //     rename = "Extensions",
    //     deserialize_with = "deserialize_watt_opt"
    // )]
    // watts: Option<u16>,
    #[serde(rename = "Extensions")]
    // #[serde(default)] // This tells Serde that the field is optional
    pub extensions: Option<Extensions>,
}

fn deserialize_datetime_opt<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    if let Some(str) = s {
        Utc.datetime_from_str(&str, "%Y-%m-%dT%H:%M:%S%.fZ")
            .map(Some)
            .map_err(de::Error::custom)
    } else {
        Ok(None)
    }
}

fn deserialize_hr_opt<'de, D>(deserializer: D) -> Result<Option<u16>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    pub struct HeartRateBpm {
        #[serde(rename = "Value")]
        value: String,
    }

    let hr_bpm: Option<HeartRateBpm> = Option::deserialize(deserializer)?;
    hr_bpm
        .map(|hr| hr.value.parse::<u16>().map_err(de::Error::custom))
        .transpose()
}

#[derive(Debug, Deserialize, Clone)]
pub struct Extensions {
    #[serde(rename = "TPX")]
    pub tpx: TPX,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TPX {
    #[serde(rename = "Watts")]
    pub watts: u16,
}
