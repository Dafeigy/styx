/// Arguments for the `push` command.
#[derive(clap::Args)]
pub struct PushArgs {
    /// Database name (e.g., "work" or "@work")
    pub db: Option<String>,

    /// Force push even if remote has changed.
    #[arg(long)]
    pub force: bool,
}

/// Arguments for the `pull` command.
#[derive(clap::Args)]
pub struct PullArgs {
    /// Database name (e.g., "work" or "@work")
    pub db: Option<String>,

    /// Force pull even if local has unsynced changes.
    #[arg(long)]
    pub force: bool,
}

/// Arguments for the `sync` command.
#[derive(clap::Args)]
pub struct SyncArgs {
    /// Force resolution direction when databases have diverged
    #[arg(long, value_enum)]
    pub force: Option<ForceDirection>,
}

#[derive(clap::ValueEnum, Clone)]
pub enum ForceDirection {
    Push,
    Pull,
}

pub async fn run_push(args: PushArgs) -> anyhow::Result<()> {
    let db_name = resolve_db(args.db.as_deref())?;
    let msg = crate::sync::push_db(&db_name, args.force).await?;
    println!("{}", msg);
    Ok(())
}

pub async fn run_pull(args: PullArgs) -> anyhow::Result<()> {
    let db_name = resolve_db(args.db.as_deref())?;
    let msg = crate::sync::pull_db(&db_name, args.force).await?;
    println!("{}", msg);
    Ok(())
}

pub async fn run_sync(_args: SyncArgs) -> anyhow::Result<()> {
    let msg = crate::sync::sync_all().await?;
    println!("{}", msg);
    Ok(())
}

pub async fn run_sync_status() -> anyhow::Result<()> {
    let msg = crate::sync::sync_status().await?;
    println!("{}", msg);
    Ok(())
}

/// Resolve the db name from an optional argument.
/// If not provided, tries all local databases. If only one exists, uses that.
/// If multiple exist, requires explicit @db.
fn resolve_db(db_arg: Option<&str>) -> anyhow::Result<String> {
    match db_arg {
        Some(s) => {
            let s = s.strip_prefix('@').unwrap_or(s);
            Ok(s.to_string())
        }
        None => {
            let dbs = crate::util::paths::list_dbs()?;
            match dbs.len() {
                0 => anyhow::bail!("no databases found; specify one with @db"),
                1 => Ok(dbs[0].clone()),
                _ => anyhow::bail!(
                    "multiple databases exist ({}); specify one with @db",
                    dbs.iter()
                        .map(|d| format!("@{}", d))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            }
        }
    }
}
