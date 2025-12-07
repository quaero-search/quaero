use anyhttp::HttpClient;
use http::Request;
use itertools::Itertools;
use phf::{phf_map, phf_set};
use publicsuffix::{List, Psl};
use std::{borrow::Cow, path::PathBuf, sync::LazyLock};
use tokio::sync::RwLock;

use crate::utils::{NormalizePath, StringClip};

// TODO: support strict sanitising hash.
// TODO: support removing hash.
/// Contains data pertaining to a sanitized URL.
/// Removes trackers and normalises.
#[derive(Debug)]
pub struct SanitizedUrl {
    protocol: Option<String>,
    host: String,
    port: Option<String>,
    path: Option<PathBuf>,
    search_params: Option<Vec<(String, String)>>,
}

impl SanitizedUrl {
    /// Creates a new sanitized url.
    pub fn new(
        url_str: &str,
        filter_search_param: impl for<'b> Fn(&'b str, &'b str) -> bool,
    ) -> SanitizedUrl {
        let url_str = urlencoding::decode(url_str).unwrap_or(Cow::Borrowed(url_str));

        // Extracts search params.
        let split = url_str.split_once("?").or_else(|| url_str.split_once("&"));
        let (rest, search_params) = match split {
            Some((rest, search_params)) => (rest, Some(search_params)),
            None => (url_str.as_ref(), None),
        };

        // Extracts protocol.
        let (protocol, rest) = match rest.split_once("://") {
            Some((protocol, host)) => (Some(protocol.to_lowercase()), host),
            None => (None, url_str.as_ref()),
        };

        // Extracts path.
        let (rest, path) = match rest.split_once("/") {
            Some((rest, path)) => (
                rest,
                if path.is_empty() {
                    None
                } else {
                    Some(path.trim_start_matches("/").normalize())
                },
            ),
            None => (rest, None),
        };

        // Extracts port.
        let (host, port) = match rest.split_once(":") {
            Some((host, port)) => (host, Some(port.to_string())),
            None => (rest, None),
        };

        let host = normalize_host(host).to_string();

        // Parses extracted search params.
        let search_params = match search_params {
            Some(search_params) => {
                let search_params =
                    parse_search_params(&host, path.as_ref(), search_params, filter_search_param)
                        .map(|(key, value)| (key.to_string(), value.to_string()));

                let is_empty = match search_params.try_len() {
                    Ok(len) => len == 0,
                    Err((low, high)) => {
                        low == 0
                            && match high {
                                Some(high) => high == 0,
                                None => false,
                            }
                    }
                };

                if is_empty {
                    None
                } else {
                    Some(search_params.collect())
                }
            }
            None => None,
        };

        SanitizedUrl {
            protocol,
            host,
            port,
            path,
            search_params,
        }
    }

    /// Converts the url into a string with stricter normalisation applied.
    pub fn to_strict_string(&self) -> String {
        let path = match self.path.as_ref() {
            Some(path) => 'path: {
                let mut components = path.components().into_iter();
                let mut path_str = String::new();

                let Some(mut last) = components.next() else {
                    break 'path Some(path.to_string_lossy());
                };

                for component in components {
                    path_str.push_str(
                        std::mem::replace(&mut last, component)
                            .as_os_str()
                            .to_string_lossy()
                            .as_ref(),
                    );
                    path_str.push('/');
                }

                let last = last.as_os_str().to_string_lossy();
                let last = match last.split_once(".") {
                    Some((last, _ext)) => last,
                    None => &last,
                };

                path_str.push_str(last);

                Some(Cow::Owned(path_str))
            }
            None => None,
        };

        let prefix = prefix_to_string(
            self.protocol.as_ref().map(|this| protocol_to_secure(this)),
            self.host.as_ref(),
            self.port.as_ref().map(|this| this.as_str()),
            path,
        );

        let search_params = search_params_to_string(self.search_params.as_ref());

        match search_params {
            Some(search_params) => format!("{prefix}?{search_params}"),
            None => prefix.to_string(),
        }
    }
}

impl ToString for SanitizedUrl {
    fn to_string(&self) -> String {
        let prefix = prefix_to_string(
            self.protocol.as_ref().map(|x| x.as_str()),
            self.host.as_ref(),
            self.port.as_ref().map(|this| this.as_str()),
            self.path.as_ref().map(|x| x.to_string_lossy()),
        );

        let search_params = search_params_to_string(self.search_params.as_ref());

        match search_params {
            Some(search_params) => format!("{prefix}?{search_params}"),
            None => prefix.to_string(),
        }
    }
}

fn prefix_to_string<'a>(
    protocol: Option<&'a str>,
    host: &'a str,
    port: Option<&'a str>,
    path: Option<Cow<'a, str>>,
) -> Cow<'a, str> {
    let hostname = match port {
        Some(port) => Cow::Owned(format!("{host}:{port}")),
        None => Cow::Borrowed(host),
    };

    match (protocol, path) {
        (Some(protocol), Some(path)) => Cow::Owned(format!("{}://{}/{}", protocol, hostname, path)),
        (Some(protocol), None) => Cow::Owned(format!("{}://{}", protocol, hostname)),
        (None, Some(path)) => Cow::Owned(format!("{}/{}", hostname, path)),
        (None, None) => hostname,
    }
}

fn normalize_host(host: &'_ str) -> Cow<'_, str> {
    let suffix_list = PUBLIC_SUFFIX_LIST.blocking_read();

    match suffix_list.suffix(host.as_bytes()) {
        Some(suffix) => {
            let suffix = String::from_utf8_lossy(suffix.as_bytes());

            let prefix = host.clip(0, suffix.len());

            Cow::Owned(format!("{prefix}{suffix}"))
        }

        None => Cow::Borrowed(host),
    }
}

fn protocol_to_secure(protocol: &str) -> &str {
    NORMALIZED_PROTOCOLS
        .get(&protocol)
        .map(|this| *this)
        .unwrap_or(protocol)
}

fn search_params_to_string<'a>(search_params: Option<&Vec<(String, String)>>) -> Option<String> {
    match search_params {
        Some(search_params) => {
            let search_params = search_params
                .into_iter()
                .map(|(name, value)| format!("{name}={value}"))
                .join("&");

            if search_params.len() != 0 {
                Some(search_params)
            } else {
                None
            }
        }

        None => None,
    }
}

fn parse_search_params<'a>(
    host: &str,
    _path: Option<&PathBuf>,
    search_params: &'a str,
    filter_search_param: impl for<'b> Fn(&'b str, &'b str) -> bool + 'a,
) -> impl Iterator<Item = (&'a str, &'a str)> {
    search_params
        .split('&')
        .filter(|s| !s.is_empty())
        .filter_map(move |pair| {
            let mut parts = pair.splitn(2, '=');
            let name = parts.next().unwrap_or("");
            let value = parts.next().unwrap_or("");

            if SEARCH_PARAM_TRACKERS.get_key(name).is_some() {
                return None;
            }

            if let Some(trackers) = HOST_SCOPED_SEARCH_PARAM_TRACKERS.get(host) {
                if trackers.get_key(name).is_some() {
                    return None;
                }
            }

            if filter_search_param(name, value) {
                return None;
            }

            Some((name, value))
        })
}

static PUBLIC_SUFFIX_LIST_RAW: &str = include_str!("./public_suffix_list.dat");

/// A list of domain suffixes (tld's).
pub static PUBLIC_SUFFIX_LIST: LazyLock<RwLock<List>> = LazyLock::new(|| {
    RwLock::new(
        PUBLIC_SUFFIX_LIST_RAW
            .parse::<List>()
            .expect("failed to parse suffix list"),
    )
});

/// Fetches an up to date copy of the public suffix list.
pub async fn refresh_public_suffix_list(client: impl HttpClient + 'static) -> anyhow::Result<()> {
    let request =
        Request::get("https://publicsuffix.org/list/public_suffix_list.dat").body(vec![])?;

    let response = client.execute(request).await?;
    let bytes = response.bytes().await?;
    let data = str::from_utf8(&bytes)?;

    *PUBLIC_SUFFIX_LIST.write().await = data.parse::<List>().expect("failed to parse suffix list");

    Ok(())
}

const NORMALIZED_PROTOCOLS: phf::Map<&'static str, &'static str> = phf_map! {
    "http" => "https",
    "ws" => "wss",
    "ftp" => "ftps",
    "smtp" => "smtps",
    "imap" => "imaps",
    "pop3" => "pop3s",
    "ldap" => "ldaps",
    "irc" => "ircs",
    "nntp" => "nntps",
};

// Trackers obtained from Brave: https://github.com/brave/brave-core/blob/master/components/query_filter/utils.cc#27
const SEARCH_PARAM_TRACKERS: phf::Set<&'static str> = phf_set! {
    // https://github.com/brave/brave-browser/issues/9019
    "__hsfp",
    "__hssc",
    "__hstc",
    // https://github.com/brave/brave-browser/issues/8975
    "__s",
    // https://github.com/brave/brave-browser/issues/40716
    "_bhlid",
    // https://github.com/brave/brave-browser/issues/39575
    "_branch_match_id",
    "_branch_referrer",
    // https://github.com/brave/brave-browser/issues/33188
    "_gl",
    // https://github.com/brave/brave-browser/issues/9019
    "_hsenc",
    // https://github.com/brave/brave-browser/issues/34578
    "_kx",
    // https://github.com/brave/brave-browser/issues/11579
    "_openstat",
    // https://github.com/brave/brave-browser/issues/32488
    "at_recipient_id",
    "at_recipient_list",
    // https://github.com/brave/brave-browser/issues/37971
    "bbeml",
    // https://github.com/brave/brave-browser/issues/25238
    "bsft_clkid",
    "bsft_uid",
    // https://github.com/brave/brave-browser/issues/9879
    "dclid",
    // https://github.com/brave/brave-browser/issues/37847
    "et_rid",
    // https://github.com/brave/brave-browser/issues/33984
    "fb_action_ids",
    "fb_comment_id",
    // https://github.com/brave/brave-browser/issues/4239
    "fbclid",
    // https://github.com/brave/brave-browser/issues/4239
    "gclid",
    // https://github.com/brave/brave-browser/issues/25691
    "guce_referrer",
    "guce_referrer_sig",
    // https://github.com/brave/brave-browser/issues/9019
    "hsCtaTracking",
    // https://github.com/brave/brave-browser/issues/33952
    "irclickid",
    // https://github.com/brave/brave-browser/issues/4239
    "mc_eid",
    // https://github.com/brave/brave-browser/issues/17507
    "ml_subscriber",
    "ml_subscriber_hash",
    // https://github.com/brave/brave-browser/issues/4239
    "msclkid",
    // https://github.com/brave/brave-browser/issues/31084
    "mtm_cid",
    // https://github.com/brave/brave-browser/issues/22082
    "oft_c",
    "oft_ck",
    "oft_d",
    "oft_id",
    "oft_ids",
    "oft_k",
    "oft_lk",
    "oft_sk",
    // https://github.com/brave/brave-browser/issues/13644
    "oly_anon_id",
    "oly_enc_id",
    // https://github.com/brave/brave-browser/issues/31084
    "pk_cid",
    // https://github.com/brave/brave-browser/issues/17451
    "rb_clickid",
    // https://github.com/brave/brave-browser/issues/17452
    "s_cid",
    // https://github.com/brave/brave-browser/issues/43077
    "sc_customer",
    "sc_eh",
    "sc_uid",
    // https://github.com/brave/brave-browser/issues/48228
    "sms_click",
    "sms_source",
    "sms_uph",
    // https://github.com/brave/brave-browser/issues/40912
    "srsltid",
    // https://github.com/brave/brave-browser/issues/24988
    "ss_email_id",
    // https://github.com/brave/brave-browser/issues/48226
    "ttclid",
    // https://github.com/brave/brave-browser/issues/18020
    "twclid",
    // https://github.com/brave/brave-browser/issues/33172
    "unicorn_click_id",
    // https://github.com/brave/brave-browser/issues/11817
    "vero_conv",
    "vero_id",
    // https://github.com/brave/brave-browser/issues/26295
    "vgo_ee",
    // https://github.com/brave/brave-browser/issues/18758
    "wbraid",
    // https://github.com/brave/brave-browser/issues/13647
    "wickedid",
    // https://github.com/brave/brave-browser/issues/11578
    "yclid",
    // https://github.com/brave/brave-browser/issues/33216
    "ymclid",
    "ysclid",
};

// Change to a trie.
/*const PATH_SCOPED_SEARCH_BYPASS_PARAM_TRACKERS: phf::Map<&'static str, phf::Set<&'static str>> = phf_map! {
    // https://github.com/brave/brave-browser/issues/44341
    "/unsubscribe" => phf_set! { "ck_subscriber_id" },
    // https://github.com/brave/brave-browser/issues/30731
    "/email/" => phf_set! { "h_sid", "h_slt" },
    // https://github.com/brave/brave-browser/issues/9018
    "unsubscribe" => phf_set! { "mkt_tok" },
    "Unsubscribe" => phf_set! { "mkt_tok" },
    "emailWebview" => phf_set! { "mkt_tok" },
};*/

const HOST_SCOPED_SEARCH_PARAM_TRACKERS: phf::Map<&'static str, phf::Set<&'static str>> = phf_map! {
    "instagram.com" => phf_set! {
        // https://github.com/brave/brave-browser/issues/35094
        "igsh",
        // https://github.com/brave/brave-browser/issues/11580
        "igshid"
    },
    "twitter.com" => phf_set! {
        // https://github.com/brave/brave-browser/issues/26966
        "ref_src",
        "ref_url"
    },
    // https://github.com/brave/brave-browser/issues/34719
    "youtube.com" => phf_set! { "si" },
    "youtu.be" => phf_set! { "si" }
};
