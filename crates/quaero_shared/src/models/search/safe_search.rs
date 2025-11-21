/// A filtering scale which controls how strictly explicit search results are blocked.
#[derive(Clone, Default)]
pub enum SafeSearch {
    /// No explicit results are blocked.
    Off = 0,

    /// Overtly explicit results are blocked.
    #[default]
    Moderate = 1,

    /// Explicit results are strictly blocked.
    Strict = 2,
}

impl SafeSearch {
    /// Converts to a lowercase string ("off", "moderate" and "strict").
    pub fn as_lowercase_string(&self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Moderate => "moderate",
            Self::Strict => "strict",
        }
    }

    /// Converts to an incrementing number (Off = 0, Moderate = 1 and Strict = 2).
    pub fn as_incrementing_usize(&self) -> isize {
        self.clone() as isize
    }

    /// Converts to an decrementing number (Off = 2, Moderate = 1 and Strict = 0).
    pub fn as_decrementing_usize(&self) -> isize {
        2 - self.as_incrementing_usize()
    }

    /// Returns `true` if safe search is Moderate or Strict.
    pub fn as_bool(&self) -> bool {
        match self {
            Self::Off => false,
            Self::Moderate | Self::Strict => true,
        }
    }
}
