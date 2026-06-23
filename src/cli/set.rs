use crate::cli::parse_key;
use crate::store::Store;
use std::io::Read;

/// Arguments for the `set` command.
#[derive(clap::Args)]
pub struct SetArgs {
    /// Key with optional @db suffix (e.g., "foo@work")
    pub key: String,

    /// Value to set. If omitted, reads from stdin.
    pub value: Option<String>,
}

pub fn run(args: SetArgs) -> anyhow::Result<()> {
    let (key_bytes, db_name) = parse_key(&args.key)?;

    if key_bytes.is_empty() {
        anyhow::bail!("key cannot be empty");
    }

    let store = Store::open(&db_name)?;

    let value_bytes = match args.value {
        Some(s) => s.into_bytes(),
        None => {
            let mut buf = Vec::new();
            std::io::stdin().read_to_end(&mut buf)?;
            buf
        }
    };

    store.set(&key_bytes, &value_bytes)?;
    Ok(())
}
