use crate::{error::MqError, config::RabbitMQConfig};
use lapin::{Connection, ConnectionProperties};
use tracing::warn;
use tokio::time;

pub struct RabbitConnector {
    pub config: RabbitMQConfig,
}

impl RabbitConnector {
    pub fn new(config: RabbitMQConfig) -> Self {
        Self { config }
    }

    pub async fn connect(&self) -> Result<Connection, MqError> {
        let mut attempt = 0;
        let conn_string = self.connection_string();

        loop {
            attempt += 1;
            match Connection::connect(&conn_string, ConnectionProperties::default()).await {
                Ok(conn) => return Ok(conn),
                Err(err) if attempt >= self.config.max_retries => {
                    return Err(MqError::ConnectionRetryExhausted {
                        host: self.config.host.clone(),
                        port: self.config.port,
                        attempts: self.config.max_retries,
                    });
                }
                Err(err) => {
                    warn!(
                        "连接尝试 {}/{} 失败: {}",
                        attempt, self.config.max_retries, err
                    );
                    time::sleep(time::Duration::from_millis(self.config.retry_delay_ms)).await;
                }
            }
        }
    }

    fn connection_string(&self) -> String {
        format!(
            "amqp://{}:{}@{}:{}/{}",
            self.config.username,
            self.config.password,
            self.config.host,
            self.config.port,
            self.config.vhost
        )
    }
}