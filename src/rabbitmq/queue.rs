use crate::{error::{MqError, is_queue_not_found}};
use lapin::{Channel, options::QueueDeclareOptions, types::FieldTable};
use tokio::time::{timeout, Duration};

pub struct QueueInspector<'a> {
    channel: &'a Channel,
}

impl<'a> QueueInspector<'a> {
    pub fn new(channel: &'a Channel) -> Self {
        Self { channel }
    }

    pub async fn get_message_count(
        &self,
        queue_name: &str,
        timeout_ms: Option<u64>,
    ) -> Result<u32, MqError> {
        let result = match timeout_ms {
            Some(ms) => timeout(
                Duration::from_millis(ms),
                self.query_queue(queue_name)
            ).await,
            None => Ok(self.query_queue(queue_name).await),
        };

        match result {
            Ok(Ok(count)) => Ok(count),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(MqError::QueueTimeout(queue_name.to_string())),
        }
    }

    async fn query_queue(&self, queue_name: &str) -> Result<u32, MqError> {
        let options = QueueDeclareOptions {
            passive: true,
            ..Default::default()
        };

        match self.channel
            .queue_declare(queue_name, options, FieldTable::default())
            .await
        {
            Ok(queue) => Ok(queue.message_count()),
            Err(e) if is_queue_not_found(&e.kind()) => {
                Err(MqError::QueueNotFound(queue_name.to_string()))
            }
            Err(e) => Err(MqError::QueueQueryError {
                queue: queue_name.to_string(),
                source: e,
            }),
        }
    }
}