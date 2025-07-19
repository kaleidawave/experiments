// TODO option for additions etc, option to reverse

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
    let repository = git2::Repository::discover(current_dir)?;

    let mut args = std::env::args().skip(1);
    let tag_prefix = args.next();

    let rest: Vec<_> = args.collect();
    let include_change_statistics = rest.iter().any(|arg| arg == "--stats");

    let head = repository.head()?;
    let head_commit = head.peel_to_commit().ok();

    let mut tags = std::collections::HashSet::new();
    let tag_iterator = repository.tag_names(tag_prefix.as_deref()).unwrap();
    for tag_name in tag_iterator.iter().flatten() {
        if let Ok(reference) = repository.find_reference(&format!("refs/tags/{}", tag_name)) {
            if let Some(target_oid) = reference.target() {
                tags.insert(target_oid);
                if let Ok(tag) = repository.find_tag(target_oid) {
                    dbg!(tag);
                }
            }
        }
    }

    let commits = std::iter::successors(head_commit, |commit| commit.parent(0).ok());
    for commit in commits {
        if tags.contains(&commit.id()) {
            break;
        }
        let summary = commit.summary().unwrap();
        let commit_short_ref = &format!("{id}", id = commit.id())[..7];
        if include_change_statistics {
            let (insertions, deletions) = diff_stats_for_commit(&repository, &commit)?;
            eprintln!(
                "- {summary} ({insertions} insertions and {deletions} deletions) ({commit_short_ref})"
            );
        } else {
            eprintln!("- {summary} ({commit_short_ref})");
        }
    }

    Ok(())
}

fn diff_stats_for_commit(
    repository: &git2::Repository,
    commit: &git2::Commit,
) -> Result<(usize, usize), git2::Error> {
    if let Ok(parent) = commit.parent(0) {
        let tree = commit.tree()?;
        let parent_tree = parent.tree()?;
        let diff = repository.diff_tree_to_tree(Some(&parent_tree), Some(&tree), None)?;

        let stats = diff.stats()?;
        let insertions = stats.insertions();
        let deletions = stats.deletions();

        Ok((insertions, deletions))
    } else {
        Ok((0, 0))
    }
}
