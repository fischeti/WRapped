use chrono::{Datelike, Timelike};
use log::info;
use std::collections::HashMap;

use crate::mail::Envelope;

pub fn merge_wrs(wr: &[Envelope], wr_re: &[Envelope]) -> WRs {
    let mut wrs = WRs::new();

    for w in wr.iter() {
        let mut wr = WR::new(w.clone(), None);
        for r in wr_re.iter() {
            if let Some(message_id) = wr.sent.message_id.as_ref() {
                if let Some(in_reply_to) = r.in_reply_to.as_ref() {
                    if message_id.eq(in_reply_to) {
                        wr.reply = Some(r.clone());
                        break;
                    }
                }
            }
        }
        wrs.wrs.push(wr);
    }

    info!(
        "Found {} Replies to {} WRs",
        wrs.num_replied_wrs(),
        wrs.num_wrs()
    );
    wrs
}

#[derive(Debug)]
pub struct WR {
    // The Envelope of the WR that was sent
    pub sent: Envelope,
    // The Envelope of the WR reply that was received, if any
    pub reply: Option<Envelope>,
}

impl WR {
    pub fn new(sent: Envelope, reply: Option<Envelope>) -> Self {
        WR { sent, reply }
    }

    pub fn wr_delay(&self) -> i64 {
        let weekday = self.sent.date.weekday();
        let days_since_friday = (weekday.num_days_from_monday() + 2) % 7;
        days_since_friday as i64
    }

    pub fn reply_delay(&self) -> Option<i64> {
        match self.reply {
            Some(ref reply) => {
                let sent_date = self.sent.date;
                let reply_date = reply.date;
                let duration = reply_date.signed_duration_since(sent_date);
                Some(duration.num_days())
            }
            None => None,
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
            let weekday = wr.sent.date.weekday();
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
                    let weekday = wr.sent.date.weekday();
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
            let hour = wr.sent.date.hour();
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
                    let hour = wr.sent.date.hour();
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
            match wr.sent.cc {
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
