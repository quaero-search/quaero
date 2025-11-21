use anyhttp::HeaderValue;
use rand::Rng;
use std::borrow::Cow;

/// A randomly generated User Agent.
pub struct UserAgent<'a>(Cow<'a, str>);

impl<'a> UserAgent<'a> {
    /// Creates a new random user agent.
    pub fn random() -> Self {
        let mut rng = rand::rng();
        let chosen = USER_AGENTS[rng.random_range(0..USER_AGENTS.len())];
        Self(Cow::Borrowed(chosen))
    }

    /// Creates a new random user agent for a browser which doesn't support JavaScript.
    /// Used for engines which enforce the use of JavaScript except for certain obscure old devices.
    pub fn random_no_js() -> Self {
        let mut rng = rand::rng();
        let chosen =
            USER_AGENTS_ALLOWED_NO_JS[rng.random_range(0..USER_AGENTS_ALLOWED_NO_JS.len())];
        Self(Cow::Borrowed(chosen))
    }
}

impl<'a> Into<HeaderValue> for UserAgent<'a> {
    fn into(self) -> HeaderValue {
        HeaderValue::from_str(self.0.as_ref()).unwrap()
    }
}

const USER_AGENTS: [&str; 10] = [
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:131.0) Gecko/20100101 Firefox/131.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 13_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Edg/129.0.0.0",
    "Mozilla/5.0 (Linux; Android 13; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Mobile Safari/537.36",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1",
    "Mozilla/5.0 (Linux; Android 13; SM-G998B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Mobile Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; WOW64; rv:131.0) Gecko/20100101 Firefox/131.0",
];

const USER_AGENTS_ALLOWED_NO_JS: [&'static str; 14] = [
    "Mozilla/5.0 (webOS/1.4.5; U; en-US) AppleWebKit/532.2 (KHTML, like Gecko) Version/1.0 Safari/532.2 Pre/1.0",
    "Mozilla/5.0 (webOS/1.4.0; U; en-US) AppleWebKit/532.2 (KHTML, like Gecko) Version/1.0 Safari/532.2 Pre/1.0",
    "Mozilla/5.0 (webOS/1.3.5; U; en-US) AppleWebKit/532.2 (KHTML, like Gecko) Version/1.0 Safari/532.2 Pre/1.0",
    "Mozilla/5.0 (webOS/2.0.0; U; en-US) AppleWebKit/534.6 (KHTML, like Gecko) Version/1.0 Safari/534.6 Pre/2.0",
    "Mozilla/5.0 (webOS/2.1.0; U; en-US) AppleWebKit/534.6 (KHTML, like Gecko) Version/1.0 Safari/534.6 Pre/2.1",
    "Mozilla/5.0 (webOS/3.0.5; U; en-US) AppleWebKit/534.6 (KHTML, like Gecko) TouchPad/1.0",
    "Mozilla/5.0 (webOS/3.0.2; U; en-US) AppleWebKit/534.6 (KHTML, like Gecko) TouchPad/1.0",
    "Mozilla/5.0 (webOS/1.2.1; U; en-GB) AppleWebKit/532.2 (KHTML, like Gecko) Version/1.0 Safari/532.2 Pre/1.0",
    "Mozilla/5.0 (webOS/1.4.0; U; fr-FR) AppleWebKit/532.2 (KHTML, like Gecko) Version/1.0 Safari/532.2 Pre/1.0",
    "Mozilla/5.0 (webOS/1.4.1; U; de-DE) AppleWebKit/532.2 (KHTML, like Gecko) Version/1.0 Safari/532.2 Pre/1.0",
    "Mozilla/5.0 (webOS/1.3.1; U; en-US) AppleWebKit/532.2 (KHTML, like Gecko) Version/1.0 Safari/532.2 Pre/1.0",
    "Mozilla/5.0 (webOS/2.0.1; U; en-US) AppleWebKit/534.6 (KHTML, like Gecko) Version/1.0 Safari/534.6 Pre/2.0",
    "Mozilla/5.0 (webOS/1.4.5; U; en-US; Pixi) AppleWebKit/532.2 (KHTML, like Gecko) Version/1.0 Safari/532.2 Pre/1.0",
    "Mozilla/5.0 (webOS/1.4.5; U; en-US; Pre) AppleWebKit/532.2 (KHTML, like Gecko) Version/1.0 Safari/532.2 Pre/1.0",
];
