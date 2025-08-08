use sea_orm::{Database, DatabaseConnection as DbConn};
use std::env;
use tracing::{error, info};

#[derive(Clone)]
pub struct DatabaseConnection {
    conn: DbConn,
}

impl DatabaseConnection {
    pub async fn new() -> Result<Self, sea_orm::DbErr> {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://lux_user:lux_password@localhost/lux_mcp".to_string());

        info!("Connecting to database...");

        match Database::connect(&database_url).await {
            Ok(conn) => {
                info!("Successfully connected to database");
                Ok(Self { conn })
            }
            Err(e) => {
                error!("Failed to connect to database: {}", e);
                Err(e)
            }
        }
    }

    pub fn get_connection(&self) -> &DbConn {
        &self.conn
    }
}
