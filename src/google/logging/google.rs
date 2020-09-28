crate::rpc_service!("logging", "https://www.googleapis.com/auth/cloud-platform");
use crate::google::generated::google::logging::v2;
use std::collections::HashMap;

pub use crate::google::generated::google::{
    api::MonitoredResource,
    logging::{
        r#type::HttpRequest,
        v2::{LogEntryOperation, LogEntrySourceLocation},
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

pub enum Payload {
    Text(String),
    Json(serde_json::Map<String, serde_json::Value>),
}

fn serde_map_to_proto_fields(
    map: serde_json::Map<String, serde_json::Value>,
) -> prost_types::Struct {
    prost_types::Struct {
        fields: map
            .into_iter()
            .map(|x| {
                let val = serde_json_value_to_proto_value(x.1);
                (x.0, val)
            })
            .collect(),
    }
}

fn serde_json_value_to_proto_value(value: serde_json::Value) -> prost_types::Value {
    prost_types::Value {
        kind: Some(match value {
            serde_json::Value::Null => prost_types::value::Kind::NullValue(0),
            serde_json::Value::Bool(v) => prost_types::value::Kind::BoolValue(v),
            serde_json::Value::Number(v) => {
                prost_types::value::Kind::NumberValue(v.as_f64().unwrap())
            }
            serde_json::Value::String(v) => prost_types::value::Kind::StringValue(v),
            serde_json::Value::Array(v) => {
                prost_types::value::Kind::ListValue(prost_types::ListValue {
                    values: v.into_iter().map(serde_json_value_to_proto_value).collect(),
                })
            }
            serde_json::Value::Object(v) => {
                prost_types::value::Kind::StructValue(serde_map_to_proto_fields(v))
            }
        }),
    }
}

impl Into<v2::log_entry::Payload> for Payload {
    fn into(self) -> v2::log_entry::Payload {
        match self {
            Payload::Text(text) => v2::log_entry::Payload::TextPayload(text),
            Payload::Json(map) => {
                v2::log_entry::Payload::JsonPayload(serde_map_to_proto_fields(map))
            }
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
            payload: Some(self.payload.into()),
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
