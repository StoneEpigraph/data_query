use anyhow::Context;
use reqwest;
use serde_json::json;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use crate::{config::Config};
type HmacSha256 = Hmac<Sha256>;

pub async fn send(content: &str) -> anyhow::Result<String> {
    // 加载配置 - 使用 anyhow 添加顶层上下文
    let config = Config::load()
        .context("配置加载失败")?;
    let dingtalk_config = config.dingtalk.clone();
    let webhook_url = dingtalk_config.webhook_url.clone();
    let secret = dingtalk_config.secret.clone();
    let access_token = dingtalk_config.custom_robot_token.clone();
    send_message(content, &webhook_url, &secret, &access_token, true).await?;
    Ok("发送成功".to_owned())
}

// 发送加签消息
async fn send_message(content: &str, webhook_url: &str, secret: &str, access_token: &str, is_at_all: bool) -> Result<(), reqwest::Error> {
    // 1. 生成签名
    let timestamp = chrono::Utc::now().timestamp_millis().to_string();
    let sign_str = format!("{}\n{}", timestamp, secret);
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC init failed");
    mac.update(sign_str.as_bytes());
    let sign = base64::encode(mac.finalize().into_bytes());

    // 2. 构建请求
    let url = format!(
        "{}?access_token={}&timestamp={}&sign={}",
        webhook_url, access_token, timestamp, sign
    );
    let payload = json!({
        "msgtype": "text",
        "text": {"content": content},
        "at": {"isAtAll": false}  // 可选：@特定用户
    });

    // 3. 发送请求
    let client = reqwest::Client::new();
    client.post(&url)
        .json(&payload)
        .send()
        .await?;
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_send_message() {
        let content = "测试内容";
        let result = send(content).await;
        assert!(result.is_ok());
    }
}