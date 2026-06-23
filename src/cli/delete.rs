use crate::cli::parse_key;
use crate::store::Store;

/// Arguments for the `delete` command.
#[derive(clap::Args)]
pub struct DeleteArgs {
    /// Key with optional @db suffix (e.g., "foo@work")
    pub key: String,
}

pub fn run(args: DeleteArgs) -> anyhow::Result<()> {
    let (key_bytes, db_name) = parse_key(&args.key)?;
    let store = Store::open(&db_name)?;
    store.delete(&key_bytes)?;
    Ok(())
}
