use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use regex::Regex;
use std::cmp::Ordering;
use std::path::PathBuf;

#[derive(Debug, Clone, thiserror::Error)]
pub enum MigrationError {
    #[error("Invalid migration filename: {filename}")]
    InvalidFilename { filename: String },

    #[error("Migration file not found: {path}")]
    FileNotFound { path: String },

    #[error("Failed to read migration file: {error}")]
    ReadError { error: String },

    #[error("Database error: {error}")]
    DatabaseError { error: String },

    #[error("Migration already applied: {name}")]
    AlreadyApplied { name: String },

    #[error("Migration not found in database: {name}")]
    NotFoundInDatabase { name: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Migration {
    pub name: String,
    pub timestamp: String,
    pub description: String,
    pub content: String,
    pub file_path: PathBuf,
    pub applied_at: Option<DateTime<Utc>>,
}

impl Migration {
    pub fn new(file_path: PathBuf, content: String) -> Result<Self, MigrationError> {
        let filename = file_path
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or_else(|| MigrationError::InvalidFilename {
                filename: file_path.to_string_lossy().to_string(),
            })?;

        let (timestamp, description) = Self::parse_filename(filename)?;

        Ok(Migration {
            name: filename.to_string(),
            timestamp,
            description,
            content,
            file_path,
            applied_at: None,
        })
    }

    pub fn with_applied_at(mut self, applied_at: DateTime<Utc>) -> Self {
        self.applied_at = Some(applied_at);
        self
    }

    fn parse_filename(filename: &str) -> Result<(String, String), MigrationError> {
        // Expected format: YYYYMMDDHHMMSS_description.sql
        let re = Regex::new(r"^(\d{14})_(.+)\.sql$").unwrap();

        if let Some(captures) = re.captures(filename) {
            let timestamp = captures.get(1).unwrap().as_str().to_string();
            let description = captures.get(2).unwrap().as_str().replace('_', " ");
            Ok((timestamp, description))
        } else {
            Err(MigrationError::InvalidFilename {
                filename: filename.to_string(),
            })
        }
    }

    pub fn is_applied(&self) -> bool {
        self.applied_at.is_some()
    }

    pub fn get_timestamp_as_datetime(&self) -> Result<DateTime<Utc>, MigrationError> {
        // Parse YYYYMMDDHHMMSS format
        let year: i32 =
            self.timestamp[0..4]
                .parse()
                .map_err(|_| MigrationError::InvalidFilename {
                    filename: self.name.clone(),
                })?;
        let month: u32 =
            self.timestamp[4..6]
                .parse()
                .map_err(|_| MigrationError::InvalidFilename {
                    filename: self.name.clone(),
                })?;
        let day: u32 =
            self.timestamp[6..8]
                .parse()
                .map_err(|_| MigrationError::InvalidFilename {
                    filename: self.name.clone(),
                })?;
        let hour: u32 =
            self.timestamp[8..10]
                .parse()
                .map_err(|_| MigrationError::InvalidFilename {
                    filename: self.name.clone(),
                })?;
        let minute: u32 =
            self.timestamp[10..12]
                .parse()
                .map_err(|_| MigrationError::InvalidFilename {
                    filename: self.name.clone(),
                })?;
        let second: u32 =
            self.timestamp[12..14]
                .parse()
                .map_err(|_| MigrationError::InvalidFilename {
                    filename: self.name.clone(),
                })?;

        Ok(Utc
            .with_ymd_and_hms(year, month, day, hour, minute, second)
            .single()
            .ok_or_else(|| MigrationError::InvalidFilename {
                filename: self.name.clone(),
            })?)
    }
}

impl Ord for Migration {
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}

impl PartialOrd for Migration {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_parse_valid_filename() {
        let (timestamp, description) =
            Migration::parse_filename("20250320174208_first_migration.sql").unwrap();
        assert_eq!(timestamp, "20250320174208");
        assert_eq!(description, "first migration");
    }

    #[test]
    fn test_parse_filename_with_underscores() {
        let (timestamp, description) =
            Migration::parse_filename("20250320174209_create_user_table.sql").unwrap();
        assert_eq!(timestamp, "20250320174209");
        assert_eq!(description, "create user table");
    }

    #[test]
    fn test_parse_invalid_filename() {
        assert!(Migration::parse_filename("invalid_filename.sql").is_err());
        assert!(Migration::parse_filename("20250320_short.sql").is_err());
        assert!(Migration::parse_filename("20250320174208_no_extension").is_err());
    }

    #[test]
    fn test_migration_creation() {
        let path = PathBuf::from("20250320174208_first_migration.sql");
        let content = "CREATE TABLE test (id INTEGER);".to_string();

        let migration = Migration::new(path, content.clone()).unwrap();

        assert_eq!(migration.name, "20250320174208_first_migration.sql");
        assert_eq!(migration.timestamp, "20250320174208");
        assert_eq!(migration.description, "first migration");
        assert_eq!(migration.content, content);
        assert!(!migration.is_applied());
    }

    #[test]
    fn test_migration_ordering() {
        let path1 = PathBuf::from("20250320174208_first.sql");
        let path2 = PathBuf::from("20250320174209_second.sql");

        let migration1 = Migration::new(path1, "".to_string()).unwrap();
        let migration2 = Migration::new(path2, "".to_string()).unwrap();

        assert!(migration1 < migration2);
    }

    #[test]
    fn test_migration_with_applied_at() {
        let path = PathBuf::from("20250320174208_test.sql");
        let applied_time = Utc::now();

        let migration = Migration::new(path, "".to_string())
            .unwrap()
            .with_applied_at(applied_time);

        assert!(migration.is_applied());
        assert_eq!(migration.applied_at, Some(applied_time));
    }

    #[test]
    fn test_timestamp_as_datetime() {
        let path = PathBuf::from("20250320174208_test.sql");
        let migration = Migration::new(path, "".to_string()).unwrap();

        let datetime = migration.get_timestamp_as_datetime().unwrap();
        assert_eq!(datetime.year(), 2025);
        assert_eq!(datetime.month(), 3);
        assert_eq!(datetime.day(), 20);
        assert_eq!(datetime.hour(), 17);
        assert_eq!(datetime.minute(), 42);
        assert_eq!(datetime.second(), 8);
    }
}
