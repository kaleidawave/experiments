fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open the Git repository and get statuses
    let current_dir = std::env::current_dir()?;
    let repo = git2::Repository::discover(current_dir)?;
    let statuses = repo.statuses(None)?;

    // Check if the file is modified but not staged
    let mut is_dirty = false;
    for entry in statuses.iter() {
        let status = entry.status();
        if let Some(_path) = entry.path() {
            let working_tree = status.is_wt_deleted()
                || status.is_wt_typechange()
                || status.is_wt_renamed()
                || status.is_wt_new()
                || status.is_wt_modified();

            // eprintln!("{path} {working_tree:?}");

            is_dirty |= working_tree;
        }
    }

    dbg!(is_dirty);

    Ok(())
}
