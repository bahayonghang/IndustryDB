#[cfg(test)]
mod tests {
    use crate::domain::DomainFactory;
    use industrydb_core::config::{ConnectionConfig, DatabaseType};
    use std::collections::HashMap;

    fn dummy_config(_target: &str) -> ConnectionConfig {
        ConnectionConfig {
            db_type: DatabaseType::Mssql,
            host: Some("localhost".into()),
            port: Some(1433),
            username: Some("sa".into()),
            password: Some("password".into()),
            database: Some("master".into()),
            server: None,
            path: None,
            trusted_connection: None,
            timeout: None,
            extra: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn factory_rejects_unknown_target() {
        let cfg = dummy_config("unknown");
        let err = DomainFactory::create(&cfg, "unknown").await;
        assert!(err.is_err());
    }
}
