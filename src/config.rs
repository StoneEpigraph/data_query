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

    pub warning: RabbitMQWarningConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RabbitMQWarningConfig {
    pub enabled: bool,
    pub warning_queue_size: u32,
    #[serde(flatten)]
    pub warning_type: Option<WarningType>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DingTalkConfig {
    pub secret: String,
    pub user_id: String,
    pub custom_robot_token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub rabbitmq: RabbitMQConfig,
    pub dingtalk: DingTalkConfig,
    pub warning: WarningConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WarningConfig {
    pub warning_valid: bool,
    #[serde(flatten)]
    pub warning_type: Option<WarningType>,
    pub warning_time_interval: u16,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(tag = "warning_type", content = "params")]
pub enum WarningType {
    DingTalk,
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

    #[test]
    fn test_enum_warning_type() {
        //多行配置文件文本

        let config_str = r#"
        [rabbitmq]
        # RabbitMQ 配置
        host = "127.0.0.1"
        port = 5672
        vhost = "/"
        username = "star"
        password = "star"
        max_retries = 3
        retry_delay_ms = 1000
        [[rabbitmq.queues]]
        name = "test_queue"
        [rabbitmq.warning]
        enabled = true
        warning_time_interval = 10
        warning_queue_size = 1000
        [dingtalk]
        # 钉钉机器人配置
        webhook_url = "https://oapi.dingtalk.com/robot/send?access_token=xxxxxx"
        secret = "xxxxxx"
        user_id = "123456"
        custom_robot_token = "xxxxxx"

        [warning]
        # 告警配置
        warning_valid = false  # 是否启用告警功能
        warning_type = "DingTalk"
        warning_time_interval = 10  # 告警时间间隔，单位：分钟
        "#;
        let config: Config = from_str(config_str).expect("Failed to parse config");
        assert_eq!(config.warning.warning_valid, false);
        assert_eq!(config.warning.warning_type, Some(WarningType::DingTalk));
    }
}