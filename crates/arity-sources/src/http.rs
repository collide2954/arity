use std::collections::HashMap;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

/// The largest response that arity reads.
const MAX_BODY: u64 = 512 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub status: u16,
    pub content_type: Option<String>,
    pub body: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error("the request to {url} failed: {message}")]
    Request { url: String, message: String },
    #[error("{url} gave the status {status}")]
    Status { url: String, status: u16 },
    #[error("the response from {url} is not valid: {message}")]
    Invalid { url: String, message: String },
}

/// A way to read a URL. The tests use a fake that gives stored responses.
pub trait Http {
    fn get(&self, url: &str) -> Result<Response, SourceError>;

    /// Reads a URL and gives an error for each status other than 200.
    fn get_ok(&self, url: &str) -> Result<Response, SourceError> {
        let response = self.get(url)?;
        if response.status == 200 {
            Ok(response)
        } else {
            Err(SourceError::Status {
                url: url.to_owned(),
                status: response.status,
            })
        }
    }
}

/// The time to wait before the next request to a host.
pub(crate) fn wait_time(last: Option<Instant>, now: Instant, interval: Duration) -> Duration {
    last.map_or(Duration::ZERO, |last| {
        interval.saturating_sub(now.saturating_duration_since(last))
    })
}

/// An HTTP client that waits between requests to the same host.
#[derive(Debug)]
pub struct Client {
    agent: ureq::Agent,
    interval: Duration,
    last: Mutex<HashMap<String, Instant>>,
}

impl Client {
    pub fn new(user_agent: &str, interval: Duration) -> Self {
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(120)))
            .http_status_as_error(false)
            .user_agent(user_agent)
            .build()
            .into();
        Self {
            agent,
            interval,
            last: Mutex::new(HashMap::new()),
        }
    }

    fn pause(&self, url: &str) {
        let host = url::Url::parse(url)
            .ok()
            .and_then(|parsed| parsed.host_str().map(str::to_owned))
            .unwrap_or_default();
        let Ok(mut last) = self.last.lock() else {
            return;
        };
        let wait = wait_time(last.get(&host).copied(), Instant::now(), self.interval);
        if !wait.is_zero() {
            thread::sleep(wait);
        }
        last.insert(host, Instant::now());
    }
}

impl Http for Client {
    fn get(&self, url: &str) -> Result<Response, SourceError> {
        self.pause(url);
        let failure = |error: ureq::Error| SourceError::Request {
            url: url.to_owned(),
            message: error.to_string(),
        };
        let mut response = self.agent.get(url).call().map_err(failure)?;
        let status = response.status().as_u16();
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let body = response
            .body_mut()
            .with_config()
            .limit(MAX_BODY)
            .read_to_vec()
            .map_err(failure)?;
        Ok(Response {
            status,
            content_type,
            body,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waits_only_for_the_rest_of_the_interval() {
        let start = Instant::now();
        let interval = Duration::from_millis(200);
        assert_eq!(wait_time(None, start, interval), Duration::ZERO);
        let later = start + Duration::from_millis(50);
        assert_eq!(
            wait_time(Some(start), later, interval),
            Duration::from_millis(150)
        );
        let much_later = start + Duration::from_secs(1);
        assert_eq!(wait_time(Some(start), much_later, interval), Duration::ZERO);
    }
}
