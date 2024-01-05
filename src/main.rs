use chrono::{DateTime, TimeZone, Utc};
use serde::de::{self, Deserializer};
use serde_derive::Deserialize;
use serde_xml_rs::from_reader;
use std::fs::File;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct TrainingCenterDatabase {
    #[serde(rename = "Activities")]
    activities: Activities,
}

#[derive(Debug, Deserialize)]
struct Activities {
    #[serde(rename = "Activity")]
    activity: Vec<Activity>,
}

#[derive(Debug, Deserialize)]
struct Activity {
    #[serde(rename = "Lap")]
    lap: Vec<Lap>,
}

#[derive(Debug, Deserialize)]
struct Lap {
    #[serde(rename = "Track")]
    track: Vec<Track>,
}

#[derive(Debug, Deserialize)]
struct Track {
    #[serde(rename = "Trackpoint")]
    trackpoints: Vec<Trackpoint>,
}

#[derive(Debug, Deserialize)]
struct Trackpoint {
    #[serde(rename = "Time", deserialize_with = "deserialize_datetime_opt")]
    time: Option<DateTime<Utc>>,

    #[serde(rename = "HeartRateBpm", deserialize_with = "deserialize_hr_opt")]
    heart_rate_bpm: Option<u16>,
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
    struct HeartRateBpm {
        #[serde(rename = "Value")]
        value: String,
    }

    let hr_bpm: Option<HeartRateBpm> = Option::deserialize(deserializer)?;
    hr_bpm
        .map(|hr| hr.value.parse::<u16>().map_err(de::Error::custom))
        .transpose()
}

fn calculate_weighted_average_hr(trackpoints: &Vec<Trackpoint>) -> Option<f64> {
    let mut total_time_weighted_hr = 0.0;
    let mut total_time = 0.0;

    for window in trackpoints.windows(2) {
        // dbg!(window);
        if let (Some(tp1), Some(tp2)) = (window[0].time, window[1].time) {
            if let (Some(hr1), Some(hr2)) = (window[0].heart_rate_bpm, window[1].heart_rate_bpm) {
                let duration = (tp2 - tp1).num_milliseconds() as f64;
                let avg_hr = (hr1 + hr2) as f64 / 2.0; // Average HR between two points
                                                       // dbg!(duration);
                                                       // dbg!(avg_hr);
                total_time_weighted_hr += avg_hr * duration;
                total_time += duration;
            }
        }
    }
    // dbg!(total_time);
    if total_time > 0.0 {
        Some(total_time_weighted_hr / total_time)
    } else {
        None
    }
}

fn main() {
    let file = File::open("ex-small.tcx").expect("Unable to open file");
    let tcx: TrainingCenterDatabase = from_reader(file).expect("Unable to parse XML");
    for act in tcx.activities.activity {
        for lap in act.lap {
            for t in lap.track {
                // dbg!(&t.trackpoints);
                let q = calculate_weighted_average_hr(&t.trackpoints);
                // dbg!(q);
                println!("Avg : {:?}", q)
            }
        }
    }
    // println!("{:#?}", tcx);
}
