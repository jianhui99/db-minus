use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Driver {
    Postgres,
    MySql,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SslMode {
    Disable,
    Prefer,
    Require,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionConfig {
    pub id: String,
    pub name: String,
    pub driver: Driver,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub database: String,
    pub ssl_mode: SslMode,
}

pub struct ConfigStore {
    path: PathBuf,
}

impl ConfigStore {
    pub fn new(dir: &Path) -> Self {
        Self { path: dir.join("connections.json") }
    }

    pub fn list(&self) -> Result<Vec<ConnectionConfig>, AppError> {
        if !self.path.exists() {
            return Ok(vec![]);
        }
        let text = fs::read_to_string(&self.path)?;
        serde_json::from_str(&text).map_err(|e| AppError::Config(e.to_string()))
    }

    pub fn save(&self, config: ConnectionConfig) -> Result<(), AppError> {
        let mut all = self.list()?;
        match all.iter_mut().find(|c| c.id == config.id) {
            Some(existing) => *existing = config,
            None => all.push(config),
        }
        self.write(&all)
    }

    pub fn delete(&self, id: &str) -> Result<(), AppError> {
        let mut all = self.list()?;
        all.retain(|c| c.id != id);
        self.write(&all)
    }

    fn write(&self, all: &[ConnectionConfig]) -> Result<(), AppError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string_pretty(all).map_err(|e| AppError::Config(e.to_string()))?;
        fs::write(&self.path, text)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample(id: &str, name: &str) -> ConnectionConfig {
        ConnectionConfig {
            id: id.into(),
            name: name.into(),
            driver: Driver::Postgres,
            host: "localhost".into(),
            port: 5433,
            username: "dbminus".into(),
            database: "dbminus_test".into(),
            ssl_mode: SslMode::Disable,
        }
    }

    #[test]
    fn list_empty_when_no_file() {
        let dir = TempDir::new().unwrap();
        let store = ConfigStore::new(dir.path());
        assert_eq!(store.list().unwrap(), vec![]);
    }

    #[test]
    fn save_then_list_roundtrip() {
        let dir = TempDir::new().unwrap();
        let store = ConfigStore::new(dir.path());
        store.save(sample("a", "pg local")).unwrap();
        let listed = store.list().unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "pg local");
    }

    #[test]
    fn save_same_id_upserts() {
        let dir = TempDir::new().unwrap();
        let store = ConfigStore::new(dir.path());
        store.save(sample("a", "old")).unwrap();
        store.save(sample("a", "new")).unwrap();
        let listed = store.list().unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "new");
    }

    #[test]
    fn delete_removes_entry() {
        let dir = TempDir::new().unwrap();
        let store = ConfigStore::new(dir.path());
        store.save(sample("a", "x")).unwrap();
        store.save(sample("b", "y")).unwrap();
        store.delete("a").unwrap();
        let listed = store.list().unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, "b");
    }

    #[test]
    fn config_serializes_camel_case() {
        let json = serde_json::to_value(sample("a", "x")).unwrap();
        assert!(json.get("sslMode").is_some());
        assert_eq!(json["driver"], "postgres");
    }
}
