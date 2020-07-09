crate::service!("logging", "https://www.googleapis.com/auth/cloud-platform");
use crate::generated::google::logging::v2;
use std::collections::HashMap;

pub use crate::generated::google::{
    api::MonitoredResource,
    logging::{
        r#type::HttpRequest,
        v2::{log_entry::Payload, LogEntryOperation, LogEntrySourceLocation},
    },
};

pub struct Log {
    pub project_id: &'static str,
    pub log_name: &'static str,
    pub resource: Option<MonitoredResource>,
    pub labels: std::collections::HashMap<String, String>,
    pub entries: Vec<LogEntry>,
}

pub struct Timestamp {
    seconds: i64,
    nanos: i32,
}

impl Into<prost_types::Timestamp> for Timestamp {
    fn into(self) -> prost_types::Timestamp {
        prost_types::Timestamp {
            seconds: self.seconds,
            nanos: self.nanos,
        }
    }
}

impl Timestamp {
    pub fn now() -> Timestamp {
        let now = std::time::SystemTime::now();
        let duration =
            prost_types::Duration::from(now.duration_since(std::time::UNIX_EPOCH).unwrap());
        Timestamp {
            seconds: duration.seconds,
            nanos: duration.nanos,
        }
    }
}

#[repr(i32)]
pub enum LogSeverity {
    /// The log entry has no assigned severity level.
    Default = 0,
    /// Debug or trace information.
    Debug = 100,
    /// Routine information, such as ongoing status or performance.
    Info = 200,
    /// Normal but significant events, such as start up, shut down, or
    /// a configuration change.
    Notice = 300,
    /// Warning events might cause problems.
    Warning = 400,
    /// Error events are likely to cause problems.
    Error = 500,
    /// Critical events cause more severe problems or outages.
    Critical = 600,
    /// A person must take an action immediately.
    Alert = 700,
    /// One or more systems are unusable.
    Emergency = 800,
}

pub struct LogEntry {
    pub timestamp: Timestamp,
    pub severity: LogSeverity,
    pub labels: HashMap<String, String>,
    pub source_code_entry: LogEntrySourceLocation,
    pub payload: Payload,
    pub operation: Option<LogEntryOperation>,
    pub http_request: Option<HttpRequest>,
}

impl Into<v2::LogEntry> for LogEntry {
    fn into(self) -> v2::LogEntry {
        v2::LogEntry {
            timestamp: Some(self.timestamp.into()),
            severity: self.severity as i32,
            http_request: self.http_request,
            labels: self.labels,
            operation: self.operation,
            source_location: Some(self.source_code_entry),
            payload: Some(self.payload),
            ..Default::default()
        }
    }
}

pub async fn write_log(log: Log) -> Result<(), tonic::Status> {
    let logger = SERVICE.get().unwrap();
    let request = v2::WriteLogEntriesRequest {
        log_name: format!("projects/{}/logs/{}", log.project_id, log.log_name),
        resource: log.resource,
        labels: log.labels,
        entries: log.entries.into_iter().map(|x| x.into()).collect(),
        partial_success: true,
        dry_run: false,
    };

    let channel = logger.channel.clone();
    let token = logger.auth.token(SCOPES).await.unwrap();
    let bearer_token = format!("Bearer {}", token.as_str());
    let token = MetadataValue::from_str(&bearer_token).unwrap();

    let mut service = v2::logging_service_v2_client::LoggingServiceV2Client::with_interceptor(
        channel,
        move |mut req: Request<()>| {
            let token = token.clone();
            req.metadata_mut().insert("authorization", token);
            Ok(req)
        },
    );

    let response = service.write_log_entries(request).await;
    response.map(|_| ())
}
