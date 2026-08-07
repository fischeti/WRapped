use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use colored::Colorize;
use serde::Serialize;
use serde_json;

use crate::error::{Result, WrError};
use crate::palette::{Palette, label};
use crate::wr::WRs;

// The weekday histograms are keyed by `chrono::Weekday as u32`, which counts
// from Monday
const WEEKDAYS: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];

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
    // The number of words written
    pub num_words: usize,
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
    pub fn from_wrs(wrs: &WRs, year: u32) -> Self {
        Stats {
            year,
            num_wrs: wrs.num_wrs(),
            num_replied_wrs: wrs.num_replied_wrs(),
            ratio_replied_wrs: wrs.ratio_replied_wrs(),
            num_words: wrs.num_words(),
            avg_wr_delay: wrs.avg_wr_delay(),
            avg_reply_delay: wrs.avg_reply_delay(),
            weekday_wr_histogram: wrs.weekday_wr_histogram(),
            weekday_reply_histogram: wrs.weekday_reply_histogram(),
            hour_wr_histogram: wrs.hour_wr_histogram(),
            hour_reply_histogram: wrs.hour_reply_histogram(),
            cc_histogram: wrs.cc_histogram(),
        }
    }

    /// The statistics as the JSON the web app fetches.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self).map_err(|e| WrError::SerializationError(e.to_string()))
    }

    pub fn write_to_file(&self, path: &Path) -> Result<()> {
        // Create the directory path if it doesn't exist
        if let Some(dir_path) = path.parent()
            && !dir_path.as_os_str().is_empty()
        {
            fs::create_dir_all(dir_path)?;
        }

        let mut file = File::create(path)?;
        file.write_all(self.to_json()?.as_bytes())?;
        Ok(())
    }

    /// Print the statistics to the terminal, in the colors the web app uses
    /// for that year.
    pub fn print_summary(&self) {
        let palette = Palette::for_year(self.year);

        self.print_title(&palette);
        self.print_metrics(&palette);
        self.print_cc_leaderboard(&palette);
    }

    fn print_title(&self, palette: &Palette) {
        // Lay the frame out around the plain text, since the color codes of
        // the title itself do not take up any width
        let width = format!("  WRapped {}  ", self.year).chars().count();
        let horizontal = "─".repeat(width);
        let bar = palette.frame("│");

        println!();
        println!("  {}", palette.frame(&format!("╭{}╮", horizontal)));
        println!(
            "  {}  {}{} {}  {}",
            bar,
            palette.highlight("WR"),
            "apped".bold(),
            palette.accent(&self.year.to_string()),
            bar
        );
        println!("  {}", palette.frame(&format!("╰{}╯", horizontal)));
        println!();
    }

    fn print_metrics(&self, palette: &Palette) {
        // Pad before coloring, so that the color codes do not count towards
        // the width of the column
        let metric = |value: String, text: &str| {
            println!(
                "   {}  {}",
                palette.highlight(&format!("{:>8}", value)),
                label(text)
            );
        };

        metric(thousands(self.num_wrs), "WRs written");
        metric(thousands(self.num_words), "words in total");
        metric(
            format!("{:.0}%", self.ratio_replied_wrs * 100.0),
            "of them got a reply",
        );
        metric(
            format!("{:.1}", self.avg_wr_delay),
            "days after Friday you sent them",
        );
        metric(
            format!("{:.1}", self.avg_reply_delay),
            "days until a reply arrived",
        );
        if let Some(weekday) = busiest_weekday(&self.weekday_wr_histogram) {
            metric(weekday.to_string(), "is when you wrote most of them");
        }
    }

    fn print_cc_leaderboard(&self, palette: &Palette) {
        let mut leaderboard: Vec<_> = self.cc_histogram.iter().collect();
        if leaderboard.is_empty() {
            return;
        }
        // Sort by count, and by name so that ties are printed in a stable order
        leaderboard.sort_by(|(a_user, a_count), (b_user, b_count)| {
            b_count.cmp(a_count).then(a_user.cmp(b_user))
        });

        println!();
        println!("   {}", label("The people you CC'd most"));
        for (place, (user, count)) in leaderboard.iter().take(3).enumerate() {
            println!(
                "   {}  {}  {}",
                ["🏆", "🥈", "🥉"][place],
                palette.accent(&format!("{:<20}", user)),
                palette.secondary(&format!("{}x", count))
            );
        }
        println!();
    }
}

/// Format a number with `,` as a thousands separator.
fn thousands(number: usize) -> String {
    let digits = number.to_string();
    let mut grouped = String::new();
    for (i, digit) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    grouped
}

/// The name of the weekday most WRs were sent on, or `None` if there are no
/// WRs at all.
fn busiest_weekday(histogram: &HashMap<u32, u32>) -> Option<&'static str> {
    histogram
        .iter()
        .filter(|&(_, &count)| count > 0)
        // Prefer the earlier weekday when two of them tie
        .max_by(|(a_day, a_count), (b_day, b_count)| a_count.cmp(b_count).then(b_day.cmp(a_day)))
        .and_then(|(&day, _)| WEEKDAYS.get(day as usize).copied())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stats(year: u32, cc: &[(&str, u32)]) -> Stats {
        Stats {
            year,
            num_wrs: 38,
            num_replied_wrs: 34,
            ratio_replied_wrs: 34.0 / 38.0,
            num_words: 49_364,
            avg_wr_delay: 2.0,
            avg_reply_delay: 1.588,
            weekday_wr_histogram: HashMap::from([(0, 22), (4, 6), (5, 4), (6, 2)]),
            weekday_reply_histogram: HashMap::from([(0, 20), (1, 2)]),
            hour_wr_histogram: HashMap::from([(17, 30), (23, 8)]),
            hour_reply_histogram: HashMap::from([(9, 34)]),
            cc_histogram: cc
                .iter()
                .map(|(user, count)| (user.to_string(), *count))
                .collect(),
        }
    }

    #[test]
    fn prints_a_summary_for_every_palette() {
        colored::control::set_override(true);

        // More people in CC than the leaderboard has places for
        let cc = [("alice", 24), ("bob", 18), ("carol", 9), ("dave", 3)];
        for year in [2021, 2022, 2023, 2025] {
            stats(year, &cc).print_summary();
        }
    }

    #[test]
    fn prints_a_summary_without_anyone_in_cc() {
        stats(2025, &[]).print_summary();
    }

    #[test]
    fn groups_digits_in_threes() {
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(7), "7");
        assert_eq!(thousands(999), "999");
        assert_eq!(thousands(1_000), "1,000");
        assert_eq!(thousands(49_364), "49,364");
        assert_eq!(thousands(1_234_567), "1,234,567");
    }

    #[test]
    fn finds_the_weekday_with_the_most_wrs() {
        // Monday is 0 for chrono, Friday is 4
        let histogram = HashMap::from([(0, 3), (4, 12), (5, 1)]);

        assert_eq!(busiest_weekday(&histogram), Some("Friday"));
    }

    #[test]
    fn breaks_a_tie_with_the_earlier_weekday() {
        let histogram = HashMap::from([(2, 5), (4, 5)]);

        assert_eq!(busiest_weekday(&histogram), Some("Wednesday"));
    }

    #[test]
    fn has_no_busiest_weekday_without_wrs() {
        assert_eq!(busiest_weekday(&HashMap::new()), None);
        // An empty histogram still has a bucket per weekday
        let empty = HashMap::from_iter((0..7).map(|day| (day, 0)));
        assert_eq!(busiest_weekday(&empty), None);
    }
}
