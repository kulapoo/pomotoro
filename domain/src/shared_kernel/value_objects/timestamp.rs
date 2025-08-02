use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration as ChronoDuration, TimeZone};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    pub fn now() -> Self {
        Self(Utc::now())
    }

    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }

    pub fn from_timestamp_millis(millis: i64) -> Option<Self> {
        Utc.timestamp_millis_opt(millis).single().map(Self)
    }

    pub fn from_timestamp_secs(secs: i64) -> Option<Self> {
        Utc.timestamp_opt(secs, 0).single().map(Self)
    }

    pub fn to_datetime(&self) -> DateTime<Utc> {
        self.0
    }

    pub fn to_timestamp_millis(&self) -> i64 {
        self.0.timestamp_millis()
    }

    pub fn to_timestamp_secs(&self) -> i64 {
        self.0.timestamp()
    }

    pub fn add_seconds(&self, seconds: i64) -> Self {
        Self(self.0 + ChronoDuration::seconds(seconds))
    }

    pub fn add_minutes(&self, minutes: i64) -> Self {
        Self(self.0 + ChronoDuration::minutes(minutes))
    }

    pub fn add_hours(&self, hours: i64) -> Self {
        Self(self.0 + ChronoDuration::hours(hours))
    }

    pub fn add_days(&self, days: i64) -> Self {
        Self(self.0 + ChronoDuration::days(days))
    }

    pub fn duration_since(&self, other: &Timestamp) -> ChronoDuration {
        self.0.signed_duration_since(other.0)
    }

    pub fn is_before(&self, other: &Timestamp) -> bool {
        self.0 < other.0
    }

    pub fn is_after(&self, other: &Timestamp) -> bool {
        self.0 > other.0
    }

    pub fn format_iso8601(&self) -> String {
        self.0.to_rfc3339()
    }

    pub fn format_human_readable(&self) -> String {
        self.0.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}

impl Display for Timestamp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format_iso8601())
    }
}

impl From<DateTime<Utc>> for Timestamp {
    fn from(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }
}

impl From<Timestamp> for DateTime<Utc> {
    fn from(ts: Timestamp) -> Self {
        ts.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimestampRange {
    pub start: Timestamp,
    pub end: Timestamp,
}

impl TimestampRange {
    pub fn new(start: Timestamp, end: Timestamp) -> Self {
        Self { start, end }
    }

    pub fn duration(&self) -> ChronoDuration {
        self.end.duration_since(&self.start)
    }

    pub fn contains(&self, timestamp: &Timestamp) -> bool {
        timestamp.is_after(&self.start) && timestamp.is_before(&self.end)
    }

    pub fn overlaps(&self, other: &TimestampRange) -> bool {
        self.start.is_before(&other.end) && self.end.is_after(&other.start)
    }

    pub fn merge(&self, other: &TimestampRange) -> Option<TimestampRange> {
        if self.overlaps(other) {
            let start = if self.start.is_before(&other.start) {
                self.start.clone()
            } else {
                other.start.clone()
            };
            let end = if self.end.is_after(&other.end) {
                self.end.clone()
            } else {
                other.end.clone()
            };
            Some(TimestampRange::new(start, end))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_timestamp_now() {
        let ts = Timestamp::now();
        let now = Utc::now();
        assert!((ts.duration_since(&Timestamp::from_datetime(now)).num_milliseconds()).abs() < 1000);
    }

    #[test]
    fn should_create_from_millis() {
        let millis = 1640995200000; // 2022-01-01 00:00:00 UTC
        let ts = Timestamp::from_timestamp_millis(millis).unwrap();
        assert_eq!(ts.to_timestamp_millis(), millis);
    }

    #[test]
    fn should_create_from_secs() {
        let secs = 1640995200; // 2022-01-01 00:00:00 UTC
        let ts = Timestamp::from_timestamp_secs(secs).unwrap();
        assert_eq!(ts.to_timestamp_secs(), secs);
    }

    #[test]
    fn should_add_duration() {
        let ts = Timestamp::now();
        let future = ts.add_minutes(30);
        assert!(future.is_after(&ts));
        assert_eq!(future.duration_since(&ts).num_minutes(), 30);
    }

    #[test]
    fn should_compare_timestamps() {
        let ts1 = Timestamp::now();
        let ts2 = ts1.add_seconds(1);
        
        assert!(ts1.is_before(&ts2));
        assert!(ts2.is_after(&ts1));
        assert_eq!(ts2.duration_since(&ts1).num_seconds(), 1);
    }

    #[test]
    fn should_format_timestamps() {
        let dt = Utc.with_ymd_and_hms(2022, 1, 1, 12, 0, 0).unwrap();
        let ts = Timestamp::from_datetime(dt);
        
        assert_eq!(ts.format_iso8601(), "2022-01-01T12:00:00+00:00");
        assert_eq!(ts.format_human_readable(), "2022-01-01 12:00:00 UTC");
    }

    #[test]
    fn should_work_with_timestamp_range() {
        let start = Timestamp::now();
        let end = start.add_hours(1);
        let range = TimestampRange::new(start.clone(), end.clone());
        
        let middle = start.add_minutes(30);
        assert!(range.contains(&middle));
        assert_eq!(range.duration().num_hours(), 1);
    }

    #[test]
    fn should_detect_overlapping_ranges() {
        let range1 = TimestampRange::new(
            Timestamp::now(),
            Timestamp::now().add_hours(2)
        );
        let range2 = TimestampRange::new(
            Timestamp::now().add_hours(1),
            Timestamp::now().add_hours(3)
        );
        
        assert!(range1.overlaps(&range2));
        assert!(range2.overlaps(&range1));
    }

    #[test]
    fn should_merge_overlapping_ranges() {
        let range1 = TimestampRange::new(
            Timestamp::now(),
            Timestamp::now().add_hours(2)
        );
        let range2 = TimestampRange::new(
            Timestamp::now().add_hours(1),
            Timestamp::now().add_hours(3)
        );
        
        let merged = range1.merge(&range2).unwrap();
        assert_eq!(merged.duration().num_hours(), 3);
    }
}