use chrono::{Duration, Utc};

use crate::models::search::DateTimeRange;

/// Commonly used presets for [DateTimeRange].
pub enum DateTimeRangePreset {
    /// A range from an hour ago to now.
    PastHour,

    /// A range from a day ago to now.
    PastDay,

    /// A range from a week ago to now.
    PastWeek,

    /// A range from a month ago to now.
    PastMonth,

    /// A range from a year ago to now.
    PastYear,
}

impl Into<DateTimeRange> for DateTimeRangePreset {
    fn into(self) -> DateTimeRange {
        let now = Utc::now();

        DateTimeRange::new(
            match self {
                Self::PastHour => now - Duration::hours(1),
                Self::PastDay => now - Duration::days(1),
                Self::PastWeek => now - Duration::weeks(1),
                Self::PastMonth => now - Duration::weeks(4),
                Self::PastYear => now - Duration::days(365),
            },
            now,
        )
    }
}
