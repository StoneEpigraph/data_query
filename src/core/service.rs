use crate::{
    config::{RabbitMQConfig, QueueConfig},
    error::MqError,
    rabbitmq::{conn::RabbitConnector, queue::QueueInspector},
};
use anyhow::Context;
use tracing::{info, warn, error};

pub struct QueueMetricsService {
    connector: RabbitConnector,
}

pub struct QueueMetric {
    pub name: String,
    pub alias: Option<String>,
    pub count: u32,
}

impl QueueMetricsService {
    pub fn new(config: RabbitMQConfig) -> Self {
        Self {
            connector: RabbitConnector::new(config),
        }
    }

    pub async fn gather_metrics(&self) -> anyhow::Result<Vec<QueueMetric>> {
        // 使用 anyhow 添加上下文
        let conn = match self.connector.connect().await {
            Ok(conn) => conn,
            Err(e) => return Err(e).context("连接 RabbitMQ 失败"),
        };
            

        info!("✅ RabbitMQ 连接成功");

        let channel = conn.create_channel()
            .await
            .context("创建通道失败")?;

        info!("🚀 通道创建成功");

        let inspector = QueueInspector::new(&channel);
        let mut results = Vec::new();
        let mut errors = 0;

        for queue in &self.connector.config.queues {
            match self.get_queue_metric(&inspector, queue).await {
                Ok(metric) => results.push(metric),
                Err(e) => {
                    errors += 1;
                    self.handle_queue_error(e, queue);
                }
            }
        }

        info!("📊 完成 {}/{} 个队列查询", results.len(), results.len() + errors);
        Ok(results)
    }

    async fn get_queue_metric(
        &self,
        inspector: &QueueInspector<'_>,
        queue: &QueueConfig,
    ) -> Result<QueueMetric, MqError> {
        let count = inspector.get_message_count(&queue.name, queue.timeout_ms).await?;

        Ok(QueueMetric {
            name: queue.name.clone(),
            alias: queue.alias.clone(),
            count,
        })
    }

    fn handle_queue_error(&self, error: MqError, queue: &QueueConfig) {
        let alias = queue.alias.as_deref()
            .or_else(|| self.connector.config.queue_aliases.get(&queue.name).map(String::as_str))
            .unwrap_or(&queue.name);

        match error {
            MqError::QueueNotFound(_) => {
                warn!("⚠️ 队列不存在: {} [{}]", alias, queue.name)
            }
            MqError::QueueTimeout(_) => {
                warn!("⏱️ 队列查询超时: {} [{}]", alias, queue.name)
            }
            _ => {
                error!("❌ 队列错误: {} [{}] - {}", alias, queue.name, error)
            }
        }
    }
}