use std::cmp::Ordering;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SqlType {
    Boolean,
    SmallInt,
    Integer,
    BigInt,
    DoublePrecision,
    Date,
    Time,
    DateTime,
    Varchar(Option<usize>),
}

impl SqlType {
    pub fn promotion_order(&self) -> u8 {
        match self {
            SqlType::Boolean => 0,
            SqlType::SmallInt => 1,
            SqlType::Integer => 2,
            SqlType::BigInt => 3,
            SqlType::DoublePrecision => 4,
            SqlType::Date => 5,
            SqlType::Time => 6,
            SqlType::DateTime => 7,
            SqlType::Varchar(_) => 8,
        }
    }

    pub fn can_promote_to(&self, other: &SqlType) -> bool {
        match (self, other) {
            // Numeric promotions
            (
                SqlType::Boolean,
                SqlType::SmallInt
                | SqlType::Integer
                | SqlType::BigInt
                | SqlType::DoublePrecision
                | SqlType::Varchar(_),
            ) => true,
            (
                SqlType::SmallInt,
                SqlType::Integer | SqlType::BigInt | SqlType::DoublePrecision | SqlType::Varchar(_),
            ) => true,
            (
                SqlType::Integer,
                SqlType::BigInt | SqlType::DoublePrecision | SqlType::Varchar(_),
            ) => true,
            (SqlType::BigInt, SqlType::DoublePrecision | SqlType::Varchar(_)) => true,
            (SqlType::DoublePrecision, SqlType::Varchar(_)) => true,

            // Date/time promotions to VARCHAR
            (SqlType::Date | SqlType::Time | SqlType::DateTime, SqlType::Varchar(_)) => true,

            // VARCHAR can accommodate larger sizes
            (SqlType::Varchar(Some(a)), SqlType::Varchar(Some(b))) => a <= b,
            (SqlType::Varchar(Some(_)), SqlType::Varchar(None)) => true,
            (SqlType::Varchar(None), SqlType::Varchar(_)) => false,

            // Same type
            (a, b) if a == b => true,

            _ => false,
        }
    }

    pub fn promote(&self, other: &SqlType) -> SqlType {
        if self == other {
            return self.clone();
        }

        let self_order = self.promotion_order();
        let other_order = other.promotion_order();

        match self_order.cmp(&other_order) {
            Ordering::Less => {
                if self.can_promote_to(other) {
                    other.clone()
                } else {
                    SqlType::Varchar(None)
                }
            }
            Ordering::Greater => {
                if other.can_promote_to(self) {
                    self.clone()
                } else {
                    SqlType::Varchar(None)
                }
            }
            Ordering::Equal => match (self, other) {
                (SqlType::Varchar(Some(a)), SqlType::Varchar(Some(b))) => {
                    SqlType::Varchar(Some((*a).max(*b)))
                }
                (SqlType::Varchar(_), SqlType::Varchar(None))
                | (SqlType::Varchar(None), SqlType::Varchar(_)) => SqlType::Varchar(None),
                _ => SqlType::Varchar(None),
            },
        }
    }

    pub fn to_postgres_ddl(&self) -> String {
        match self {
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

    pub fn to_mysql_ddl(&self) -> String {
        match self {
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

    pub fn to_netezza_ddl(&self) -> String {
        match self {
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
}

impl fmt::Display for SqlType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SqlType::Boolean => write!(f, "BOOLEAN"),
            SqlType::SmallInt => write!(f, "SMALLINT"),
            SqlType::Integer => write!(f, "INTEGER"),
            SqlType::BigInt => write!(f, "BIGINT"),
            SqlType::DoublePrecision => write!(f, "DOUBLE PRECISION"),
            SqlType::Date => write!(f, "DATE"),
            SqlType::Time => write!(f, "TIME"),
            SqlType::DateTime => write!(f, "DATETIME"),
            SqlType::Varchar(Some(n)) => write!(f, "VARCHAR({})", n),
            SqlType::Varchar(None) => write!(f, "VARCHAR"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ColumnStats {
    pub name: String,
    pub sql_type: SqlType,
    pub null_count: usize,
    pub total_count: usize,
    pub max_length: usize,
    pub min_value: Option<String>,
    pub max_value: Option<String>,
    pub sample_values: Vec<String>,
    pub type_promotions: Vec<String>,
}

impl ColumnStats {
    pub fn new(name: String) -> Self {
        ColumnStats {
            name,
            sql_type: SqlType::Boolean, // Start with most restrictive type
            null_count: 0,
            total_count: 0,
            max_length: 0,
            min_value: None,
            max_value: None,
            sample_values: Vec::new(),
            type_promotions: Vec::new(),
        }
    }

    pub fn null_percentage(&self) -> f64 {
        if self.total_count == 0 {
            0.0
        } else {
            (self.null_count as f64 / self.total_count as f64) * 100.0
        }
    }

    pub fn is_nullable(&self) -> bool {
        self.null_count > 0
    }
}
