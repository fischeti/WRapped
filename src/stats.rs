use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use serde::Serialize;
use serde_json;

use crate::wr::WRs;

#[derive(Debug, Serialize)]
pub struct Stats {
    // The year of the WRs
    pub year: u32,
    // The number of WRs
    pub num_wrs: usize,
    // The number of WRs that were replied to
    pub num_replied_wrs: usize,
    // The ratio of WRs that were replied to
    pub ratio_replied_wrs: f64,
    // The number of skipped WRs
    pub num_skipped_wrs: usize,
    // The average delay of the WRs
    pub avg_wr_delay: f64,
    // The average delay of the replied WRs
    pub avg_reply_delay: f64,
    // The histogram of the day of the week the WRs were sent
    pub weekday_wr_histogram: HashMap<u32, u32>,
    // The histogram of the day of the week the WRs were replied to
    pub weekday_reply_histogram: HashMap<u32, u32>,
    // The histogram of the hour of the day the WRs were sent
    pub hour_wr_histogram: HashMap<u32, u32>,
    // The histogram of the hour of the day the WRs were replied to
    pub hour_reply_histogram: HashMap<u32, u32>,
    // The histogram of the people that were in CC of the WRs
    pub cc_histogram: HashMap<String, u32>,
}

impl Stats {
    pub fn from_wrs(wrs: &WRs, year: u32, num_holidays: u32) -> Self {
        Stats {
            year: year,
            num_wrs: wrs.num_wrs(),
            num_replied_wrs: wrs.num_replied_wrs(),
            ratio_replied_wrs: wrs.ratio_replied_wrs(),
            num_skipped_wrs: wrs.num_skipped_wrs(num_holidays),
            avg_wr_delay: wrs.avg_wr_delay(),
            avg_reply_delay: wrs.avg_reply_delay(),
            weekday_wr_histogram: wrs.weekday_wr_histogram(),
            weekday_reply_histogram: wrs.weekday_reply_histogram(),
            hour_wr_histogram: wrs.hour_wr_histogram(),
            hour_reply_histogram: wrs.hour_reply_histogram(),
            cc_histogram: wrs.cc_histogram(),
        }
    }

    pub fn write_to_file(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Create the directory path if it doesn't exist
        let path = Path::new(file_path);
        if let Some(dir_path) = path.parent() {
            fs::create_dir_all(dir_path)?;
        }

        // Serialize and write to the file
        let serialized = serde_json::to_string_pretty(&self)?;
        let mut file = File::create(path)?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }
}
