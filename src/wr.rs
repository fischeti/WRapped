use chrono::{Datelike, Timelike};
use log::info;
use std::collections::HashMap;

use crate::mail::Mail;

pub fn merge_wrs(wrs: &[Mail], wrs_re: &[Mail]) -> WRs {
    let mut merged_wrs = WRs::new();
    for wr_mail in wrs.iter() {
        let mut wr = WR::new(wr_mail.clone(), None);
        for re_mail in wrs_re.iter() {
            if let Some(message_id) = wr_mail.env.message_id.as_ref() {
                if let Some(in_reply_to) = re_mail.env.in_reply_to.as_ref() {
                    if message_id.eq(in_reply_to) {
                        wr.reply = Some(re_mail.clone());
                        break;
                    }
                }
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

    pub fn wr_delay(&self) -> i64 {
        let weekday = self.sent.env.date.weekday();
        let days_since_friday = (weekday.num_days_from_monday() + 2) % 7;
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
        let num_replied_wrs: usize = self.num_replied_wrs();
        let num_wrs = self.num_wrs();
        num_replied_wrs as f64 / num_wrs as f64
    }

    pub fn avg_wr_delay(&self) -> f64 {
        let num_replied_wrs = self.num_replied_wrs();
        let wr_delay_sum: i64 = self.wrs.iter().map(|wr| wr.wr_delay()).sum();
        wr_delay_sum as f64 / num_replied_wrs as f64
    }

    pub fn avg_reply_delay(&self) -> f64 {
        let num_replied_wrs = self.num_replied_wrs();
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
        for wr in self.wrs.iter() {
            match wr.reply {
                Some(_) => {
                    let weekday = wr.sent.env.date.weekday();
                    hist.entry(weekday as u32).and_modify(|e| *e += 1);
                }
                None => continue,
            };
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
        for wr in self.wrs.iter() {
            match wr.reply {
                Some(_) => {
                    let hour = wr.sent.env.date.hour();
                    hist.entry(hour).and_modify(|e| *e += 1);
                }
                None => continue,
            };
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
