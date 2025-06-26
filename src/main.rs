mod error;
mod config;
mod rabbitmq;
mod core;
mod util;

use std::fmt::Write;
use anyhow::{Context, Result};
use tracing::{error, info};
use crate::{
    config::Config,
    core::service::QueueMetricsService,
};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 加载配置 - 使用 anyhow 添加顶层上下文
    let config = Config::load()
        .context("配置加载失败")?;

    // 创建服务
    let service = QueueMetricsService::new(config.rabbitmq);

    // 执行指标收集
    match service.gather_metrics().await {
        Ok(metrics) => {
            print_metrics(&metrics)?;
            info!("✅ 监控任务完成");
            Ok(())
        }
        Err(e) => {
            // 获取错误链的所有信息
            let mut full_error = String::new();
            write!(&mut full_error, "{:#}", e)?;

            for cause in e.chain() {
                write!(&mut full_error, "\nCaused by: {}", cause)?;
            }

            error!("监控任务失败: {}", full_error);
            Err(anyhow::anyhow!(full_error))
        }
    }
}

/// 打印监控结果表格
fn print_metrics(metrics: &[crate::core::service::QueueMetric]) -> Result<(), std::fmt::Error> {
    // 计算最大列宽
    let mut max_name_width = "队列".len();
    for metric in metrics {
        let name = metric.name.as_str();
        max_name_width = max_name_width.max(name.len());
    }

    let header_sep = "─".repeat(max_name_width + 14);

    println!("┌{}┬──────────────┐", header_sep);
    println!("│ {:width$} │     count    │", "queue", width = max_name_width + 12);
    println!("├{}┼──────────────┤", header_sep);

    for metric in metrics {
        let name = metric.name.as_str();
        println!("│ {:<width$} │ {:<12} │", name, metric.count, width = max_name_width + 12);
    }

    println!("└{}┴──────────────┘", header_sep);
    Ok(())
}