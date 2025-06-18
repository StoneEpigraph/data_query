use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf};
use dotenvy;

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

#[derive(Debug, Deserialize)]
pub struct Config {
    pub rabbitmq: RabbitMQConfig,
}

impl Config {
    pub fn load() -> Result<Self, config::ConfigError> {
        dotenvy::dotenv().ok();

        let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("config.toml");

        let env_prefix = "MQAPP";

        let mut config = config::Config::builder()
            .add_source(config::File::from(config_path))
            .add_source(
                config::Environment::with_prefix(env_prefix)
                    .separator("__")
                    .list_separator(",")
                    .try_parsing(true),
            )
            .build()?;

        // 特殊处理队列列表的解析
        if let Ok(queue_names) = config.get_array("rabbitmq.queues") {
            // 如果是字符串列表，转换为队列配置对象
            let queue_configs: Vec<QueueConfig> = queue_names
                .into_iter()
                .filter_map(|v| {
                    if let Ok(name) = v.clone().into_string() {
                        Some(QueueConfig {
                            name,
                            alias: None,
                            timeout_ms: None,
                        })
                    } else {
                        None
                    }
                })
                .collect();

            if !queue_configs.is_empty() {
                config.set("rabbitmq.queues", queue_configs)?;
            }
        }

        config.try_deserialize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::{Config as RawConfig, Environment, File};
    use std::collections::HashMap;
    use std::fs;
    use toml::from_str;

    #[test]
    fn test_config_compatibility() {
        // 测试字符串列表配置
        let toml_str = r#"
            [rabbitmq]
            queues = ["queue1", "queue2"]
        "#;

        let mut config = RawConfig::builder();
        let config_context = fs::read_to_string("config.toml")?;
        config.add_source(config_context.try_into().unwrap()).build()?;
        
        dbg!(&config);

    }
}