use crate::config::OracleConfig;
use crate::error::AppError;
use r2d2_oracle::OracleConnectionManager;

pub async fn create_pool(
    username: &str,
    password: &str,
    host: &str,
    port: u16,
    db_name: &str,
) -> Result<r2d2::Pool<OracleConnectionManager>, AppError> {
    let dsn = format!("//{}:{}/{}", host, port, db_name);
    println!("Connecting to database: {}", dsn);
    let manager = OracleConnectionManager::new(username, password, &dsn);
    Ok(r2d2::Pool::builder().build(manager).unwrap())
}
pub async fn get_connect(
    username: &str,
    password: &str,
    host: &str,
    port: u16,
    db_name: &str,
) -> Result<r2d2::PooledConnection<OracleConnectionManager>, AppError> {
    let pool = create_pool(username, password, host, port, db_name)
        .await?
        .get()
        .unwrap();
    Ok(pool)
}

pub async fn get_connect_from_config(
    oracle_config: OracleConfig,
) -> Result<r2d2::PooledConnection<OracleConnectionManager>, AppError> {
    let pool = create_pool(
        &oracle_config.username,
        &oracle_config.password,
        &oracle_config.host,
        oracle_config.port,
        &oracle_config.db_name,
    )
    .await?
    .get()
    .unwrap();
    Ok(pool)
}
