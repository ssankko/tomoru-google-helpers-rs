mod google;
pub(super) use google::*;

use once_cell::sync::OnceCell;
use std::{collections::HashMap, option::Option, string::String, time::Duration};
use tokio::sync::Mutex;

static CURRENT_RESOURCE: OnceCell<google::MonitoredResource> = OnceCell::new();
static PROJECT_ID: OnceCell<&'static str> = OnceCell::new();
static LOG_NAME: OnceCell<&'static str> = OnceCell::new();

pub fn initialize_logger(project_id: &'static str, log_name: &'static str) {
    PROJECT_ID.set(project_id).unwrap();
    LOG_NAME.set(log_name).unwrap();
}

/// https://cloud.google.com/monitoring/api/resources
pub fn describe_current_resource(
    r#type: String,
    instance_id: String,
    project_id: String,
    zone: String,
) {
    let mut labels = HashMap::new();
    labels.insert("instance_id".to_owned(), instance_id);
    labels.insert("project_id".to_owned(), project_id);
    labels.insert("zone".to_owned(), zone);
    CURRENT_RESOURCE
        .set(google::MonitoredResource { r#type, labels })
        .unwrap();
}

lazy_static::lazy_static! {
    static ref LOGGER_QUEUE: Mutex<Vec<LogEntry>> = {
        tokio::spawn(async {
            loop{
                tokio::time::delay_for(Duration::from_secs(5)).await;
                let queue = LOGGER_QUEUE.lock().await.drain(..).collect();
                let res = google::write_log(google::Log{
                    project_id: PROJECT_ID.get().unwrap(),
                    log_name: LOG_NAME.get().unwrap(),
                    resource: CURRENT_RESOURCE.get().cloned(),
                    labels: Default::default(),
                    entries: queue,
                }).await;
                if let Err(e) = res {
                    eprintln!("[GOOGLE LOGGER] Failed to write log: {}", e);
                }
            }
        });
        Mutex::new(Vec::with_capacity(32))
    };
}

pub struct HttpRequest {
    /// The request method. Examples: `"GET"`, `"HEAD"`, `"PUT"`, `"POST"`.
    pub request_method: String,
    /// The scheme (http, https), the host name, the path and the query
    /// portion of the URL that was requested.
    /// Example: `"http://example.com/some/info?color=red"`.
    pub request_url: String,
    /// The size of the HTTP request message in bytes, including the request
    /// headers and the request body.
    pub request_size: i64,
    /// The user agent sent by the client. Example:
    /// `"Mozilla/4.0 (compatible; MSIE 6.0; Windows 98; Q312461; .NET
    /// CLR 1.0.3705)"`.
    pub user_agent: String,
    /// The IP address (IPv4 or IPv6) of the client that issued the HTTP
    /// request. Examples: `"192.168.1.1"`, `"FE80::0202:B3FF:FE1E:8329"`.
    pub remote_ip: Option<String>,
    /// The referer URL of the request, as defined in
    /// [HTTP/1.1 Header Field
    /// Definitions](http://www.w3.org/Protocols/rfc2616/rfc2616-sec14.html).
    pub referer: String,
    /// The request processing latency on the server, from the time the request was
    /// received until the response was sent.
    pub latency: Option<Duration>,
    /// Protocol used for the request. Examples: "HTTP/1.1", "HTTP/2", "websocket"
    pub protocol: String,
}

impl Into<google::HttpRequest> for HttpRequest {
    fn into(self) -> google::HttpRequest {
        google::HttpRequest {
            request_method: self.request_method,
            request_url: self.request_url,
            request_size: self.request_size,
            user_agent: self.user_agent,
            remote_ip: self.remote_ip.unwrap_or_default(),
            referer: self.referer,
            latency: self.latency.map(|x| x.into()),
            protocol: self.protocol,
            ..Default::default()
        }
    }
}

#[cfg(feature = "logging-hyper-requests")]
impl From<hyper::Request<hyper::Body>> for HttpRequest {
    fn from(item: hyper::Request<hyper::Body>) -> Self {
        use futures::stream::Stream;
        HttpRequest {
            request_method: item.method().to_string(),
            request_url: item.uri().to_string(),
            request_size: item.body().size_hint().0 as i64,
            user_agent: item
                .headers()
                .get("User-Agent")
                .map(|x| x.to_str().unwrap_or("").to_string())
                .unwrap_or_default(),
            remote_ip: None,
            referer: item
                .headers()
                .get("Referer")
                .map(|x| x.to_str().unwrap_or("").to_string())
                .unwrap_or_default(),
            latency: None,
            protocol: format!("{:?}", item.version()),
        }
    }
}

#[cfg(feature = "logging-hyper-requests")]
impl From<&hyper::Request<hyper::Body>> for HttpRequest {
    fn from(item: &hyper::Request<hyper::Body>) -> Self {
        use futures::stream::Stream;
        HttpRequest {
            request_method: item.method().to_string(),
            request_url: item.uri().to_string(),
            request_size: item.body().size_hint().0 as i64,
            user_agent: item
                .headers()
                .get("User-Agent")
                .map(|x| x.to_str().unwrap_or("").to_string())
                .unwrap_or_default(),
            remote_ip: None,
            referer: item
                .headers()
                .get("Referer")
                .map(|x| x.to_str().unwrap_or("").to_string())
                .unwrap_or_default(),
            latency: None,
            protocol: format!("{:?}", item.version()),
        }
    }
}

impl HttpRequest {
    pub fn with_latency(mut self, latency: Duration) -> Self {
        self.latency = Some(latency);
        self
    }

    pub fn with_ip(mut self, ip: impl Into<String>) -> Self {
        self.remote_ip = Some(ip.into());
        self
    }
}

#[derive(Default, Clone)]
pub struct LogContext {
    request: Option<google::HttpRequest>,
    operation: Option<String>,
    labels: std::collections::HashMap<String, String>,
}

impl LogContext {
    pub fn new() -> LogContext {
        Default::default()
    }

    pub fn label(mut self, label: impl Into<String>, value: impl Into<String>) -> LogContext {
        self.labels.insert(label.into(), value.into());
        self
    }

    // pub fn operation(mut self, operation_id: impl Into<String>) -> LogContext {
    //     self.operation = Some(operation_id.into());
    //     self
    // }

    pub fn request(mut self, request: impl Into<google::HttpRequest>) -> LogContext {
        self.request = Some(request.into());
        self
    }

    pub fn with(mut self, context: LogContext) -> LogContext {
        if let Some(request) = context.request {
            self.request = Some(request)
        }
        if let Some(operation) = context.operation {
            self.operation = Some(operation)
        }
        for (label, value) in context.labels {
            let _ = self.labels.insert(label, value);
        }
        self
    }
}

impl Into<LogContext> for &LogContext {
    fn into(self) -> LogContext {
        self.clone()
    }
}

pub struct LogBuilder {
    payload: Option<Payload>,
    context: LogContext,
    time: google::Timestamp,
    severity: LogSeverity,
    file: &'static str,
    fn_name: &'static str,
    line: i64,
    operation_first: bool,
    operation_last: bool,
}

impl LogBuilder {
    pub fn new(
        severity: LogSeverity,
        line: i64,
        file: &'static str,
        fn_name: &'static str,
    ) -> LogBuilder {
        LogBuilder {
            payload: None,
            context: Default::default(),
            time: google::Timestamp::now(),
            severity,
            file,
            line,
            fn_name,
            operation_first: false,
            operation_last: false,
        }
    }

    pub fn context(mut self, log_context: impl Into<LogContext>) -> LogBuilder {
        self.context = log_context.into();
        self
    }

    pub fn label(mut self, label: impl Into<String>, value: impl Into<String>) -> LogBuilder {
        self.context.labels.insert(label.into(), value.into());
        self
    }

    // pub fn operation(mut self, operation_id: impl Into<String>) -> LogBuilder {
    //     self.context.operation = Some(operation_id.into());
    //     self
    // }

    // pub fn first(mut self) -> LogBuilder {
    //     self.operation_first = true;
    //     self
    // }

    pub fn request(mut self, request: impl Into<google::HttpRequest>) -> LogBuilder {
        self.context.request = Some(request.into());
        self
    }

    // pub fn last(mut self) -> LogBuilder {
    //     self.operation_last = true;
    //     self
    // }

    pub fn send_text(mut self, text: impl Into<String>) {
        self.payload = Some(Payload::Text(text.into()));
        self.build_and_push();
    }

    pub fn send_json(mut self, json: impl serde::Serialize) {
        let json = serde_json::to_value(json).unwrap();
        let payload = match json {
            serde_json::Value::Object(map) => Payload::Json(map),
            val => Payload::Text(val.to_string()),
        };

        self.payload = Some(payload);
        self.build_and_push();
    }

    fn build_and_push(self) {
        let enabled_google_logging = option_env!("GOOGLE_LOGGING_ENABLED");
        if let Some("true") = enabled_google_logging {
            let entry = LogEntry {
                timestamp: self.time,
                severity: self.severity,
                labels: self.context.labels,
                source_code_entry: google::LogEntrySourceLocation {
                    file: self.file.to_owned(),
                    line: self.line,
                    function: self.fn_name.to_owned(),
                },
                payload: self.payload.unwrap(),
                http_request: self.context.request,
                operation: if let Some(operation) = self.context.operation {
                    Some(google::LogEntryOperation {
                        id: operation,
                        producer: String::new(),
                        first: self.operation_first,
                        last: self.operation_last,
                    })
                } else {
                    None
                },
            };

            match LOGGER_QUEUE.try_lock() {
                Ok(mut guard) => {
                    guard.push(entry);
                }
                Err(_) => {
                    tokio::spawn(async {
                        LOGGER_QUEUE.lock().await.push(entry);
                    });
                }
            }
        } else {
            let text = match self.payload.unwrap() {
                Payload::Text(s) => s,
                Payload::Json(map) => format!("{:?}", map),
            };
            let labels = self
                .context
                .labels
                .into_iter()
                .map(|x| format!("[{} = {}]", x.0, x.1))
                .collect::<Vec<String>>()
                .join(" ");
            let time = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, false);
            println!(
                "{:<29} | {}\n[{}]{} -> {}\n",
                format!("{}:{}", self.file, self.line),
                self.fn_name,
                time,
                labels,
                text.replace("\n", "\n>   ")
            );
        }
    }
}

#[allow(unused_macros)]
macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}

#[allow(unused_macros)]
macro_rules! log {
    ($sev: tt) => {{
        let line = line!() as i64;
        let file = file!();
        let fn_name = function!();
        crate::logger::LogBuilder::new(google::LogSeverity::$sev, line, file, fn_name)
    }};
}

#[macro_export]
macro_rules! log_debug {
    () => {
        log!(Debug)
    };
}

#[macro_export]
macro_rules! log_info {
    () => {
        log!(Info)
    };
}

#[macro_export]
macro_rules! log_warn {
    () => {
        log!(Warning)
    };
}

#[macro_export]
macro_rules! log_error {
    () => {
        log!(Error)
    };
}

#[macro_export]
macro_rules! log_critical {
    () => {
        log!(Critical)
    };
}
