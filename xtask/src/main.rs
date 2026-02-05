//! FlagLite xtask - Development automation tasks
//!
//! Usage: cargo xtask <command>
//!
//! Commands:
//!   (none yet - placeholder for future dev tasks)

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    
    if args.is_empty() {
        eprintln!("Usage: cargo xtask <command>");
        eprintln!();
        eprintln!("Commands:");
        eprintln!("  (none yet - placeholder for future dev tasks)");
        std::process::exit(1);
    }

    match args[0].as_str() {
        cmd => {
            eprintln!("Unknown command: {}", cmd);
            std::process::exit(1);
        }
    }
}
