use super::{is_within_symlink, path_passes_filters};
use ignore::gitignore::Gitignore;
use virtual_fs::{Stub, VirtualFS};

#[test]
fn test_path_passes_filters() {
    VirtualFS::test("test_path_passes_filters", |dirs, mut sandbox| {
        sandbox.mkdir("my_repo");
        sandbox.mkdir("my_repo/.git");
        sandbox.mkdir("my_repo/.git/refs");
        sandbox.mkdir("my_repo/.git/refs/heads");
        sandbox.mkdir("my_repo/src");
        sandbox.mkdir("my_repo/target");
        sandbox.mkdir("my_repo/target/debug");
        sandbox.mkdir("outside_of_codebase");
        sandbox.with_files(vec![
            Stub::EmptyFile("my_repo/README.txt"),
            Stub::EmptyFile("my_repo/.git/blob.txt"),
            Stub::EmptyFile("my_repo/.git/HEAD"),
            Stub::EmptyFile("my_repo/.git/refs/heads/main"),
            Stub::EmptyFile("my_repo/.git/refs/heads/feature-branch"),
            Stub::EmptyFile("my_repo/src/main.rs"),
            Stub::EmptyFile("my_repo/target/debug/a.out"),
            Stub::EmptyFile("outside_of_codebase/text.txt"),
        ]);
        sandbox.with_files(vec![Stub::FileWithContent("my_repo/.gitignore", "target")]);

        let test_gitignore_entry = dirs.tests().join("my_repo/.gitignore");
        let gitignores = vec![Gitignore::new(test_gitignore_entry).0];

        // Do NOT ignore a file that does not exist (for deletions)
        assert!(path_passes_filters(
            dirs.tests().join("my_repo/does_not_exist.txt").as_path(),
            &gitignores
        ));

        assert!(path_passes_filters(
            dirs.tests().join("my_repo/src").as_path(),
            &gitignores
        ));
        assert!(path_passes_filters(
            dirs.tests().join("my_repo/src/main.rs").as_path(),
            &gitignores
        ));
        assert!(path_passes_filters(
            dirs.tests().join("outside_of_codebase/text.txt").as_path(),
            &gitignores
        ));

        // Allow .git internal files that provide useful signals
        assert!(path_passes_filters(
            dirs.tests().join("my_repo/.git/HEAD").as_path(),
            &gitignores
        ));
        assert!(path_passes_filters(
            dirs.tests().join("my_repo/.git/refs/heads").as_path(),
            &gitignores
        ));
        assert!(path_passes_filters(
            dirs.tests().join("my_repo/.git/refs/heads/main").as_path(),
            &gitignores
        ));
        assert!(path_passes_filters(
            dirs.tests()
                .join("my_repo/.git/refs/heads/feature-branch")
                .as_path(),
            &gitignores
        ));
        // Non-allowlisted .git/ internal files are filtered out
        assert!(!path_passes_filters(
            dirs.tests().join("my_repo/.git/index").as_path(),
            &gitignores
        ));
        assert!(!path_passes_filters(
            dirs.tests().join("my_repo/.git/blob.txt").as_path(),
            &gitignores
        ));

        // .git directory itself is still ignored
        assert!(!path_passes_filters(
            dirs.tests().join("my_repo/.git").as_path(),
            &gitignores
        ));

        // Ignore .gitignored paths and their children.
        assert!(!path_passes_filters(
            dirs.tests().join("my_repo/target/").as_path(),
            &gitignores
        ));
        assert!(!path_passes_filters(
            dirs.tests().join("my_repo/target/debug").as_path(),
            &gitignores
        ));
        assert!(!path_passes_filters(
            dirs.tests().join("my_repo/target/debug/a.out").as_path(),
            &gitignores
        ));

        // Ignore a .gitignored file that does not exist (for deletions)
        assert!(!path_passes_filters(
            &dirs.tests().join("my_repo/target/does_not_exist.txt"),
            &gitignores
        ));

        // Ensure paths are canonicalized before being matched against gitignores.
        assert!(path_passes_filters(
            dirs.tests()
                .join("outside_of_codebase/../my_repo/README.txt")
                .as_path(),
            &gitignores
        ));
        assert!(!path_passes_filters(
            dirs.tests()
                .join("outside_of_codebase/../my_repo/target/debug/a.out")
                .as_path(),
            &gitignores
        ));
    });
}

#[test]
fn test_git_path_filtering_allowlist() {
    use super::{is_commit_related_git_file, is_index_lock_file, should_ignore_git_path};
    use std::path::Path;

    // Non-git paths should not be ignored
    assert!(!should_ignore_git_path(Path::new(
        "/home/user/project/src/main.rs"
    )));
    assert!(!should_ignore_git_path(Path::new(
        "/home/user/project/README.md"
    )));

    // .git directory itself should be ignored
    assert!(should_ignore_git_path(Path::new("/home/user/project/.git")));

    // Allowlisted: commit-related files are NOT ignored
    assert!(!should_ignore_git_path(Path::new(
        "/home/user/project/.git/HEAD"
    )));
    assert!(!should_ignore_git_path(Path::new(
        "/home/user/project/.git/refs/heads/main"
    )));
    assert!(!should_ignore_git_path(Path::new(
        "/home/user/project/.git/refs/heads/feature-branch"
    )));

    // Allowlisted: index.lock is NOT ignored
    assert!(!should_ignore_git_path(Path::new(
        "/home/user/project/.git/index.lock"
    )));

    // Everything else in .git/ IS ignored
    assert!(should_ignore_git_path(Path::new(
        "/home/user/project/.git/index"
    )));
    assert!(should_ignore_git_path(Path::new(
        "/home/user/project/.git/config"
    )));
    assert!(should_ignore_git_path(Path::new(
        "/home/user/project/.git/COMMIT_EDITMSG"
    )));
    assert!(should_ignore_git_path(Path::new(
        "/home/user/project/.git/FETCH_HEAD"
    )));
    assert!(should_ignore_git_path(Path::new(
        "/home/user/project/.git/ORIG_HEAD"
    )));
    assert!(should_ignore_git_path(Path::new(
        "/home/user/project/.git/refs/tags/v1.0"
    )));
    assert!(should_ignore_git_path(Path::new(
        "/home/user/project/.git/refs/remotes/origin/main"
    )));
    assert!(should_ignore_git_path(Path::new(
        "/home/user/project/.git/objects/abc123"
    )));
    assert!(should_ignore_git_path(Path::new(
        "/home/user/project/.git/hooks/pre-commit"
    )));
    assert!(should_ignore_git_path(Path::new(
        "/home/user/project/.git/logs/HEAD"
    )));

    // Worktree paths: allowlisted patterns under .git/worktrees/<name>/
    assert!(!should_ignore_git_path(Path::new(
        "/home/user/project/.git/worktrees/my-wt/HEAD"
    )));
    assert!(!should_ignore_git_path(Path::new(
        "/home/user/project/.git/worktrees/my-wt/index.lock"
    )));
    // Non-allowlisted worktree paths are still ignored
    assert!(should_ignore_git_path(Path::new(
        "/home/user/project/.git/worktrees/my-wt/index"
    )));
    assert!(should_ignore_git_path(Path::new(
        "/home/user/project/.git/worktrees/my-wt/COMMIT_EDITMSG"
    )));
    // worktrees dir itself (no content after worktree name) is ignored
    assert!(should_ignore_git_path(Path::new(
        "/home/user/project/.git/worktrees"
    )));
    assert!(should_ignore_git_path(Path::new(
        "/home/user/project/.git/worktrees/my-wt"
    )));

    // is_commit_related_git_file
    assert!(is_commit_related_git_file(Path::new("/repo/.git/HEAD")));
    assert!(is_commit_related_git_file(Path::new(
        "/repo/.git/refs/heads/main"
    )));
    assert!(is_commit_related_git_file(Path::new(
        "/repo/.git/worktrees/wt/HEAD"
    )));
    assert!(!is_commit_related_git_file(Path::new(
        "/repo/.git/index.lock"
    )));
    assert!(!is_commit_related_git_file(Path::new(
        "/repo/.git/refs/tags/v1"
    )));

    // is_index_lock_file
    assert!(is_index_lock_file(Path::new("/repo/.git/index.lock")));
    assert!(is_index_lock_file(Path::new(
        "/repo/.git/worktrees/wt/index.lock"
    )));
    assert!(!is_index_lock_file(Path::new("/repo/.git/HEAD")));
    assert!(!is_index_lock_file(Path::new("/repo/.git/index")));
}

#[test]
fn test_is_shared_git_ref() {
    use super::is_shared_git_ref;
    use std::path::Path;

    // Shared refs — broadcast to all repos
    assert!(is_shared_git_ref(Path::new("/repo/.git/refs/heads/main")));
    assert!(is_shared_git_ref(Path::new(
        "/repo/.git/refs/heads/feature"
    )));

    // Repo-specific — NOT shared
    assert!(!is_shared_git_ref(Path::new("/repo/.git/HEAD")));
    assert!(!is_shared_git_ref(Path::new("/repo/.git/index.lock")));

    // Worktree paths — NOT shared
    assert!(!is_shared_git_ref(Path::new(
        "/repo/.git/worktrees/foo/HEAD"
    )));
    assert!(!is_shared_git_ref(Path::new(
        "/repo/.git/worktrees/foo/refs/heads/main"
    )));

    // Other .git internals — NOT shared
    assert!(!is_shared_git_ref(Path::new("/repo/.git/refs/tags/v1")));
    assert!(!is_shared_git_ref(Path::new(
        "/repo/.git/refs/remotes/origin/main"
    )));
    assert!(!is_shared_git_ref(Path::new("/repo/.git/config")));

    // Not a git path at all
    assert!(!is_shared_git_ref(Path::new("/repo/src/main.rs")));
}

#[test]
fn test_extract_worktree_git_dir() {
    use super::extract_worktree_git_dir;
    use std::path::{Path, PathBuf};

    // Standard worktree path extracts the per-worktree gitdir
    assert_eq!(
        extract_worktree_git_dir(Path::new("/repo/.git/worktrees/foo/HEAD")),
        Some(PathBuf::from("/repo/.git/worktrees/foo"))
    );
    assert_eq!(
        extract_worktree_git_dir(Path::new("/repo/.git/worktrees/bar/index.lock")),
        Some(PathBuf::from("/repo/.git/worktrees/bar"))
    );

    // Non-worktree paths return None
    assert_eq!(extract_worktree_git_dir(Path::new("/repo/.git/HEAD")), None);
    assert_eq!(
        extract_worktree_git_dir(Path::new("/repo/.git/refs/heads/main")),
        None
    );
    assert_eq!(
        extract_worktree_git_dir(Path::new("/repo/src/main.rs")),
        None
    );

    // Edge case: not enough depth after worktrees/
    assert_eq!(
        extract_worktree_git_dir(Path::new("/repo/.git/worktrees")),
        None
    );
    assert_eq!(
        extract_worktree_git_dir(Path::new("/repo/.git/worktrees/foo")),
        None
    );
}

#[cfg(unix)]
#[test]
fn is_within_symlink_prunes_directory_symlinks_and_descendants() {
    let tmp = tempfile::tempdir().unwrap();
    let repo_root = tmp.path().canonicalize().unwrap();
    let target = repo_root.join("external");
    std::fs::create_dir_all(&target).unwrap();
    let symlink = repo_root.join("result");
    std::os::unix::fs::symlink(&target, &symlink).unwrap();
    let descendant = symlink.join("nested");

    assert!(is_within_symlink(&symlink, &repo_root));
    assert!(is_within_symlink(&descendant, &repo_root));

    let normal_dir = repo_root.join("src");
    std::fs::create_dir_all(&normal_dir).unwrap();
    assert!(!is_within_symlink(&normal_dir, &repo_root));
}

#[cfg(unix)]
#[test]
fn is_within_symlink_allows_symlinked_repo_root() {
    let tmp = tempfile::tempdir().unwrap();
    let real_root = tmp.path().join("real_repo");
    std::fs::create_dir_all(&real_root).unwrap();
    let symlinked_root = tmp.path().join("alias");
    std::os::unix::fs::symlink(&real_root, &symlinked_root).unwrap();
    let repo_root = symlinked_root.canonicalize().unwrap();

    assert!(!is_within_symlink(&repo_root, &repo_root));
    let child = repo_root.join("src");
    std::fs::create_dir_all(&child).unwrap();
    assert!(!is_within_symlink(&child, &repo_root));
}

#[cfg(unix)]
#[test]
fn is_within_symlink_ignores_symlinks_above_repo_root() {
    let tmp = tempfile::tempdir().unwrap();
    let symlinked_parent = tmp.path().join("link_parent");
    let real_parent = tmp.path().join("real_parent");
    std::fs::create_dir_all(&real_parent).unwrap();
    std::os::unix::fs::symlink(&real_parent, &symlinked_parent).unwrap();
    let repo_root = symlinked_parent.join("repo");
    std::fs::create_dir_all(&repo_root).unwrap();
    let repo_root = repo_root.canonicalize().unwrap();

    let child = repo_root.join("src");
    std::fs::create_dir_all(&child).unwrap();
    assert!(!is_within_symlink(&child, &repo_root));
}
