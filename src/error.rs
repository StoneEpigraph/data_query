use thiserror::Error;
use lapin;

#[derive(Debug, Error)]
pub enum MqError {
    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("RabbitMQ 连接错误")]
    ConnectionError(#[source] lapin::Error),

    #[error("无法连接到 '{host}:{port}'，重试 {attempts} 次后失败")]
    ConnectionRetryExhausted {
        host: String,
        port: u16,
        attempts: u32,
    },

    #[error("队列查询错误: {queue}")]
    QueueQueryError {
        queue: String,
        #[source] source: lapin::Error,
    },

    #[error("队列不存在: {0}")]
    QueueNotFound(String),

    #[error("队列操作超时: {0}")]
    QueueTimeout(String),

    #[error("无效队列名: '{0}'")]
    InvalidQueueName(String),

    #[error("队列配置错误: {0}")]
    QueueConfigError(String),
}

/// 检查是否为队列不存在错误
pub fn is_queue_not_found(error: &lapin::ErrorKind) -> bool {
    match error {
        lapin::ErrorKind::InvalidChannel(_code) => true,
        _ => false,
    }
}