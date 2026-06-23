use crate::util::paths;

/// Arguments for the `delete-db` command.
#[derive(clap::Args)]
pub struct DeleteDbArgs {
    /// Database name with @ prefix (e.g., @work)
    pub db: String,
}

pub fn run_list_dbs() -> anyhow::Result<()> {
    let dbs = paths::list_dbs()?;
    if dbs.is_empty() {
        println!("(no databases)");
    } else {
        for db in &dbs {
            println!("@{}", db);
        }
    }
    Ok(())
}

pub fn run_delete_db(args: DeleteDbArgs) -> anyhow::Result<()> {
    // Strip leading @ if present
    let db_name = args.db.strip_prefix('@').unwrap_or(&args.db);

    // Check if the db exists — if not, try fuzzy matching for suggestions
    if !paths::db_exists(db_name)? {
        let dbs = paths::list_dbs()?;
        let suggestions = find_suggestions(db_name, &dbs);
        if suggestions.is_empty() {
            anyhow::bail!("database @{} does not exist", db_name);
        } else {
            let s: Vec<String> = suggestions.iter().map(|s| format!("@{}", s)).collect();
            anyhow::bail!("database @{} does not exist. did you mean {}?",
                db_name, s.join(", "));
        }
    }

    let path = paths::db_path(db_name)?;
    let display = paths::display_path(&path);

    // Confirm deletion
    eprint!("Are you sure you want to delete '{}' and all its contents? (y/n) ", display);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim().eq_ignore_ascii_case("y") {
        let deleted = paths::delete_db(db_name)?;
        eprintln!("Deleted '{}'", paths::display_path(&deleted));
    } else {
        eprintln!("Did not delete '{}'", display);
    }

    Ok(())
}

/// Find database name suggestions using Levenshtein distance.
fn find_suggestions(target: &str, dbs: &[String]) -> Vec<String> {
    let mut scored: Vec<(f64, &String)> = dbs
        .iter()
        .map(|db| {
            let dist = strsim::levenshtein(target, db) as f64;
            let max_len = target.len().max(db.len()) as f64;
            let similarity = 1.0 - (dist / max_len.max(1.0));
            (similarity, db)
        })
        .collect();

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    scored
        .into_iter()
        .filter(|(sim, _)| *sim > 0.3)
        .take(3)
        .map(|(_, db)| db.clone())
        .collect()
}
