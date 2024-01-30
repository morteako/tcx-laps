use chrono::{DateTime, Duration, Utc};
use parse::{Lap, Trackpoint, TrainingCenterDatabase};
use serde_xml_rs::from_reader;
use std::{fs::File, ops::Add};

mod parse;

fn to_data_vec(d: TrainingCenterDatabase) -> Vec<DataLap> {
    let laps: Vec<Lap> = d
        .activities
        .activity
        .into_iter()
        .flat_map(|a| a.lap)
        .collect();
    let q: Vec<Vec<parse::Trackpoint>> = laps
        .into_iter()
        .map(|l| {
            l.track
                .into_iter()
                .flat_map(|t| t.trackpoints)
                .collect::<Vec<Trackpoint>>()
        })
        .collect();
    let qq: Vec<DataLap> = q
        .into_iter()
        .map(|v| DataLap {
            hr_data: v.clone().into_iter().filter_map(to_hr_data).collect(),
            watt_data: v.into_iter().filter_map(to_watt_data).collect(),
        })
        .collect();

    qq
}

struct DataLap {
    hr_data: Vec<Data>,
    watt_data: Vec<Data>,
}

struct Data {
    ts: DateTime<Utc>,
    val: u16,
}

fn to_hr_data(t: Trackpoint) -> Option<Data> {
    t.time
        .zip(t.heart_rate_bpm)
        .map(|w| Data { ts: w.0, val: w.1 })
}

fn to_watt_data(t: Trackpoint) -> Option<Data> {
    t.time.zip(t.extensions).map(|w| Data {
        ts: w.0,
        val: w.1.tpx.watts,
    })
}

// TODO max duration
// TODO excel format
// TODO CMD LINE ARGS?

#[derive(Debug)]
struct TimeInfo {
    total_time: Duration,
    avg_hr: f64,
    max_hr: u16,
    avg_watt: f64,
}

struct Avg {
    total_time: Duration,
    avg: f64,
}

fn get_avg(v: &Vec<Data>) -> Avg {
    let mut total_time_weighted_avg: f64 = 0.0;
    let mut total_time: f64 = 0.0;
    for window in v.windows(2) {
        let p1 = &window[0];
        let p2 = &window[1];
        let duration = (p2.ts - p1.ts).num_milliseconds() as f64;
        total_time += duration;
        let cur_avg = (p1.val + p2.val) as f64 / 2.0; // Average HR between two points
        total_time_weighted_avg += cur_avg * duration;
    }
    Avg {
        total_time: Duration::milliseconds(total_time as i64),
        avg: total_time_weighted_avg / total_time,
    }
}

fn calculate_weighted_average_hr(trackpoints: &DataLap) -> TimeInfo {
    let max_hr = trackpoints.hr_data.iter().map(|h| h.val).max().unwrap();
    let watt_avg = get_avg(&trackpoints.watt_data);
    let hr_avg = get_avg(&trackpoints.hr_data);

    TimeInfo {
        avg_hr: hr_avg.avg,
        total_time: hr_avg.total_time,
        max_hr: max_hr,
        avg_watt: watt_avg.avg,
    }
}

fn format_time_info(timeinfo: TimeInfo) -> String {
    let time = format!(
        "{:02}:{:02}",
        timeinfo.total_time.num_minutes(),
        (timeinfo.total_time.num_seconds()) % 60
    );

    let avg_hr = format!("{:.0}", timeinfo.avg_hr);
    let avg_watt = format!("{:.0}", timeinfo.avg_watt);
    let max_hr = format!("{:.0}", timeinfo.max_hr);

    format!("{}\t{}\t{}\t{}", time, avg_watt, avg_hr, max_hr)
}

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    file: String,

    /// Number of times to greet
    #[arg(short, long, value_parser, num_args = 1.., value_delimiter = ' ')]
    laps: Vec<usize>,
}

fn main() {
    let args = Args::parse();
    // dbg!(args.laps);
    let file = File::open(args.file).expect("Unable to open file");
    let tcx: TrainingCenterDatabase = from_reader(file).expect("Unable to parse XML");

    let all_data = to_data_vec(tcx);
    println!("Lap\tmm:ss\tavgW\tavgHR\tmaxHR");
    for (li, lap) in all_data
        .into_iter()
        .enumerate()
        .filter(|a| args.laps.contains(&a.0.add(1)))
    {
        // dbg!(&t.trackpoints);
        let q = calculate_weighted_average_hr(&lap);

        println!("{:>2}\t{}", li + 1, format_time_info(q));
    }
    // println!("{:#?}", tcx);
}
