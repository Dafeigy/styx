use crate::cli::parse_key;
use crate::config::ClioConfig;
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

    let config = ClioConfig::load()?;
    let limit = config.store.max_value_size;

    let value_bytes = match args.value {
        Some(s) => s.into_bytes(),
        None => {
            let mut buf = Vec::new();
            if limit > 0 {
                // Read at most limit+1 bytes; if we reach limit+1, we know
                // the input exceeds the cap without buffering the entire stream.
                std::io::stdin()
                    .take(limit + 1)
                    .read_to_end(&mut buf)?;
            } else {
                std::io::stdin().read_to_end(&mut buf)?;
            }
            buf
        }
    };

    if limit > 0 && value_bytes.len() > limit as usize {
        anyhow::bail!(
            "value is {} bytes, exceeds max_value_size of {} bytes ({:.1} MB)\n\
             Adjust [store].max_value_size in ~/.config/clio/config.toml, \
             or set it to 0 to disable the limit.",
            value_bytes.len(),
            limit,
            limit as f64 / 1_048_576.0,
        );
    }

    let store = Store::open(&db_name)?;
    store.set(&key_bytes, &value_bytes)?;
    Ok(())
}
