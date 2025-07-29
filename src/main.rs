mod conf;
mod config;
mod core;
mod error;
mod logger;
mod oracle;
mod rabbitmq;
mod route;
mod util;

use crate::route::home;
use anyhow::{Context, Result};
use axum::{routing, Router};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    logger::init();

    // 加载配置 - 使用 anyhow 添加顶层上下文
    // let conf = Config::load().context("配置加载失败")?;

    // monitor::monitor_rabbitmq(conf)
    //     .await?
    //     .expect("monitor rabbitmq error");

    // let connect = get_connect_from_config(conf.oracle).await?;
    // tracing::info!("get Oracle connect success");
    // let mut statement = connect
    //     .statement("SELECT REV_ FROM ACT_GE_PROPERTY")
    //     .build()?;
    // let mut cursor = statement.query(&[]).unwrap();
    // while let Some(row) = cursor.next() {
    //     tracing::info!("row: {:?}", row);
    //     println!("value: {:?}", row?.get::<_, String>(0));
    // }

    let router = route::init();
    let dsn = format!("127.0.0.1:{}", conf::get().server.port());
    let listener = TcpListener::bind(&dsn).await?;
    tracing::info!("listening on http://{}", &dsn);
    axum::serve(listener, router).await?;

    Ok(())
}
