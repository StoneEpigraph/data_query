mod config;
mod core;
mod error;
mod rabbitmq;
mod util;

use crate::config::Config;
use crate::rabbitmq::monitor;
use anyhow::{Context, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 加载配置 - 使用 anyhow 添加顶层上下文
    let config = Config::load().context("配置加载失败")?;

    monitor::monitor_rabbitmq(config)
        .await?
        .expect("monitor rabbitmq error");

    Ok(())
}
