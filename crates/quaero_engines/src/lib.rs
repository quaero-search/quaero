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
