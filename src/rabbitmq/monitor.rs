use crate::config::{Config, RabbitMQWarningConfig, WarningType};
use crate::core::service::{QueueMetric, QueueMetricsService};
use crate::util::ding_util;
use anyhow::{Error, Result};
use std::fmt::Write;
use tracing::{error, info};

pub async fn monitor_rabbitmq(config: Config) -> Result<Result<()>, Error> {
    // 创建服务
    let service = QueueMetricsService::new(config.rabbitmq.clone());

    // 执行指标收集
    Ok(match service.gather_metrics().await {
        Ok(metrics) => {
            print_metrics(&metrics)?;
            info!("✅ 监控任务完成");
            send_warning_msg(&metrics, config.rabbitmq.warning).await?;
            Ok(())
        }
        Err(e) => {
            // 获取错误链的所有信息
            let mut full_error = String::new();
            write!(&mut full_error, "{:#}", e)?;

            for cause in e.chain() {
                write!(&mut full_error, "\nCaused by: {}", cause)?;
            }
            ding_util::send(&full_error).await?;
            error!("❌ 监控任务失败: {}", full_error);
            Err(anyhow::anyhow!(full_error))
        }
    })
}

async fn send_warning_msg(
    metrics: &[QueueMetric],
    warning_config: RabbitMQWarningConfig,
) -> std::result::Result<(), Error> {
    if metrics.is_empty() {
        return Ok(());
    }

    if !warning_config.enabled {
        return Ok(());
    }

    let mut warning_msg = "队列数量过多，请检查是否有死信队列或过多的消息积压。\n".to_string();
    let mut warning_count = 0;
    for metric in metrics {
        let name = metric.name.as_str();
        let count = metric.count;
        if count > warning_config.warning_queue_size {
            warning_msg.push_str(format!("{}: {}\n", name, count).as_str());
            warning_count += 1;
        }
    }

    if warning_count == 0 {
        return Ok(());
    }

    warning_msg.push_str(format!("共有 {} 个队列超过警戒值。", warning_count).as_str());

    match warning_config.warning_type {
        None => Ok(()),
        Some(WarningType::DingTalk) => {
            ding_util::send(&warning_msg)
                .await
                .or_else(|err| Err(anyhow::anyhow!("钉钉通知失败: {}", err)))
                .expect("TODO: panic message");
            info!("✅ 已发送钉钉通知");
            Ok(())
        }
    }
    .or_else(|err: anyhow::Error| Err(anyhow::anyhow!(err)))
}

/// 打印监控结果表格
pub fn print_metrics(
    metrics: &[crate::core::service::QueueMetric],
) -> anyhow::Result<(), std::fmt::Error> {
    // 计算最大列宽
    let mut max_name_width = "队列".len();
    for metric in metrics {
        let name = metric.name.as_str();
        max_name_width = max_name_width.max(name.len());
    }

    let header_sep = "─".repeat(max_name_width + 14);

    println!("┌{}┬──────────────┐", header_sep);
    println!(
        "│ {:width$} │     count    │",
        "queue",
        width = max_name_width + 12
    );
    println!("├{}┼──────────────┤", header_sep);

    for metric in metrics {
        let name = metric.name.as_str();
        println!(
            "│ {:<width$} │ {:<12} │",
            name,
            metric.count,
            width = max_name_width + 12
        );
    }

    println!("└{}┴──────────────┘", header_sep);
    Ok(())
}
