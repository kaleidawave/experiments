// TODO option for additions etc, option to reverse
use git_utilities::{commit_iterator, diff_stats_for_commit, tags_under_prefix};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;

    let mut args = std::env::args().skip(1);
    let tag_prefix = args.next();

    let rest: Vec<_> = args.collect();
    let include_change_statistics = rest.iter().any(|arg| arg == "--stats");

    let repository = git2::Repository::discover(current_dir)?;
    let tags = tags_under_prefix(&repository, tag_prefix.as_deref());
    let commits = commit_iterator(&repository)?;

    for (idx, commit) in commits.enumerate() {
        if tags.contains(&commit.id()) {
            if idx == 0 {
                continue;
            } else {
                break;
            }
        }
        let summary = commit.summary().unwrap();
        let commit_short_ref = &commit.id().to_string()[..7];
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
