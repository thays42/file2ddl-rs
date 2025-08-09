use crate::types::SqlType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub trait DatabaseDialect {
    fn name(&self) -> &'static str;
    fn map_type(&self, sql_type: &SqlType) -> String;
    fn supports_feature(&self, feature: DatabaseFeature) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DatabaseFeature {
    UnlimitedVarchar,
    BooleanType,
    DoublePrecision,
    TimestampType,
}

pub struct PostgreSQL;
pub struct MySQL;
pub struct Netezza;

impl DatabaseDialect for PostgreSQL {
    fn name(&self) -> &'static str {
        "postgresql"
    }

    fn map_type(&self, sql_type: &SqlType) -> String {
        match sql_type {
            SqlType::Boolean => "BOOLEAN".to_string(),
            SqlType::SmallInt => "SMALLINT".to_string(),
            SqlType::Integer => "INTEGER".to_string(),
            SqlType::BigInt => "BIGINT".to_string(),
            SqlType::DoublePrecision => "DOUBLE PRECISION".to_string(),
            SqlType::Date => "DATE".to_string(),
            SqlType::Time => "TIME".to_string(),
            SqlType::DateTime => "TIMESTAMP".to_string(),
            SqlType::Varchar(Some(n)) => format!("VARCHAR({})", n),
            SqlType::Varchar(None) => "TEXT".to_string(),
        }
    }

    fn supports_feature(&self, feature: DatabaseFeature) -> bool {
        match feature {
            DatabaseFeature::UnlimitedVarchar => true,
            DatabaseFeature::BooleanType => true,
            DatabaseFeature::DoublePrecision => true,
            DatabaseFeature::TimestampType => true,
        }
    }
}

impl DatabaseDialect for MySQL {
    fn name(&self) -> &'static str {
        "mysql"
    }

    fn map_type(&self, sql_type: &SqlType) -> String {
        match sql_type {
            SqlType::Boolean => "BOOLEAN".to_string(),
            SqlType::SmallInt => "SMALLINT".to_string(),
            SqlType::Integer => "INTEGER".to_string(),
            SqlType::BigInt => "BIGINT".to_string(),
            SqlType::DoublePrecision => "DOUBLE".to_string(),
            SqlType::Date => "DATE".to_string(),
            SqlType::Time => "TIME".to_string(),
            SqlType::DateTime => "DATETIME".to_string(),
            SqlType::Varchar(Some(n)) => format!("VARCHAR({})", n),
            SqlType::Varchar(None) => "TEXT".to_string(),
        }
    }

    fn supports_feature(&self, feature: DatabaseFeature) -> bool {
        match feature {
            DatabaseFeature::UnlimitedVarchar => true,
            DatabaseFeature::BooleanType => true,
            DatabaseFeature::DoublePrecision => false, // Uses DOUBLE instead
            DatabaseFeature::TimestampType => false, // Uses DATETIME instead
        }
    }
}

impl DatabaseDialect for Netezza {
    fn name(&self) -> &'static str {
        "netezza"
    }

    fn map_type(&self, sql_type: &SqlType) -> String {
        match sql_type {
            SqlType::Boolean => "BOOLEAN".to_string(),
            SqlType::SmallInt => "SMALLINT".to_string(),
            SqlType::Integer => "INTEGER".to_string(),
            SqlType::BigInt => "BIGINT".to_string(),
            SqlType::DoublePrecision => "DOUBLE PRECISION".to_string(),
            SqlType::Date => "DATE".to_string(),
            SqlType::Time => "TIME".to_string(),
            SqlType::DateTime => "TIMESTAMP".to_string(),
            SqlType::Varchar(Some(n)) => format!("VARCHAR({})", n),
            SqlType::Varchar(None) => "VARCHAR(65535)".to_string(),
        }
    }

    fn supports_feature(&self, feature: DatabaseFeature) -> bool {
        match feature {
            DatabaseFeature::UnlimitedVarchar => false, // Has 65535 limit
            DatabaseFeature::BooleanType => true,
            DatabaseFeature::DoublePrecision => true,
            DatabaseFeature::TimestampType => true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub name: String,
    pub type_mappings: HashMap<String, String>,
    pub features: HashMap<String, bool>,
    pub default_varchar_length: Option<usize>,
    pub unlimited_varchar_type: String,
}

impl DatabaseConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: DatabaseConfig = serde_json::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        let required_types = [
            "Boolean", "SmallInt", "Integer", "BigInt", "DoublePrecision",
            "Date", "Time", "DateTime", "Varchar", "VarcharUnlimited"
        ];
        
        for required_type in &required_types {
            if !self.type_mappings.contains_key(*required_type) {
                return Err(anyhow::anyhow!("Missing type mapping for: {}", required_type));
            }
        }
        
        Ok(())
    }

    pub fn to_builtin_databases() -> HashMap<&'static str, DatabaseConfig> {
        let mut configs = HashMap::new();
        
        // PostgreSQL config
        let mut pg_mappings = HashMap::new();
        pg_mappings.insert("Boolean".to_string(), "BOOLEAN".to_string());
        pg_mappings.insert("SmallInt".to_string(), "SMALLINT".to_string());
        pg_mappings.insert("Integer".to_string(), "INTEGER".to_string());
        pg_mappings.insert("BigInt".to_string(), "BIGINT".to_string());
        pg_mappings.insert("DoublePrecision".to_string(), "DOUBLE PRECISION".to_string());
        pg_mappings.insert("Date".to_string(), "DATE".to_string());
        pg_mappings.insert("Time".to_string(), "TIME".to_string());
        pg_mappings.insert("DateTime".to_string(), "TIMESTAMP".to_string());
        pg_mappings.insert("Varchar".to_string(), "VARCHAR({})".to_string());
        pg_mappings.insert("VarcharUnlimited".to_string(), "TEXT".to_string());
        
        let mut pg_features = HashMap::new();
        pg_features.insert("unlimited_varchar".to_string(), true);
        pg_features.insert("boolean_type".to_string(), true);
        
        configs.insert("postgresql", DatabaseConfig {
            name: "PostgreSQL".to_string(),
            type_mappings: pg_mappings,
            features: pg_features,
            default_varchar_length: None,
            unlimited_varchar_type: "TEXT".to_string(),
        });

        // MySQL config
        let mut mysql_mappings = HashMap::new();
        mysql_mappings.insert("Boolean".to_string(), "BOOLEAN".to_string());
        mysql_mappings.insert("SmallInt".to_string(), "SMALLINT".to_string());
        mysql_mappings.insert("Integer".to_string(), "INTEGER".to_string());
        mysql_mappings.insert("BigInt".to_string(), "BIGINT".to_string());
        mysql_mappings.insert("DoublePrecision".to_string(), "DOUBLE".to_string());
        mysql_mappings.insert("Date".to_string(), "DATE".to_string());
        mysql_mappings.insert("Time".to_string(), "TIME".to_string());
        mysql_mappings.insert("DateTime".to_string(), "DATETIME".to_string());
        mysql_mappings.insert("Varchar".to_string(), "VARCHAR({})".to_string());
        mysql_mappings.insert("VarcharUnlimited".to_string(), "TEXT".to_string());
        
        configs.insert("mysql", DatabaseConfig {
            name: "MySQL".to_string(),
            type_mappings: mysql_mappings,
            features: HashMap::new(),
            default_varchar_length: None,
            unlimited_varchar_type: "TEXT".to_string(),
        });

        // Netezza config
        let mut netezza_mappings = HashMap::new();
        netezza_mappings.insert("Boolean".to_string(), "BOOLEAN".to_string());
        netezza_mappings.insert("SmallInt".to_string(), "SMALLINT".to_string());
        netezza_mappings.insert("Integer".to_string(), "INTEGER".to_string());
        netezza_mappings.insert("BigInt".to_string(), "BIGINT".to_string());
        netezza_mappings.insert("DoublePrecision".to_string(), "DOUBLE PRECISION".to_string());
        netezza_mappings.insert("Date".to_string(), "DATE".to_string());
        netezza_mappings.insert("Time".to_string(), "TIME".to_string());
        netezza_mappings.insert("DateTime".to_string(), "TIMESTAMP".to_string());
        netezza_mappings.insert("Varchar".to_string(), "VARCHAR({})".to_string());
        netezza_mappings.insert("VarcharUnlimited".to_string(), "VARCHAR(65535)".to_string());
        
        configs.insert("netezza", DatabaseConfig {
            name: "Netezza".to_string(),
            type_mappings: netezza_mappings,
            features: HashMap::new(),
            default_varchar_length: Some(65535),
            unlimited_varchar_type: "VARCHAR(65535)".to_string(),
        });

        configs
    }
}

pub struct ConfigurableDialect {
    config: DatabaseConfig,
}

impl ConfigurableDialect {
    pub fn new(config: DatabaseConfig) -> Self {
        Self { config }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let config = DatabaseConfig::from_file(path)?;
        Ok(Self::new(config))
    }
}

impl DatabaseDialect for ConfigurableDialect {
    fn name(&self) -> &'static str {
        // Note: This is a limitation - we need to return a static str
        // For dynamic names, we'd need to change the trait signature
        "custom"
    }

    fn map_type(&self, sql_type: &SqlType) -> String {
        let type_key = match sql_type {
            SqlType::Boolean => "Boolean",
            SqlType::SmallInt => "SmallInt", 
            SqlType::Integer => "Integer",
            SqlType::BigInt => "BigInt",
            SqlType::DoublePrecision => "DoublePrecision",
            SqlType::Date => "Date",
            SqlType::Time => "Time",
            SqlType::DateTime => "DateTime",
            SqlType::Varchar(Some(n)) => {
                let default_template = "VARCHAR({})".to_string();
                let template = self.config.type_mappings.get("Varchar")
                    .unwrap_or(&default_template);
                return template.replace("{}", &n.to_string());
            }
            SqlType::Varchar(None) => "VarcharUnlimited",
        };

        self.config.type_mappings
            .get(type_key)
            .cloned()
            .unwrap_or_else(|| format!("UNKNOWN_{}", type_key))
    }

    fn supports_feature(&self, feature: DatabaseFeature) -> bool {
        let feature_key = match feature {
            DatabaseFeature::UnlimitedVarchar => "unlimited_varchar",
            DatabaseFeature::BooleanType => "boolean_type", 
            DatabaseFeature::DoublePrecision => "double_precision",
            DatabaseFeature::TimestampType => "timestamp_type",
        };

        self.config.features.get(feature_key).copied().unwrap_or(false)
    }
}

pub fn get_database_dialect(name: &str) -> anyhow::Result<Box<dyn DatabaseDialect>> {
    match name.to_lowercase().as_str() {
        "postgresql" | "postgres" => Ok(Box::new(PostgreSQL)),
        "mysql" => Ok(Box::new(MySQL)),
        "netezza" => Ok(Box::new(Netezza)),
        _ => Err(anyhow::anyhow!("Unsupported database: {}", name)),
    }
}

pub fn get_database_dialect_from_config<P: AsRef<Path>>(config_path: P) -> anyhow::Result<Box<dyn DatabaseDialect>> {
    let dialect = ConfigurableDialect::from_file(config_path)?;
    Ok(Box::new(dialect))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_postgresql_mapping() {
        let pg = PostgreSQL;
        assert_eq!(pg.map_type(&SqlType::Boolean), "BOOLEAN");
        assert_eq!(pg.map_type(&SqlType::DateTime), "TIMESTAMP");
        assert_eq!(pg.map_type(&SqlType::Varchar(None)), "TEXT");
    }

    #[test]
    fn test_mysql_mapping() {
        let mysql = MySQL;
        assert_eq!(mysql.map_type(&SqlType::DoublePrecision), "DOUBLE");
        assert_eq!(mysql.map_type(&SqlType::DateTime), "DATETIME");
    }

    #[test]
    fn test_netezza_mapping() {
        let netezza = Netezza;
        assert_eq!(netezza.map_type(&SqlType::Varchar(None)), "VARCHAR(65535)");
    }

    #[test]
    fn test_database_factory() {
        assert!(get_database_dialect("postgresql").is_ok());
        assert!(get_database_dialect("mysql").is_ok());
        assert!(get_database_dialect("netezza").is_ok());
        assert!(get_database_dialect("unsupported").is_err());
    }

    #[test]
    fn test_custom_database_config() {
        let config_json = r#"
        {
          "name": "Oracle",
          "type_mappings": {
            "Boolean": "CHAR(1)",
            "SmallInt": "NUMBER(5)",
            "Integer": "NUMBER(10)",
            "BigInt": "NUMBER(19)",
            "DoublePrecision": "BINARY_DOUBLE",
            "Date": "DATE",
            "Time": "TIMESTAMP",
            "DateTime": "TIMESTAMP",
            "Varchar": "VARCHAR2({})",
            "VarcharUnlimited": "CLOB"
          },
          "features": {
            "unlimited_varchar": true,
            "boolean_type": false
          },
          "default_varchar_length": 4000,
          "unlimited_varchar_type": "CLOB"
        }
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", config_json).unwrap();
        
        let dialect = get_database_dialect_from_config(temp_file.path()).unwrap();
        
        assert_eq!(dialect.map_type(&SqlType::Boolean), "CHAR(1)");
        assert_eq!(dialect.map_type(&SqlType::Integer), "NUMBER(10)");
        assert_eq!(dialect.map_type(&SqlType::Varchar(Some(100))), "VARCHAR2(100)");
        assert_eq!(dialect.map_type(&SqlType::Varchar(None)), "CLOB");
    }

    #[test]
    fn test_config_validation() {
        let invalid_config = r#"
        {
          "name": "InvalidDB",
          "type_mappings": {
            "Boolean": "BOOL"
          },
          "features": {},
          "default_varchar_length": null,
          "unlimited_varchar_type": "TEXT"
        }
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", invalid_config).unwrap();
        
        let result = get_database_dialect_from_config(temp_file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_builtin_database_configs() {
        let configs = DatabaseConfig::to_builtin_databases();
        
        assert!(configs.contains_key("postgresql"));
        assert!(configs.contains_key("mysql"));
        assert!(configs.contains_key("netezza"));
        
        let pg_config = &configs["postgresql"];
        assert_eq!(pg_config.name, "PostgreSQL");
        assert_eq!(pg_config.type_mappings["Boolean"], "BOOLEAN");
    }
}