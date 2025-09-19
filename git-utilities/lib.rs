pub fn diff_stats_for_commit(
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

pub fn tags_under_prefix(
    repository: &git2::Repository,
    tag_prefix: Option<&str>,
) -> std::collections::HashSet<git2::Oid> {
    let mut tags = std::collections::HashSet::new();
    let tag_iterator = repository.tag_names(tag_prefix).unwrap();
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
    tags
}

pub fn commit_iterator<'a>(
    repository: &'a git2::Repository,
) -> Result<impl Iterator<Item = git2::Commit<'a>>, Box<dyn std::error::Error>> {
    let head = repository.head()?;
    let head_commit = head.peel_to_commit().ok();
    let commits = std::iter::successors(head_commit, |commit| commit.parent(0).ok());
    Ok(commits)
}
