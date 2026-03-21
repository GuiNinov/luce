use luce_migrations::MigrationGenerator;
use std::env;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let description = &args[1];
    let migrations_dir = args.get(2).map(|s| s.as_str()).unwrap_or("migrations");

    let file_path =
        MigrationGenerator::generate_migration(Path::new(migrations_dir), description, None)?;

    println!("✓ Created migration file: {}", file_path.display());
    println!();
    println!("Next steps:");
    println!("1. Edit the migration file to add your SQL statements");
    println!("2. Run 'migrate up <database_url>' to apply the migration");

    Ok(())
}

fn print_usage() {
    println!("Luce Migration Generator");
    println!();
    println!("USAGE:");
    println!("    generate <DESCRIPTION> [MIGRATIONS_DIR]");
    println!();
    println!("ARGUMENTS:");
    println!("    DESCRIPTION       Description of the migration (will be sanitized for filename)");
    println!("    MIGRATIONS_DIR    Directory to create the migration file (default: migrations)");
    println!();
    println!("EXAMPLES:");
    println!("    generate \"create users table\"");
    println!("    generate \"add indexes to tasks\" ./my_migrations");
    println!("    generate \"update task status enum\"");
}
