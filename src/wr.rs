use chrono::{Datelike, Timelike};
use log::info;
use std::collections::HashMap;

use crate::mail::Mail;

pub fn merge_wrs(wrs: &[Mail], wrs_re: &[Mail]) -> WRs {
    let mut merged_wrs = WRs::new();
    for wr_mail in wrs.iter() {
        let mut wr = WR::new(wr_mail.clone(), None);
        for re_mail in wrs_re.iter() {
            if let Some(message_id) = wr_mail.env.message_id.as_ref()
                && let Some(in_reply_to) = re_mail.env.in_reply_to.as_ref()
                && message_id.eq(in_reply_to)
            {
                wr.reply = Some(re_mail.clone());
                break;
            }
        }
        merged_wrs.wrs.push(wr);
    }

    info!(
        "Merged {} Replies with {} WRs",
        merged_wrs.num_replied_wrs(),
        merged_wrs.num_wrs()
    );
    merged_wrs
}

#[derive(Debug)]
pub struct WR {
    // The Envelope of the WR that was sent
    pub sent: Mail,
    // The Envelope of the WR reply that was received, if any
    pub reply: Option<Mail>,
}

impl WR {
    pub fn new(sent: Mail, reply: Option<Mail>) -> Self {
        WR { sent, reply }
    }

    /// How many days after the Friday of that week the WR was sent, so a WR
    /// sent on Friday itself has a delay of 0 and one sent on Thursday of 6.
    pub fn wr_delay(&self) -> i64 {
        let weekday = self.sent.env.date.weekday();
        let days_since_friday = (weekday.num_days_from_monday() + 3) % 7;
        days_since_friday as i64
    }

    pub fn reply_delay(&self) -> Option<i64> {
        match self.reply {
            Some(ref reply) => {
                let sent_date = self.sent.env.date;
                let reply_date = reply.env.date;
                let duration = reply_date.signed_duration_since(sent_date);
                Some(duration.num_days())
            }
            None => None,
        }
    }

    pub fn num_words(&self) -> usize {
        match self.sent.body {
            Some(ref body) => body.split_whitespace().count(),
            None => 0,
        }
    }
}

#[derive(Debug, Default)]
pub struct WRs {
    // All the WRs that were sent and received
    pub wrs: Vec<WR>,
}

impl WRs {
    pub fn new() -> Self {
        WRs { wrs: Vec::new() }
    }

    pub fn push(&mut self, wr: WR) {
        self.wrs.push(wr);
    }

    pub fn pop(&mut self) -> Option<WR> {
        self.wrs.pop()
    }

    pub fn len(&self) -> usize {
        self.wrs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.wrs.is_empty()
    }

    pub fn num_wrs(&self) -> usize {
        self.wrs.len()
    }

    pub fn num_replied_wrs(&self) -> usize {
        self.wrs.iter().filter(|wr| wr.reply.is_some()).count()
    }

    pub fn num_words(&self) -> usize {
        self.wrs.iter().map(|wr| wr.num_words()).sum()
    }

    pub fn ratio_replied_wrs(&self) -> f64 {
        let num_wrs = self.num_wrs();
        if num_wrs == 0 {
            return 0.0;
        }
        self.num_replied_wrs() as f64 / num_wrs as f64
    }

    pub fn avg_wr_delay(&self) -> f64 {
        // The delay is defined for every WR, so average over all of them
        let num_wrs = self.num_wrs();
        if num_wrs == 0 {
            return 0.0;
        }
        let wr_delay_sum: i64 = self.wrs.iter().map(|wr| wr.wr_delay()).sum();
        wr_delay_sum as f64 / num_wrs as f64
    }

    pub fn avg_reply_delay(&self) -> f64 {
        // Only WRs that got a reply have a reply delay
        let num_replied_wrs = self.num_replied_wrs();
        if num_replied_wrs == 0 {
            return 0.0;
        }
        let reply_delay_sum: i64 = self.wrs.iter().filter_map(|wr| wr.reply_delay()).sum();
        reply_delay_sum as f64 / num_replied_wrs as f64
    }

    pub fn weekday_wr_histogram(&self) -> HashMap<u32, u32> {
        let mut hist = HashMap::new();

        for day in 0..7 {
            hist.insert(day, 0);
        }
        for wr in self.wrs.iter() {
            let weekday = wr.sent.env.date.weekday();
            hist.entry(weekday as u32).and_modify(|e| *e += 1);
        }
        hist
    }

    pub fn weekday_reply_histogram(&self) -> HashMap<u32, u32> {
        let mut hist = HashMap::new();

        for day in 0..7 {
            hist.insert(day, 0);
        }
        for reply in self.wrs.iter().filter_map(|wr| wr.reply.as_ref()) {
            let weekday = reply.env.date.weekday();
            hist.entry(weekday as u32).and_modify(|e| *e += 1);
        }
        hist
    }

    pub fn hour_wr_histogram(&self) -> HashMap<u32, u32> {
        let mut hist = HashMap::new();

        for hour in 0..24 {
            hist.insert(hour, 0);
        }
        for wr in self.wrs.iter() {
            let hour = wr.sent.env.date.hour();
            hist.entry(hour).and_modify(|e| *e += 1);
        }
        hist
    }

    pub fn hour_reply_histogram(&self) -> HashMap<u32, u32> {
        let mut hist = HashMap::new();

        for hour in 0..24 {
            hist.insert(hour, 0);
        }
        for reply in self.wrs.iter().filter_map(|wr| wr.reply.as_ref()) {
            let hour = reply.env.date.hour();
            hist.entry(hour).and_modify(|e| *e += 1);
        }
        hist
    }

    pub fn cc_histogram(&self) -> HashMap<String, u32> {
        let mut hist = HashMap::new();

        for wr in self.wrs.iter() {
            match wr.sent.env.cc {
                Some(ref cc) => {
                    for addr in cc.iter() {
                        if let Some(user) = &addr.user {
                            hist.entry(user.to_string())
                                .and_modify(|e| *e += 1)
                                .or_insert(1);
                        }
                    }
                }
                None => continue,
            };
        }
        hist
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mail::{Address, Envelope};
    use chrono::DateTime;

    /// Build a mail sent at `date`, which must be in RFC2822 format.
    fn mail(id: u32, date: &str, message_id: &str, in_reply_to: Option<&str>) -> Mail {
        Mail {
            id,
            env: Envelope {
                date: DateTime::parse_from_rfc2822(date).unwrap(),
                subject: "WR".to_string(),
                cc: None,
                in_reply_to: in_reply_to.map(|s| s.to_string()),
                message_id: Some(message_id.to_string()),
            },
            body: None,
        }
    }

    fn cc(users: &[&str]) -> Option<Vec<Address>> {
        Some(
            users
                .iter()
                .map(|user| Address {
                    name: None,
                    user: Some(user.to_string()),
                    email: Some(format!("{}@example.com", user)),
                })
                .collect(),
        )
    }

    #[test]
    fn merges_replies_by_message_id() {
        // Two WRs, but the reply refers to the second one
        let wrs = vec![
            mail(1, "Fri, 6 Jan 2023 17:00:00 +0100", "<wr-1>", None),
            mail(2, "Fri, 13 Jan 2023 17:00:00 +0100", "<wr-2>", None),
        ];
        let replies = vec![mail(
            3,
            "Mon, 16 Jan 2023 09:00:00 +0100",
            "<re-2>",
            Some("<wr-2>"),
        )];

        let merged = merge_wrs(&wrs, &replies);

        assert_eq!(merged.num_wrs(), 2);
        assert_eq!(merged.num_replied_wrs(), 1);
        assert!(merged.wrs[0].reply.is_none());
        assert_eq!(
            merged.wrs[1].reply.as_ref().unwrap().env.message_id,
            Some("<re-2>".to_string())
        );
    }

    #[test]
    fn does_not_merge_unrelated_replies() {
        let wrs = vec![mail(1, "Fri, 6 Jan 2023 17:00:00 +0100", "<wr-1>", None)];
        let replies = vec![mail(
            2,
            "Mon, 9 Jan 2023 09:00:00 +0100",
            "<re-x>",
            Some("<some-other-mail>"),
        )];

        let merged = merge_wrs(&wrs, &replies);

        assert_eq!(merged.num_replied_wrs(), 0);
        assert_eq!(merged.ratio_replied_wrs(), 0.0);
    }

    #[test]
    fn wr_delay_counts_days_since_friday() {
        // Friday is the reference day, so a WR sent on Friday has no delay
        let friday = WR::new(mail(1, "Fri, 6 Jan 2023 17:00:00 +0100", "<a>", None), None);
        let saturday = WR::new(mail(2, "Sat, 7 Jan 2023 17:00:00 +0100", "<b>", None), None);
        let monday = WR::new(mail(3, "Mon, 9 Jan 2023 17:00:00 +0100", "<c>", None), None);
        let thursday = WR::new(
            mail(4, "Thu, 12 Jan 2023 17:00:00 +0100", "<d>", None),
            None,
        );

        assert_eq!(friday.wr_delay(), 0);
        assert_eq!(saturday.wr_delay(), 1);
        assert_eq!(monday.wr_delay(), 3);
        assert_eq!(thursday.wr_delay(), 6);
    }

    #[test]
    fn avg_wr_delay_averages_over_all_wrs() {
        // Three WRs with delays 0, 3 and 3, only one of which got a reply.
        // The average has to be over all three WRs, not just the replied one.
        let wrs = WRs {
            wrs: vec![
                WR::new(mail(1, "Fri, 6 Jan 2023 17:00:00 +0100", "<a>", None), None),
                WR::new(
                    mail(2, "Mon, 16 Jan 2023 17:00:00 +0100", "<b>", None),
                    None,
                ),
                WR::new(
                    mail(3, "Mon, 23 Jan 2023 17:00:00 +0100", "<c>", None),
                    Some(mail(
                        4,
                        "Tue, 24 Jan 2023 09:00:00 +0100",
                        "<re-c>",
                        Some("<c>"),
                    )),
                ),
            ],
        };

        assert_eq!(wrs.avg_wr_delay(), 2.0);
    }

    #[test]
    fn avg_reply_delay_averages_over_replied_wrs_only() {
        let wrs = WRs {
            wrs: vec![
                // Replied to after 2 days
                WR::new(
                    mail(1, "Fri, 6 Jan 2023 12:00:00 +0100", "<a>", None),
                    Some(mail(
                        2,
                        "Sun, 8 Jan 2023 12:00:00 +0100",
                        "<re-a>",
                        Some("<a>"),
                    )),
                ),
                // Replied to after 4 days
                WR::new(
                    mail(3, "Fri, 13 Jan 2023 12:00:00 +0100", "<b>", None),
                    Some(mail(
                        4,
                        "Tue, 17 Jan 2023 12:00:00 +0100",
                        "<re-b>",
                        Some("<b>"),
                    )),
                ),
                // Never replied to, must not drag the average down
                WR::new(
                    mail(5, "Fri, 20 Jan 2023 12:00:00 +0100", "<c>", None),
                    None,
                ),
            ],
        };

        assert_eq!(wrs.avg_reply_delay(), 3.0);
        assert!((wrs.ratio_replied_wrs() - 2.0 / 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn reply_histograms_use_the_date_of_the_reply() {
        // WR sent on Friday evening, replied to on Monday morning
        let wrs = WRs {
            wrs: vec![WR::new(
                mail(1, "Fri, 6 Jan 2023 18:00:00 +0100", "<a>", None),
                Some(mail(
                    2,
                    "Mon, 9 Jan 2023 08:00:00 +0100",
                    "<re-a>",
                    Some("<a>"),
                )),
            )],
        };

        // Monday is 0 for chrono, Friday is 4
        let weekday_hist = wrs.weekday_reply_histogram();
        assert_eq!(weekday_hist[&0], 1, "the reply arrived on a Monday");
        assert_eq!(
            weekday_hist[&4], 0,
            "the WR, not the reply, was sent Friday"
        );

        let hour_hist = wrs.hour_reply_histogram();
        assert_eq!(hour_hist[&8], 1, "the reply arrived at 08:00");
        assert_eq!(
            hour_hist[&18], 0,
            "the WR, not the reply, was sent at 18:00"
        );
    }

    #[test]
    fn wr_histograms_use_the_date_of_the_wr() {
        let wrs = WRs {
            wrs: vec![WR::new(
                mail(1, "Fri, 6 Jan 2023 18:00:00 +0100", "<a>", None),
                Some(mail(
                    2,
                    "Mon, 9 Jan 2023 08:00:00 +0100",
                    "<re-a>",
                    Some("<a>"),
                )),
            )],
        };

        assert_eq!(wrs.weekday_wr_histogram()[&4], 1);
        assert_eq!(wrs.hour_wr_histogram()[&18], 1);
    }

    #[test]
    fn histograms_cover_every_bucket() {
        let wrs = WRs::new();

        assert_eq!(wrs.weekday_wr_histogram().len(), 7);
        assert_eq!(wrs.weekday_reply_histogram().len(), 7);
        assert_eq!(wrs.hour_wr_histogram().len(), 24);
        assert_eq!(wrs.hour_reply_histogram().len(), 24);
    }

    #[test]
    fn counts_words_of_all_wr_bodies() {
        let mut first = mail(1, "Fri, 6 Jan 2023 17:00:00 +0100", "<a>", None);
        first.body = Some("Did some\nwork  this week".to_string());
        let mut second = mail(2, "Fri, 13 Jan 2023 17:00:00 +0100", "<b>", None);
        second.body = Some("Nothing".to_string());
        // A WR whose body could not be fetched counts as zero words
        let third = mail(3, "Fri, 20 Jan 2023 17:00:00 +0100", "<c>", None);

        let wrs = WRs {
            wrs: vec![
                WR::new(first, None),
                WR::new(second, None),
                WR::new(third, None),
            ],
        };

        assert_eq!(wrs.num_words(), 6);
    }

    #[test]
    fn cc_histogram_counts_every_recipient() {
        let mut first = mail(1, "Fri, 6 Jan 2023 17:00:00 +0100", "<a>", None);
        first.env.cc = cc(&["alice", "bob"]);
        let mut second = mail(2, "Fri, 13 Jan 2023 17:00:00 +0100", "<b>", None);
        second.env.cc = cc(&["alice"]);
        // A WR without anyone in CC
        let third = mail(3, "Fri, 20 Jan 2023 17:00:00 +0100", "<c>", None);

        let wrs = WRs {
            wrs: vec![
                WR::new(first, None),
                WR::new(second, None),
                WR::new(third, None),
            ],
        };

        let hist = wrs.cc_histogram();
        assert_eq!(hist[&"alice".to_string()], 2);
        assert_eq!(hist[&"bob".to_string()], 1);
        assert_eq!(hist.len(), 2);
    }

    #[test]
    fn empty_stats_do_not_divide_by_zero() {
        let wrs = WRs::new();

        assert_eq!(wrs.num_wrs(), 0);
        assert!(wrs.ratio_replied_wrs().is_finite());
        assert!(wrs.avg_wr_delay().is_finite());
        assert!(wrs.avg_reply_delay().is_finite());
    }
}
