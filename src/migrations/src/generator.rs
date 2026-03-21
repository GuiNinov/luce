use chrono::{Datelike, Timelike, Utc};
use std::fs;
use std::path::{Path, PathBuf};

use crate::migration::MigrationError;

pub struct MigrationGenerator;

impl MigrationGenerator {
    pub fn generate_migration(
        migrations_dir: &Path,
        description: &str,
        content: Option<&str>,
    ) -> Result<PathBuf, MigrationError> {
        // Ensure migrations directory exists
        if !migrations_dir.exists() {
            fs::create_dir_all(migrations_dir).map_err(|e| MigrationError::ReadError {
                error: format!("Failed to create migrations directory: {}", e),
            })?;
        }

        // Generate timestamp in YYYYMMDDHHMMSS format
        let now = Utc::now();
        let timestamp = format!(
            "{:04}{:02}{:02}{:02}{:02}{:02}",
            now.year(),
            now.month(),
            now.day(),
            now.hour(),
            now.minute(),
            now.second()
        );

        // Sanitize description for filename
        let sanitized_description = description
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>()
            .to_lowercase();

        let filename = format!("{}_{}.sql", timestamp, sanitized_description);
        let file_path = migrations_dir.join(&filename);

        // Check if file already exists (timestamp collision)
        if file_path.exists() {
            return Err(MigrationError::InvalidFilename {
                filename: filename.clone(),
            });
        }

        // Default content if none provided
        let default_content = format!(
            "-- Migration: {}\n-- Created at: {}\n\n-- Add your SQL statements here\n",
            description,
            now.to_rfc3339()
        );
        let migration_content = content.unwrap_or(&default_content);

        // Write the migration file
        fs::write(&file_path, migration_content).map_err(|e| MigrationError::ReadError {
            error: format!("Failed to write migration file: {}", e),
        })?;

        println!("Generated migration: {}", file_path.display());
        Ok(file_path)
    }

    pub fn generate_rollback_migration(
        migrations_dir: &Path,
        original_migration_name: &str,
        rollback_content: &str,
    ) -> Result<PathBuf, MigrationError> {
        let description = format!("rollback_{}", original_migration_name.replace(".sql", ""));
        Self::generate_migration(migrations_dir, &description, Some(rollback_content))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_generate_migration() {
        let temp_dir = tempdir().unwrap();
        let migrations_dir = temp_dir.path();

        let file_path = MigrationGenerator::generate_migration(
            migrations_dir,
            "create user table",
            Some("CREATE TABLE users (id INTEGER PRIMARY KEY);"),
        )
        .unwrap();

        assert!(file_path.exists());

        let filename = file_path.file_name().unwrap().to_str().unwrap();
        assert!(filename.ends_with("_create_user_table.sql"));
        assert!(filename.len() == 14 + 1 + "create_user_table".len() + 4); // timestamp + _ + description + .sql

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("CREATE TABLE users"));
    }

    #[test]
    fn test_generate_migration_with_default_content() {
        let temp_dir = tempdir().unwrap();
        let migrations_dir = temp_dir.path();

        let file_path =
            MigrationGenerator::generate_migration(migrations_dir, "test migration", None).unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("-- Migration: test migration"));
        assert!(content.contains("-- Add your SQL statements here"));
    }

    #[test]
    fn test_generate_migration_sanitizes_description() {
        let temp_dir = tempdir().unwrap();
        let migrations_dir = temp_dir.path();

        let file_path = MigrationGenerator::generate_migration(
            migrations_dir,
            "Create User-Table & Add Indexes!",
            Some("-- test"),
        )
        .unwrap();

        let filename = file_path.file_name().unwrap().to_str().unwrap();
        assert!(filename.contains("create_user_table___add_indexes_"));
        assert!(!filename.contains("-"));
        assert!(!filename.contains("&"));
        assert!(!filename.contains("!"));
    }

    #[test]
    fn test_generate_rollback_migration() {
        let temp_dir = tempdir().unwrap();
        let migrations_dir = temp_dir.path();

        let file_path = MigrationGenerator::generate_rollback_migration(
            migrations_dir,
            "20250320174208_create_users.sql",
            "DROP TABLE users;",
        )
        .unwrap();

        let filename = file_path.file_name().unwrap().to_str().unwrap();
        assert!(filename.contains("rollback_20250320174208_create_users"));

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("DROP TABLE users;"));
    }

    #[test]
    fn test_creates_migrations_directory() {
        let temp_dir = tempdir().unwrap();
        let migrations_dir = temp_dir.path().join("new_migrations");

        assert!(!migrations_dir.exists());

        MigrationGenerator::generate_migration(&migrations_dir, "test", Some("-- test")).unwrap();

        assert!(migrations_dir.exists());
    }
}
