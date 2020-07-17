crate::service!(
    "cloudtasks",
    "https://www.googleapis.com/auth/cloud-platform"
);
use super::generated::google::cloud::tasks::v2beta3::*;
use std::collections::HashMap;

pub struct QueueSettings<'a> {
    pub project_id: &'a str,
    pub location: &'a str,
    pub queue_name: &'a str,
}

impl<'a> QueueSettings<'a> {
    fn form_queue(&self) -> String {
        format!(
            "projects/{}/locations/{}/queues/{}",
            self.project_id, self.location, self.queue_name
        )
    }
}

pub struct TaskData<'a> {
    pub url: String,
    pub body: Vec<u8>,
    pub queue: QueueSettings<'a>,
}

pub async fn create_task<'a>(task: TaskData<'a>) -> Result<(), tonic::Status> {
    let tasks = SERVICE.get().unwrap();

    let queue = task.queue.form_queue();

    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_owned(), "application/json".to_owned());

    let request = CreateTaskRequest {
        parent: queue.clone(),
        task: Some(Task {
            payload_type: Some(task::PayloadType::HttpRequest(HttpRequest {
                url: task.url,
                body: task.body,
                headers,
                ..HttpRequest::default()
            })),
            ..Task::default()
        }),
        ..CreateTaskRequest::default()
    };

    let channel = tasks.channel.clone();

    let token = tasks.auth.token(SCOPES).await.unwrap();
    let bearer_token = format!("Bearer {}", token.as_str());
    let token = MetadataValue::from_str(&bearer_token).unwrap();

    let mut service = cloud_tasks_client::CloudTasksClient::with_interceptor(
        channel,
        move |mut req: Request<()>| {
            let token = token.clone();
            req.metadata_mut().insert("authorization", token);
            req.metadata_mut().insert(
                "x-goog-request-params",
                MetadataValue::from_str(&format!("parent={}", queue)).unwrap(),
            );
            Ok(req)
        },
    );

    let response = service.create_task(request).await;

    response.map(|_| ())
}
