use chrono::{DateTime, Utc};

/// A range between two DateTime's
pub struct DateTimeRange {
    /// The start of the range.
    pub start: DateTime<Utc>,

    /// The end of the range.
    pub end: DateTime<Utc>,
}

impl DateTimeRange {
    /// Creates a new DateTime range.
    pub fn new(start: impl Into<DateTime<Utc>>, end: impl Into<DateTime<Utc>>) -> Self {
        Self {
            start: start.into(),
            end: end.into(),
        }
    }

    /// Finds the closest DateTime from a list of presets. Used for
    /// engines which don't support searching by a custom DateTime range.
    pub fn find_closest_preset<'a, const N: usize, E: ?Sized>(
        &self,
        presets: &[(chrono::Duration, &'a E); N],
    ) -> &'a E {
        let now = Utc::now();
        let (range_start, range_end) = (&self.start, &self.end);

        for (preset, value) in presets {
            let (preset_start, preset_end) = (now - *preset, now + *preset);

            if range_start >= &preset_start && range_end <= &preset_end {
                return value;
            }
        }

        return presets.last().unwrap().1;
    }
}
