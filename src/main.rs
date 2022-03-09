use chrono::{NaiveDateTime};
use scienceobjectsdb_rust_api::sciobjectsdb::sciobjsdb::api::notification::services::v1::{CreateEventStreamingGroupRequest, NotficationStreamAck, NotificationStreamGroupRequest, NotificationStreamInit, StreamAll};
use scienceobjectsdb_rust_api::sciobjectsdb::sciobjsdb::api::notification::services::v1::create_event_streaming_group_request::{EventResources, StreamType};
use scienceobjectsdb_rust_api::sciobjectsdb::sciobjsdb::api::notification::services::v1::notification_stream_group_request::StreamAction;
use scienceobjectsdb_rust_api::sciobjectsdb::sciobjsdb::api::notification::services::v1::update_notification_service_client::UpdateNotificationServiceClient;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Status};
use tonic::metadata::{AsciiMetadataKey, AsciiMetadataValue};
use tonic::service::Interceptor;
use tonic::transport::Endpoint;

use futures::stream::StreamExt;
use scienceobjectsdb_rust_api::sciobjectsdb::sciobjsdb::api::notification::services::v1::event_notification_message::UpdateType;

const ENDPOINT: &str = "https://api.scienceobjectsdb.nfdi-dev.gi.denbi.de/swagger-ui/";
// Insert your project id here
const PROJECT_ID: &str = "TODO";
// Inser your API token here
const API_TOKEN: &str = "TODO";

/// The interceptor appends the `API_TOKEN` to each request
#[derive(Clone, Debug)]
struct APITokenInterceptor {
    key: AsciiMetadataKey,
    token: AsciiMetadataValue,
}

impl APITokenInterceptor {
    fn new(token: &'static str) -> APITokenInterceptor {
        let key = AsciiMetadataKey::from_static("api_token");
        let value = AsciiMetadataValue::from_static(token);
        APITokenInterceptor { key, token: value }
    }
}

impl Interceptor for APITokenInterceptor {
    // Append the API token to the given request
    fn call(&mut self, mut request: Request<()>) -> std::result::Result<Request<()>, Status> {
        request
            .metadata_mut()
            .append(self.key.clone(), self.token.clone());
        Ok(request)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a channel
    let channel = Endpoint::from_static(ENDPOINT).connect().await?;

    // Create an interceptor instance
    let interceptor = APITokenInterceptor::new(API_TOKEN);

    // Create the client
    let mut notification_client =
        UpdateNotificationServiceClient::with_interceptor(channel, interceptor);

    // Create a group/subscription
    let group_id = notification_client
        .create_event_streaming_group(CreateEventStreamingGroupRequest {
            // We subscribe to a project
            resource: EventResources::ProjectResource.into(),
            // The project id
            resource_id: PROJECT_ID.to_string(),
            // We want notifications of all sub resources
            include_subresource: true,
            // We want all messages
            stream_type: Some(StreamType::StreamAll(StreamAll {})),
            ..Default::default()
        })
        .await?
        .into_inner()
        .stream_group_id;

    // We need a channel to send the initial request and acknowledgements to the server
    let (tx, rx) = tokio::sync::mpsc::channel(8);

    // Initialize the server connection
    let mut notification_stream = notification_client
        .notification_stream_group(ReceiverStream::new(rx))
        .await?
        .into_inner();

    // Send the initial request
    tx.send(NotificationStreamGroupRequest {
        // Do not close the notification stream
        close: false,
        // Submit the group id
        stream_action: Some(StreamAction::Init(NotificationStreamInit {
            stream_group_id: group_id,
        })),
    })
    .await?;

    // Wait and process the notifications until the stream ends or an error is received
    while let Some(Ok(batch)) = notification_stream.next().await {
        println!(
            "Received chunk of {} notifications.",
            batch.notification.len()
        );

        // Loop the notification of the current batch
        for notification in batch.notification {
            let message = notification.message.unwrap();

            // Every message contains a sequence number and a timestamp
            // in order to identify outdated messages.
            // Moreover it contains the type and id of the affected resource
            // and the change type.
            println!("Received notification: {} at {:?} for {} ({}): {}",
                notification.sequence,
                notification.timestamp.map(|ts| NaiveDateTime::from_timestamp(ts.seconds, ts.nanos as u32)),
                match message.resource {
                    x if x == EventResources::Unspecified as i32 => "Unspecified",
                    x if x == EventResources::AllResource as i32 => "All",
                    x if x == EventResources::ProjectResource as i32 => "Project",
                    x if x == EventResources::DatasetResource as i32 => "Dataset",
                    x if x == EventResources::DatasetVersionResource as i32 => "DatasetVersion",
                    x if x == EventResources::ObjectGroupResource as i32 => "ObjectGroup",
                    _ => panic!("Unknown resource type")
                },
                message.resource_id,
                match message.updated_type {
                    x if x == UpdateType::Unspecified as i32 => "Unspecified",
                    x if x == UpdateType::Updated as i32 => "Updated",
                    x if x == UpdateType::MetadataUpdated as i32 => "MetadataUpdated",
                    x if x == UpdateType::Created as i32 => "Created",
                    x if x == UpdateType::Deleted as i32 => "Deleted",
                    x if x == UpdateType::Available as i32 => "Available",
                    _ => panic!("Unknown update type")
                }
            )
        }

        // Acknowledge the batch
        tx.send(NotificationStreamGroupRequest {
            // Do not close the notification stream
            close: false,
            // Submit the ack id
            stream_action: Some(StreamAction::Ack(NotficationStreamAck {
                ack_chunk_id: vec![batch.ack_chunk_id],
            })),
        })
        .await?;
    }
    Ok(())
}
