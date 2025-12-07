//! A collection of engines for Quaero that scrapes results from various search engines.

#![warn(missing_docs)]

use quaero_shared::models::engine::TaggedEngine;

macro_rules! pub_use_modules {
    ($($name:ident),+) => {
        $(
            mod $name;
            pub use $name::*;
        )+
    };
}

pub_use_modules![bing, brave, google, mojeek, yahoo, yandex];

/// Constructs a list of the default engines.
#[inline(always)]
pub fn default() -> [TaggedEngine; 6] {
    [
        BingEngine::new(),
        BraveEngine::new(),
        GoogleEngine::new(),
        MojeekEngine::new(),
        YahooEngine::new(),
        YandexEngine::new(),
    ]
}
