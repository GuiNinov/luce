use luce_migrations::{MigrationApplier, MigrationRunner};
use std::env;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        print_usage();
        std::process::exit(1);
    }

    let command = &args[1];
    let database_url = &args[2];
    let migrations_dir = args.get(3).map(|s| s.as_str()).unwrap_or("migrations");

    let applier = MigrationApplier::new(database_url).await?;

    match command.as_str() {
        "up" | "migrate" => {
            println!("Running migrations...");
            let count = applier.run_migrations(Path::new(migrations_dir)).await?;
            if count > 0 {
                println!("Successfully applied {} migration(s).", count);
            } else {
                println!("Database is up to date.");
            }
        }
        "status" => {
            println!("Migration status:");
            let applied = applier.get_applied_migrations().await?;
            let pending = applier
                .get_pending_migrations(Path::new(migrations_dir))
                .await?;

            println!("\nApplied migrations ({}):", applied.len());
            for migration in applied {
                println!("  ✓ {} ({})", migration.name, migration.description);
            }

            println!("\nPending migrations ({}):", pending.len());
            for migration in &pending {
                println!("  ○ {} ({})", migration.name, migration.description);
            }

            if pending.is_empty() {
                println!("\nDatabase is up to date!");
            }
        }
        "rollback" => {
            println!("Rolling back last migration...");
            match applier.rollback_last_migration().await? {
                Some(migration) => {
                    println!("✓ Rolled back: {}", migration.name);
                    println!("Note: Only the migration record was removed. Manual schema changes may be required.");
                }
                None => {
                    println!("No migrations to rollback.");
                }
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
            std::process::exit(1);
        }
    }

    Ok(())
}

fn print_usage() {
    println!("Luce Database Migration Tool");
    println!();
    println!("USAGE:");
    println!("    migrate <COMMAND> <DATABASE_URL> [MIGRATIONS_DIR]");
    println!();
    println!("COMMANDS:");
    println!("    up, migrate    Apply all pending migrations");
    println!("    status         Show migration status");
    println!("    rollback       Rollback the last applied migration");
    println!();
    println!("ARGUMENTS:");
    println!("    DATABASE_URL      SQLite database connection string (e.g., sqlite:./luce.db)");
    println!("    MIGRATIONS_DIR    Directory containing migration files (default: migrations)");
    println!();
    println!("EXAMPLES:");
    println!("    migrate up sqlite:./luce.db");
    println!("    migrate status sqlite:./luce.db ./migrations");
    println!("    migrate rollback sqlite:./luce.db");
}
