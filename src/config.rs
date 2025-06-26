use crate::error::AppError;
use crate::error::AppError::ConfigError;
use dotenvy;
use serde::Deserialize;
use std::{collections::HashMap, fs};
use std::path::Path;
use toml::from_str;

#[derive(Debug, Deserialize, Clone)]
pub struct QueueConfig {
    pub name: String,
    #[serde(default)]
    pub alias: Option<String>,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RabbitMQConfig {
    // 基础连接配置
    pub host: String,
    pub port: u16,
    pub vhost: String,
    pub username: String,
    pub password: String,
    pub max_retries: u32,
    pub retry_delay_ms: u64,

    // 改为队列配置列表
    pub queues: Vec<QueueConfig>,

    // 全局别名映射
    #[serde(default)]
    pub queue_aliases: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DingTalkConfig {
    pub webhook_url: String,
    pub secret: String,
    pub user_id: String,
    pub custom_robot_token: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub rabbitmq: RabbitMQConfig,
    pub dingtalk: DingTalkConfig,
}

impl Config {
    pub fn load() -> Result<Self, AppError> {
        match from_str(&*fs::read_to_string(Path::new("config.toml")).unwrap()) {
            Ok(config) => Ok(config),
            Err(error) => Err(ConfigError(error.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use toml::from_str;

    #[test]
    fn test_config_compatibility() {
        let config: Config = from_str(fs::read_to_string("config.toml").unwrap().as_str()).unwrap();
        assert_eq!(config.rabbitmq.port, 5672);
        assert_eq!(config.rabbitmq.username, "star");
        
    }
}