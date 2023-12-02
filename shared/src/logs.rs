#![allow(clippy::crate_in_macro_def)]
use pgrx::*;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub const DEFAULT_LOG_LEVEL: LogLevel = LogLevel::INFO;
pub static mut PARADE_LOGS_TABLE_INITIALIZED: bool = false;

// Logs will live in the table created below.
// The schema must already exist when this code is executed.
extension_sql!(
    r#"
    DO $$
    BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_tables
                   WHERE schemaname = 'paradedb' AND tablename = 'logs') THEN
        CREATE TABLE paradedb.logs (
            id SERIAL PRIMARY KEY,
            timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            level TEXT NOT NULL,
            module TEXT NOT NULL,
            file TEXT NOT NULL,
            line INTEGER NOT NULL,
            message TEXT NOT NULL,
            json JSON,
            pid INTEGER NOT NULL,
            backtrace TEXT
        );
        ELSE
            RAISE WARNING 'The table paradedb.logs already exists, skipping.';
        END IF;
    END $$;
    "#
    name = "create_parade_logs_table"
);

#[derive(Serialize, Deserialize, Debug)]
pub enum LogLevel {
    INFO,
    WARN,
    ERROR,
    DEBUG,
    TRACE,
}

impl IntoDatum for LogLevel {
    fn into_datum(self) -> Option<pgrx::pg_sys::Datum> {
        let self_string = &self.to_string();
        self_string.into_datum()
    }

    fn type_oid() -> pgrx::pg_sys::Oid {
        pgrx::prelude::pg_sys::TEXTOID
    }
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize)]
pub struct LogJson {
    pub data: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl IntoDatum for LogJson {
    fn into_datum(self) -> Option<pgrx::pg_sys::Datum> {
        let string = serde_json::to_string(&self).expect("failed to serialize Json value");
        string.into_datum()
    }

    fn type_oid() -> pgrx::prelude::pg_sys::Oid {
        pgrx::prelude::pg_sys::TEXTOID
    }
}

impl Display for LogJson {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string(self) {
            Ok(json_str) => write!(f, "{}", json_str),
            Err(_) => write!(f, "{{}}"), // Fallback to an empty JSON object
        }
    }
}
