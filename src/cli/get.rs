use crate::cli::parse_key;
use crate::store::Store;
use crate::util::format;

/// Arguments for the `get` command.
#[derive(clap::Args)]
pub struct GetArgs {
    /// Key with optional @db suffix (e.g., "api-key@secrets")
    pub key: String,

    /// Show binary values instead of omitting them
    #[arg(short = 'b', long)]
    pub show_binary: bool,
}

pub fn run(args: GetArgs) -> anyhow::Result<()> {
    let (key_bytes, db_name) = parse_key(&args.key)?;
    let store = Store::open(&db_name)?;

    match store.get(&key_bytes)? {
        Some(value) => {
            if args.show_binary {
                // Write raw bytes to stdout
                use std::io::Write;
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                handle.write_all(&value)?;
            } else {
                format::print_value(&value);
            }
            Ok(())
        }
        None => {
            anyhow::bail!("key '{}' not found in @{}",
                String::from_utf8_lossy(&key_bytes), db_name);
        }
    }
}
