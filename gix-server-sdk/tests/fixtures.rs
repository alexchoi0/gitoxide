use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

pub struct TestRepo {
    #[allow(dead_code)]
    dir: TempDir,
    pub path: PathBuf,
}

impl TestRepo {
    pub fn new() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::create_dir_all(path.join("src")).expect("failed to create src dir");
        std::fs::create_dir_all(path.join("docs")).expect("failed to create docs dir");

        std::fs::write(path.join("README.md"), "# Test Repository\n\nThis is a test repository.\n")
            .expect("failed to write README.md");

        std::fs::write(
            path.join("src/main.rs"),
            r#"fn main() {
    println!("Hello, world!");
}
"#,
        )
        .expect("failed to write main.rs");

        std::fs::write(
            path.join("src/lib.rs"),
            r#"pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }
}
"#,
        )
        .expect("failed to write lib.rs");

        std::fs::write(
            path.join("docs/guide.md"),
            "# User Guide\n\n## Getting Started\n\nWelcome to the guide.\n",
        )
        .expect("failed to write guide.md");

        let binary_data: Vec<u8> = (0..=255).collect();
        std::fs::write(path.join("data.bin"), &binary_data).expect("failed to write binary file");

        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit"]);

        Self { dir, path }
    }

    pub fn with_history() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "alice@example.com"]);
        run_git(&path, &["config", "user.name", "Alice Developer"]);

        std::fs::create_dir_all(path.join("src")).expect("failed to create src dir");
        std::fs::create_dir_all(path.join("tests")).expect("failed to create tests dir");

        std::fs::write(path.join("README.md"), "# Project\n").expect("failed to write README.md");
        std::fs::write(path.join("src/main.rs"), "fn main() {}\n")
            .expect("failed to write main.rs");

        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial project setup"]);

        run_git(&path, &["config", "user.email", "bob@example.com"]);
        run_git(&path, &["config", "user.name", "Bob Contributor"]);

        std::fs::write(
            path.join("src/lib.rs"),
            r#"pub fn hello() -> &'static str {
    "Hello"
}
"#,
        )
        .expect("failed to write lib.rs");

        run_git(&path, &["add", "src/lib.rs"]);
        run_git(&path, &["commit", "-m", "Add library module"]);

        run_git(&path, &["config", "user.email", "alice@example.com"]);
        run_git(&path, &["config", "user.name", "Alice Developer"]);

        std::fs::write(
            path.join("src/main.rs"),
            r#"mod lib;

fn main() {
    println!("{}", lib::hello());
}
"#,
        )
        .expect("failed to write main.rs");

        run_git(&path, &["add", "src/main.rs"]);
        run_git(&path, &["commit", "-m", "Integrate library into main"]);

        run_git(&path, &["config", "user.email", "carol@example.com"]);
        run_git(&path, &["config", "user.name", "Carol Tester"]);

        std::fs::write(
            path.join("tests/integration.rs"),
            r#"#[test]
fn test_hello() {
    assert_eq!(project::hello(), "Hello");
}
"#,
        )
        .expect("failed to write integration test");

        run_git(&path, &["add", "tests/integration.rs"]);
        run_git(&path, &["commit", "-m", "Add integration tests"]);

        run_git(&path, &["config", "user.email", "bob@example.com"]);
        run_git(&path, &["config", "user.name", "Bob Contributor"]);

        std::fs::write(
            path.join("src/lib.rs"),
            r#"pub fn hello() -> &'static str {
    "Hello"
}

pub fn goodbye() -> &'static str {
    "Goodbye"
}
"#,
        )
        .expect("failed to write lib.rs");

        run_git(&path, &["add", "src/lib.rs"]);
        run_git(&path, &["commit", "-m", "Add goodbye function"]);

        run_git(&path, &["config", "user.email", "alice@example.com"]);
        run_git(&path, &["config", "user.name", "Alice Developer"]);

        std::fs::write(
            path.join("README.md"),
            "# Project\n\nA sample Rust project.\n\n## Features\n\n- Hello function\n- Goodbye function\n",
        )
        .expect("failed to write README.md");

        std::fs::remove_file(path.join("tests/integration.rs"))
            .expect("failed to remove integration test");
        std::fs::remove_dir(path.join("tests")).expect("failed to remove tests dir");

        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Update README and remove old tests"]);

        Self { dir, path }
    }

    pub fn with_submodules() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        let submodule_dir = path.join("submodule_source");
        std::fs::create_dir_all(&submodule_dir).expect("failed to create submodule source dir");

        run_git(&submodule_dir, &["init"]);
        run_git(&submodule_dir, &["config", "user.email", "test@example.com"]);
        run_git(&submodule_dir, &["config", "user.name", "Test User"]);

        std::fs::write(submodule_dir.join("lib.rs"), "pub fn submodule_func() {}\n")
            .expect("failed to write submodule lib.rs");

        run_git(&submodule_dir, &["add", "."]);
        run_git(&submodule_dir, &["commit", "-m", "Submodule initial commit"]);

        let main_repo_dir = path.join("main_repo");
        std::fs::create_dir_all(&main_repo_dir).expect("failed to create main repo dir");

        run_git(&main_repo_dir, &["init"]);
        run_git(&main_repo_dir, &["config", "user.email", "test@example.com"]);
        run_git(&main_repo_dir, &["config", "user.name", "Test User"]);

        std::fs::write(main_repo_dir.join("main.rs"), "fn main() {}\n")
            .expect("failed to write main.rs");

        run_git(&main_repo_dir, &["add", "."]);
        run_git(&main_repo_dir, &["commit", "-m", "Main repo initial commit"]);

        let submodule_path_str = submodule_dir.to_str().expect("invalid submodule path");
        run_git(
            &main_repo_dir,
            &["submodule", "add", submodule_path_str, "vendor/submodule"],
        );
        run_git(&main_repo_dir, &["commit", "-m", "Add submodule"]);

        Self {
            dir,
            path: main_repo_dir,
        }
    }

    pub fn with_multiple_submodules() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        let submodule1_dir = path.join("submodule1_source");
        std::fs::create_dir_all(&submodule1_dir).expect("failed to create submodule1 source dir");
        run_git(&submodule1_dir, &["init"]);
        run_git(&submodule1_dir, &["config", "user.email", "test@example.com"]);
        run_git(&submodule1_dir, &["config", "user.name", "Test User"]);
        std::fs::write(submodule1_dir.join("lib.rs"), "pub fn func1() {}\n")
            .expect("failed to write submodule1 lib.rs");
        run_git(&submodule1_dir, &["add", "."]);
        run_git(&submodule1_dir, &["commit", "-m", "Submodule1 initial commit"]);

        let submodule2_dir = path.join("submodule2_source");
        std::fs::create_dir_all(&submodule2_dir).expect("failed to create submodule2 source dir");
        run_git(&submodule2_dir, &["init"]);
        run_git(&submodule2_dir, &["config", "user.email", "test@example.com"]);
        run_git(&submodule2_dir, &["config", "user.name", "Test User"]);
        std::fs::write(submodule2_dir.join("util.rs"), "pub fn func2() {}\n")
            .expect("failed to write submodule2 util.rs");
        run_git(&submodule2_dir, &["add", "."]);
        run_git(&submodule2_dir, &["commit", "-m", "Submodule2 initial commit"]);

        let main_repo_dir = path.join("main_repo");
        std::fs::create_dir_all(&main_repo_dir).expect("failed to create main repo dir");
        run_git(&main_repo_dir, &["init"]);
        run_git(&main_repo_dir, &["config", "user.email", "test@example.com"]);
        run_git(&main_repo_dir, &["config", "user.name", "Test User"]);
        std::fs::write(main_repo_dir.join("main.rs"), "fn main() {}\n")
            .expect("failed to write main.rs");
        run_git(&main_repo_dir, &["add", "."]);
        run_git(&main_repo_dir, &["commit", "-m", "Main repo initial commit"]);

        let submodule1_path_str = submodule1_dir.to_str().expect("invalid submodule1 path");
        run_git(
            &main_repo_dir,
            &["submodule", "add", submodule1_path_str, "libs/first"],
        );

        let submodule2_path_str = submodule2_dir.to_str().expect("invalid submodule2 path");
        run_git(
            &main_repo_dir,
            &["submodule", "add", submodule2_path_str, "libs/second"],
        );

        run_git(&main_repo_dir, &["commit", "-m", "Add multiple submodules"]);

        Self {
            dir,
            path: main_repo_dir,
        }
    }

    pub fn with_inactive_submodule() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        let submodule_dir = path.join("submodule_source");
        std::fs::create_dir_all(&submodule_dir).expect("failed to create submodule source dir");
        run_git(&submodule_dir, &["init"]);
        run_git(&submodule_dir, &["config", "user.email", "test@example.com"]);
        run_git(&submodule_dir, &["config", "user.name", "Test User"]);
        std::fs::write(submodule_dir.join("lib.rs"), "pub fn inactive_func() {}\n")
            .expect("failed to write submodule lib.rs");
        run_git(&submodule_dir, &["add", "."]);
        run_git(&submodule_dir, &["commit", "-m", "Submodule initial commit"]);

        let main_repo_dir = path.join("main_repo");
        std::fs::create_dir_all(&main_repo_dir).expect("failed to create main repo dir");
        run_git(&main_repo_dir, &["init"]);
        run_git(&main_repo_dir, &["config", "user.email", "test@example.com"]);
        run_git(&main_repo_dir, &["config", "user.name", "Test User"]);
        std::fs::write(main_repo_dir.join("main.rs"), "fn main() {}\n")
            .expect("failed to write main.rs");
        run_git(&main_repo_dir, &["add", "."]);
        run_git(&main_repo_dir, &["commit", "-m", "Main repo initial commit"]);

        let submodule_path_str = submodule_dir.to_str().expect("invalid submodule path");
        run_git(
            &main_repo_dir,
            &["submodule", "add", submodule_path_str, "vendor/inactive"],
        );
        run_git(&main_repo_dir, &["commit", "-m", "Add submodule"]);

        run_git(
            &main_repo_dir,
            &["config", "submodule.vendor/inactive.active", "false"],
        );

        Self {
            dir,
            path: main_repo_dir,
        }
    }

    pub fn with_nested_submodule_path() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        let submodule_dir = path.join("submodule_source");
        std::fs::create_dir_all(&submodule_dir).expect("failed to create submodule source dir");
        run_git(&submodule_dir, &["init"]);
        run_git(&submodule_dir, &["config", "user.email", "test@example.com"]);
        run_git(&submodule_dir, &["config", "user.name", "Test User"]);
        std::fs::write(submodule_dir.join("lib.rs"), "pub fn nested_func() {}\n")
            .expect("failed to write submodule lib.rs");
        run_git(&submodule_dir, &["add", "."]);
        run_git(&submodule_dir, &["commit", "-m", "Submodule initial commit"]);

        let main_repo_dir = path.join("main_repo");
        std::fs::create_dir_all(&main_repo_dir).expect("failed to create main repo dir");
        run_git(&main_repo_dir, &["init"]);
        run_git(&main_repo_dir, &["config", "user.email", "test@example.com"]);
        run_git(&main_repo_dir, &["config", "user.name", "Test User"]);
        std::fs::write(main_repo_dir.join("main.rs"), "fn main() {}\n")
            .expect("failed to write main.rs");
        run_git(&main_repo_dir, &["add", "."]);
        run_git(&main_repo_dir, &["commit", "-m", "Main repo initial commit"]);

        let submodule_path_str = submodule_dir.to_str().expect("invalid submodule path");
        run_git(
            &main_repo_dir,
            &["submodule", "add", submodule_path_str, "deep/nested/path/submodule"],
        );
        run_git(&main_repo_dir, &["commit", "-m", "Add deeply nested submodule"]);

        Self {
            dir,
            path: main_repo_dir,
        }
    }

    pub fn with_submodule_without_url() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        let submodule_dir = path.join("submodule_source");
        std::fs::create_dir_all(&submodule_dir).expect("failed to create submodule source dir");
        run_git(&submodule_dir, &["init"]);
        run_git(&submodule_dir, &["config", "user.email", "test@example.com"]);
        run_git(&submodule_dir, &["config", "user.name", "Test User"]);
        std::fs::write(submodule_dir.join("lib.rs"), "pub fn func() {}\n")
            .expect("failed to write submodule lib.rs");
        run_git(&submodule_dir, &["add", "."]);
        run_git(&submodule_dir, &["commit", "-m", "Submodule initial commit"]);

        let main_repo_dir = path.join("main_repo");
        std::fs::create_dir_all(&main_repo_dir).expect("failed to create main repo dir");
        run_git(&main_repo_dir, &["init"]);
        run_git(&main_repo_dir, &["config", "user.email", "test@example.com"]);
        run_git(&main_repo_dir, &["config", "user.name", "Test User"]);
        std::fs::write(main_repo_dir.join("main.rs"), "fn main() {}\n")
            .expect("failed to write main.rs");
        run_git(&main_repo_dir, &["add", "."]);
        run_git(&main_repo_dir, &["commit", "-m", "Main repo initial commit"]);

        let submodule_path_str = submodule_dir.to_str().expect("invalid submodule path");
        run_git(
            &main_repo_dir,
            &["submodule", "add", submodule_path_str, "vendor/nourl"],
        );
        run_git(&main_repo_dir, &["commit", "-m", "Add submodule"]);

        let gitmodules_path = main_repo_dir.join(".gitmodules");
        let gitmodules_content = std::fs::read_to_string(&gitmodules_path)
            .expect("failed to read .gitmodules");
        let modified_content = gitmodules_content
            .lines()
            .filter(|line| !line.trim().starts_with("url"))
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&gitmodules_path, modified_content)
            .expect("failed to write .gitmodules");

        run_git(&main_repo_dir, &["add", ".gitmodules"]);
        run_git(&main_repo_dir, &["commit", "-m", "Remove url from .gitmodules"]);

        Self {
            dir,
            path: main_repo_dir,
        }
    }

    pub fn with_submodule_not_initialized() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::write(path.join("main.rs"), "fn main() {}\n")
            .expect("failed to write main.rs");

        std::fs::write(
            path.join(".gitmodules"),
            "[submodule \"vendor/uninit\"]\n\tpath = vendor/uninit\n\turl = https://example.com/repo.git\n",
        )
        .expect("failed to write .gitmodules");

        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit with uninitialized submodule"]);

        Self { dir, path }
    }

    pub fn with_submodule_missing_path() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::write(path.join("main.rs"), "fn main() {}\n")
            .expect("failed to write main.rs");

        std::fs::write(
            path.join(".gitmodules"),
            "[submodule \"vendor/broken\"]\n\turl = https://example.com/repo.git\n",
        )
        .expect("failed to write .gitmodules");

        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Commit with submodule missing path"]);

        Self { dir, path }
    }

    pub fn with_submodule_absolute_path() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::write(path.join("main.rs"), "fn main() {}\n")
            .expect("failed to write main.rs");

        std::fs::write(
            path.join(".gitmodules"),
            "[submodule \"vendor/absolute\"]\n\tpath = /absolute/path/to/submodule\n\turl = https://example.com/repo.git\n",
        )
        .expect("failed to write .gitmodules");

        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Commit with submodule absolute path"]);

        Self { dir, path }
    }

    pub fn with_corrupt_gitmodules() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::write(path.join("main.rs"), "fn main() {}\n")
            .expect("failed to write main.rs");

        std::fs::write(
            path.join(".gitmodules"),
            "[submodule \"vendor/corrupt\"\n\tpath = vendor/corrupt\n\turl = https://example.com/repo.git\n",
        )
        .expect("failed to write .gitmodules");

        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Commit with corrupt gitmodules"]);

        Self { dir, path }
    }

    pub fn with_submodule_invalid_active() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        let submodule_dir = path.join("submodule_source");
        std::fs::create_dir_all(&submodule_dir).expect("failed to create submodule source dir");

        run_git(&submodule_dir, &["init"]);
        run_git(&submodule_dir, &["config", "user.email", "test@example.com"]);
        run_git(&submodule_dir, &["config", "user.name", "Test User"]);

        std::fs::write(submodule_dir.join("lib.rs"), "pub fn func() {}\n")
            .expect("failed to write submodule lib.rs");

        run_git(&submodule_dir, &["add", "."]);
        run_git(&submodule_dir, &["commit", "-m", "Submodule initial commit"]);

        let main_repo_dir = path.join("main_repo");
        std::fs::create_dir_all(&main_repo_dir).expect("failed to create main repo dir");

        run_git(&main_repo_dir, &["init"]);
        run_git(&main_repo_dir, &["config", "user.email", "test@example.com"]);
        run_git(&main_repo_dir, &["config", "user.name", "Test User"]);

        std::fs::write(main_repo_dir.join("main.rs"), "fn main() {}\n")
            .expect("failed to write main.rs");

        run_git(&main_repo_dir, &["add", "."]);
        run_git(&main_repo_dir, &["commit", "-m", "Main repo initial commit"]);

        let submodule_path_str = submodule_dir.to_str().expect("invalid submodule path");
        run_git(
            &main_repo_dir,
            &["submodule", "add", submodule_path_str, "vendor/invalid"],
        );
        run_git(&main_repo_dir, &["commit", "-m", "Add submodule"]);

        run_git(
            &main_repo_dir,
            &["config", "submodule.vendor/invalid.active", "not-a-boolean"],
        );

        Self {
            dir,
            path: main_repo_dir,
        }
    }

    pub fn with_submodule_corrupt_head() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        let submodule_dir = path.join("submodule_source");
        std::fs::create_dir_all(&submodule_dir).expect("failed to create submodule source dir");

        run_git(&submodule_dir, &["init"]);
        run_git(&submodule_dir, &["config", "user.email", "test@example.com"]);
        run_git(&submodule_dir, &["config", "user.name", "Test User"]);

        std::fs::write(submodule_dir.join("lib.rs"), "pub fn func() {}\n")
            .expect("failed to write submodule lib.rs");

        run_git(&submodule_dir, &["add", "."]);
        run_git(&submodule_dir, &["commit", "-m", "Submodule initial commit"]);

        let main_repo_dir = path.join("main_repo");
        std::fs::create_dir_all(&main_repo_dir).expect("failed to create main repo dir");

        run_git(&main_repo_dir, &["init"]);
        run_git(&main_repo_dir, &["config", "user.email", "test@example.com"]);
        run_git(&main_repo_dir, &["config", "user.name", "Test User"]);

        std::fs::write(main_repo_dir.join("main.rs"), "fn main() {}\n")
            .expect("failed to write main.rs");

        run_git(&main_repo_dir, &["add", "."]);
        run_git(&main_repo_dir, &["commit", "-m", "Main repo initial commit"]);

        let submodule_path_str = submodule_dir.to_str().expect("invalid submodule path");
        run_git(
            &main_repo_dir,
            &["submodule", "add", submodule_path_str, "vendor/corrupt"],
        );
        run_git(&main_repo_dir, &["commit", "-m", "Add submodule"]);

        let head_path = main_repo_dir.join(".git/HEAD");
        std::fs::write(&head_path, "ref: refs/heads/nonexistent\n")
            .expect("failed to corrupt HEAD");

        Self {
            dir,
            path: main_repo_dir,
        }
    }

    pub fn with_submodule_corrupt_index() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        let submodule_dir = path.join("submodule_source");
        std::fs::create_dir_all(&submodule_dir).expect("failed to create submodule source dir");

        run_git(&submodule_dir, &["init"]);
        run_git(&submodule_dir, &["config", "user.email", "test@example.com"]);
        run_git(&submodule_dir, &["config", "user.name", "Test User"]);

        std::fs::write(submodule_dir.join("lib.rs"), "pub fn func() {}\n")
            .expect("failed to write submodule lib.rs");

        run_git(&submodule_dir, &["add", "."]);
        run_git(&submodule_dir, &["commit", "-m", "Submodule initial commit"]);

        let main_repo_dir = path.join("main_repo");
        std::fs::create_dir_all(&main_repo_dir).expect("failed to create main repo dir");

        run_git(&main_repo_dir, &["init"]);
        run_git(&main_repo_dir, &["config", "user.email", "test@example.com"]);
        run_git(&main_repo_dir, &["config", "user.name", "Test User"]);

        std::fs::write(main_repo_dir.join("main.rs"), "fn main() {}\n")
            .expect("failed to write main.rs");

        run_git(&main_repo_dir, &["add", "."]);
        run_git(&main_repo_dir, &["commit", "-m", "Main repo initial commit"]);

        let submodule_path_str = submodule_dir.to_str().expect("invalid submodule path");
        run_git(
            &main_repo_dir,
            &["submodule", "add", submodule_path_str, "vendor/corrupt"],
        );
        run_git(&main_repo_dir, &["commit", "-m", "Add submodule"]);

        let index_path = main_repo_dir.join(".git/index");
        std::fs::write(&index_path, b"DIRC\x00\x00\x00\x02\x00\x00\x00\x00corrupted data")
            .expect("failed to corrupt index");

        Self {
            dir,
            path: main_repo_dir,
        }
    }

    pub fn with_attributes() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::create_dir_all(path.join("src")).expect("failed to create src dir");
        std::fs::create_dir_all(path.join("build")).expect("failed to create build dir");
        std::fs::create_dir_all(path.join("logs")).expect("failed to create logs dir");

        std::fs::write(
            path.join(".gitignore"),
            r#"# Build artifacts
/target/
/build/
*.o
*.a

# IDE
.idea/
.vscode/
*.swp

# Logs
*.log
/logs/

# Environment
.env
.env.local

# OS files
.DS_Store
Thumbs.db
"#,
        )
        .expect("failed to write .gitignore");

        std::fs::write(
            path.join(".gitattributes"),
            r#"# Auto detect text files
* text=auto

# Rust source files
*.rs text diff=rust

# Documentation
*.md text diff=markdown

# Binary files
*.png binary
*.jpg binary
*.gif binary
*.ico binary
*.bin binary

# Lock files
Cargo.lock text -diff

# Shell scripts
*.sh text eol=lf

# Windows batch files
*.bat text eol=crlf
*.cmd text eol=crlf
"#,
        )
        .expect("failed to write .gitattributes");

        std::fs::write(path.join("README.md"), "# Project with Attributes\n")
            .expect("failed to write README.md");

        std::fs::write(
            path.join("src/main.rs"),
            r#"fn main() {
    println!("Hello!");
}
"#,
        )
        .expect("failed to write main.rs");

        std::fs::write(path.join("build/output.o"), vec![0u8; 100])
            .expect("failed to write build artifact");
        std::fs::write(path.join("logs/app.log"), "2024-01-01 INFO: Started\n")
            .expect("failed to write log");
        std::fs::write(path.join(".env"), "SECRET_KEY=abc123\n").expect("failed to write .env");

        let png_header: Vec<u8> = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
            0x44, 0x52,
        ];
        std::fs::write(path.join("icon.png"), &png_header).expect("failed to write png");

        std::fs::write(path.join("setup.sh"), "#!/bin/bash\necho 'Setup'\n")
            .expect("failed to write shell script");

        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit with gitignore and gitattributes"]);

        Self { dir, path }
    }

    pub fn empty() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        Self { dir, path }
    }

    pub fn bare() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init", "--bare"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        let work_dir = TempDir::new().expect("failed to create work temp dir");
        let work_path = work_dir.path().to_path_buf();

        run_git(&work_path, &["clone", path.to_str().unwrap(), "."]);
        run_git(&work_path, &["config", "user.email", "test@example.com"]);
        run_git(&work_path, &["config", "user.name", "Test User"]);

        std::fs::write(work_path.join("README.md"), "# Bare Test Repository\n\nThis is a test.\n")
            .expect("failed to write README.md");

        run_git(&work_path, &["add", "."]);
        run_git(&work_path, &["commit", "-m", "Initial commit"]);
        run_git(&work_path, &["push", "origin", "HEAD:main"]);

        Self { dir, path }
    }

    pub fn without_index() -> Self {
        let repo = Self::new();

        let index_path = repo.path.join(".git").join("index");
        if index_path.exists() {
            std::fs::remove_file(&index_path).expect("failed to remove index");
        }

        repo
    }

    pub fn with_branches() -> Self {
        let repo = Self::new();

        run_git(&repo.path, &["checkout", "-b", "feature-a"]);
        std::fs::write(repo.path.join("feature-a.txt"), "Feature A content")
            .expect("failed to write feature-a.txt");
        run_git(&repo.path, &["add", "."]);
        run_git(&repo.path, &["commit", "-m", "Add feature A"]);

        run_git(&repo.path, &["checkout", "-b", "feature-b"]);
        std::fs::write(repo.path.join("feature-b.txt"), "Feature B content")
            .expect("failed to write feature-b.txt");
        run_git(&repo.path, &["add", "."]);
        run_git(&repo.path, &["commit", "-m", "Add feature B"]);

        run_git(&repo.path, &["checkout", "main"]);

        repo
    }

    pub fn with_tags() -> Self {
        let repo = Self::new();

        run_git(&repo.path, &["tag", "v1.0.0"]);

        std::fs::write(repo.path.join("version.txt"), "1.1.0")
            .expect("failed to write version.txt");
        run_git(&repo.path, &["add", "."]);
        run_git(&repo.path, &["commit", "-m", "Version 1.1.0"]);

        run_git(&repo.path, &["tag", "-a", "v1.1.0", "-m", "Release v1.1.0"]);

        repo
    }

    pub fn with_merge_commits() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::write(path.join("base.txt"), "base content\n").expect("failed to write base.txt");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit"]);

        run_git(&path, &["checkout", "-b", "feature"]);
        std::fs::write(path.join("feature.txt"), "feature content\n")
            .expect("failed to write feature.txt");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Add feature"]);

        run_git(&path, &["checkout", "main"]);
        std::fs::write(path.join("main-only.txt"), "main content\n")
            .expect("failed to write main-only.txt");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Add main-only file"]);

        run_git(&path, &["merge", "feature", "-m", "Merge feature into main"]);

        Self { dir, path }
    }

    pub fn with_merge_first_parent_changed() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::write(path.join("shared.txt"), "initial content\n")
            .expect("failed to write shared.txt");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit"]);

        run_git(&path, &["checkout", "-b", "feature"]);
        std::fs::write(path.join("feature-only.txt"), "feature content\n")
            .expect("failed to write feature-only.txt");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Add feature file"]);

        run_git(&path, &["checkout", "main"]);
        std::fs::write(path.join("shared.txt"), "main modified content\n")
            .expect("failed to modify shared.txt on main");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Modify shared.txt on main"]);

        run_git(&path, &["merge", "feature", "-m", "Merge feature into main"]);

        Self { dir, path }
    }

    pub fn with_orphan_branch() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::write(path.join("main.txt"), "main content\n").expect("failed to write main.txt");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Main branch commit"]);

        run_git(&path, &["checkout", "--orphan", "orphan"]);
        run_git(&path, &["rm", "-rf", "."]);
        std::fs::write(path.join("orphan.txt"), "orphan content\n")
            .expect("failed to write orphan.txt");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Orphan branch commit"]);

        Self { dir, path }
    }

    pub fn with_deep_paths() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::create_dir_all(path.join("a/b/c")).expect("failed to create nested dirs");
        std::fs::write(path.join("root.txt"), "root\n").expect("failed to write root.txt");
        std::fs::write(path.join("a/level1.txt"), "level1\n").expect("failed to write level1.txt");
        std::fs::write(path.join("a/b/level2.txt"), "level2\n")
            .expect("failed to write level2.txt");
        std::fs::write(path.join("a/b/c/level3.txt"), "level3\n")
            .expect("failed to write level3.txt");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit with deep paths"]);

        std::fs::write(path.join("a/b/c/level3.txt"), "level3 modified\n")
            .expect("failed to modify level3.txt");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Modify deeply nested file"]);

        std::fs::write(path.join("a/level1.txt"), "level1 modified\n")
            .expect("failed to modify level1.txt");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Modify level1 file"]);

        std::fs::remove_file(path.join("a/b/level2.txt")).expect("failed to remove level2.txt");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Delete level2 file"]);

        Self { dir, path }
    }

    pub fn with_detached_head() -> Self {
        let repo = Self::new();

        std::fs::write(repo.path.join("second.txt"), "Second commit content")
            .expect("failed to write second.txt");
        run_git(&repo.path, &["add", "."]);
        run_git(&repo.path, &["commit", "-m", "Second commit"]);

        let output = Command::new("git")
            .args(["rev-parse", "HEAD~1"])
            .current_dir(&repo.path)
            .output()
            .expect("failed to get commit hash");
        let commit_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();

        run_git(&repo.path, &["checkout", &commit_hash]);

        repo
    }

    pub fn with_symbolic_ref() -> Self {
        let repo = Self::new();

        run_git(&repo.path, &["checkout", "-b", "develop"]);
        std::fs::write(repo.path.join("develop.txt"), "develop content")
            .expect("failed to write develop.txt");
        run_git(&repo.path, &["add", "."]);
        run_git(&repo.path, &["commit", "-m", "Develop commit"]);

        run_git(&repo.path, &["symbolic-ref", "refs/heads/alias", "refs/heads/develop"]);

        repo
    }

    pub fn with_unborn_nondefault_branch() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init", "--initial-branch=custom-main"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        Self { dir, path }
    }

    pub fn with_deep_nesting() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::create_dir_all(path.join("a/b/c")).expect("failed to create nested dirs");

        std::fs::write(path.join("root.txt"), "root file").expect("failed to write root.txt");
        std::fs::write(path.join("a/level1.txt"), "level 1 file").expect("failed to write level1.txt");
        std::fs::write(path.join("a/b/level2.txt"), "level 2 file").expect("failed to write level2.txt");
        std::fs::write(path.join("a/b/c/deep.txt"), "deep file").expect("failed to write deep.txt");

        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit with deep nesting"]);

        Self { dir, path }
    }

    pub fn single_file() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::write(path.join("single.txt"), "single file content").expect("failed to write single.txt");

        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Single file commit"]);

        Self { dir, path }
    }

    pub fn with_deep_single_path() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::create_dir_all(path.join("level1/level2/level3/level4")).expect("failed to create dirs");
        std::fs::write(path.join("level1/level2/level3/level4/deepest.txt"), "deepest file")
            .expect("failed to write deepest.txt");

        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Deep single path commit"]);

        Self { dir, path }
    }

    pub fn with_many_siblings() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::write(path.join("file1.txt"), "file 1").expect("failed to write file1.txt");
        std::fs::write(path.join("file2.txt"), "file 2").expect("failed to write file2.txt");
        std::fs::write(path.join("file3.txt"), "file 3").expect("failed to write file3.txt");

        std::fs::create_dir_all(path.join("dir1")).expect("failed to create dir1");
        std::fs::create_dir_all(path.join("dir2")).expect("failed to create dir2");

        std::fs::write(path.join("dir1/nested1.txt"), "nested 1").expect("failed to write nested1.txt");
        std::fs::write(path.join("dir2/nested2.txt"), "nested 2").expect("failed to write nested2.txt");

        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Many siblings commit"]);

        Self { dir, path }
    }

    pub fn with_multiple_nested_dirs() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::create_dir_all(path.join("a/aa")).expect("failed to create a/aa");
        std::fs::create_dir_all(path.join("b/bb")).expect("failed to create b/bb");

        std::fs::write(path.join("root.txt"), "root").expect("failed to write root.txt");
        std::fs::write(path.join("a/a_file.txt"), "a file").expect("failed to write a_file.txt");
        std::fs::write(path.join("a/aa/aa_file.txt"), "aa file").expect("failed to write aa_file.txt");
        std::fs::write(path.join("b/b_file.txt"), "b file").expect("failed to write b_file.txt");
        std::fs::write(path.join("b/bb/bb_file.txt"), "bb file").expect("failed to write bb_file.txt");

        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Multiple nested dirs commit"]);

        Self { dir, path }
    }

    pub fn with_empty_subdirs() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::create_dir_all(path.join("empty_dir")).expect("failed to create empty_dir");
        std::fs::write(path.join("empty_dir/.gitkeep"), "").expect("failed to write .gitkeep");
        std::fs::write(path.join("file.txt"), "file content").expect("failed to write file.txt");

        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Commit with empty-ish dir"]);

        Self { dir, path }
    }

    pub fn with_packed_objects() -> Self {
        let repo = Self::new();

        run_git(&repo.path, &["gc", "--aggressive"]);

        repo
    }

    pub fn with_only_packed_objects() -> Self {
        let repo = Self::new();

        run_git(&repo.path, &["gc", "--aggressive", "--prune=now"]);

        let objects_dir = repo.path.join(".git/objects");
        for entry in std::fs::read_dir(&objects_dir).expect("read objects dir") {
            let entry = entry.expect("read entry");
            let name = entry.file_name().to_string_lossy().to_string();
            if name.len() == 2 && name.chars().all(|c| c.is_ascii_hexdigit()) {
                std::fs::remove_dir_all(entry.path()).expect("remove loose object dir");
            }
        }

        repo
    }

    pub fn with_corrupted_pack() -> Self {
        let repo = Self::with_only_packed_objects();

        let pack_dir = repo.path.join(".git/objects/pack");
        if let Ok(entries) = std::fs::read_dir(&pack_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().map(|e| e == "pack").unwrap_or(false) {
                        let data = std::fs::read(&path).expect("read pack file");
                        let mut corrupted = data;
                        if corrupted.len() > 100 {
                            corrupted[50] ^= 0xFF;
                            corrupted[51] ^= 0xFF;
                            corrupted[52] ^= 0xFF;
                        }
                        std::fs::write(&path, corrupted).expect("write corrupted pack");
                        break;
                    }
                }
            }
        }

        repo
    }

    pub fn with_corrupted_loose_object() -> Self {
        let repo = Self::new();

        let objects_dir = repo.path.join(".git/objects");
        for entry in std::fs::read_dir(&objects_dir).expect("read objects dir") {
            let entry = entry.expect("read entry");
            let name = entry.file_name().to_string_lossy().to_string();
            if name.len() == 2 && name.chars().all(|c| c.is_ascii_hexdigit()) {
                if let Ok(inner_entries) = std::fs::read_dir(entry.path()) {
                    for inner in inner_entries {
                        if let Ok(inner) = inner {
                            let path = inner.path();
                            #[cfg(unix)]
                            {
                                use std::os::unix::fs::PermissionsExt;
                                let mut perms = std::fs::metadata(&path)
                                    .expect("read metadata")
                                    .permissions();
                                perms.set_mode(0o644);
                                std::fs::set_permissions(&path, perms).expect("set permissions");
                            }
                            std::fs::write(&path, b"corrupted data").expect("corrupt object");
                            return repo;
                        }
                    }
                }
            }
        }

        repo
    }

    pub fn with_unchanged_nested_file() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::create_dir_all(path.join("nested/deep")).expect("failed to create nested dirs");
        std::fs::write(path.join("nested/deep/file.txt"), "nested content\n")
            .expect("failed to write nested file");
        std::fs::write(path.join("other.txt"), "other content\n")
            .expect("failed to write other file");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit with nested and other file"]);

        std::fs::write(path.join("other.txt"), "modified other content\n")
            .expect("failed to modify other file");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Modify only the other file"]);

        Self { dir, path }
    }

    pub fn with_empty_commits() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::write(path.join("file.txt"), "content\n")
            .expect("failed to write file");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit"]);

        run_git(&path, &["commit", "--allow-empty", "-m", "Empty commit 1"]);
        run_git(&path, &["commit", "--allow-empty", "-m", "Empty commit 2"]);

        std::fs::write(path.join("file.txt"), "modified content\n")
            .expect("failed to modify file");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Real commit after empty ones"]);

        Self { dir, path }
    }

    pub fn with_corrupt_tree_reference() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::create_dir_all(path.join("subdir")).expect("failed to create subdir");
        std::fs::write(path.join("root.txt"), "root content\n").expect("failed to write root.txt");
        std::fs::write(path.join("subdir/nested.txt"), "nested content\n").expect("failed to write nested.txt");

        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit"]);

        let subdir_tree_id_output = Command::new("git")
            .current_dir(&path)
            .args(["rev-parse", "HEAD:subdir"])
            .output()
            .expect("failed to get subdir tree id");
        let subdir_tree_id = String::from_utf8_lossy(&subdir_tree_id_output.stdout).trim().to_string();

        let object_path = path.join(".git/objects")
            .join(&subdir_tree_id[..2])
            .join(&subdir_tree_id[2..]);

        if object_path.exists() {
            std::fs::remove_file(&object_path).expect("failed to remove tree object");
        }

        Self { dir, path }
    }

    pub fn with_merge_first_parent_path_changed() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        run_git(&path, &["init"]);
        run_git(&path, &["config", "user.email", "test@example.com"]);
        run_git(&path, &["config", "user.name", "Test User"]);

        std::fs::write(path.join("file.txt"), "initial\n").expect("failed to write file.txt");
        std::fs::write(path.join("base.txt"), "base\n").expect("failed to write base.txt");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Initial commit"]);

        run_git(&path, &["checkout", "-b", "branch-b"]);
        std::fs::write(path.join("branch-b.txt"), "branch-b content\n").expect("failed to write branch-b.txt");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Add branch-b.txt on branch-b"]);

        run_git(&path, &["checkout", "main"]);
        std::fs::write(path.join("file.txt"), "modified on main\n").expect("failed to modify file.txt on main");
        run_git(&path, &["add", "."]);
        run_git(&path, &["commit", "-m", "Modify file.txt on main"]);

        run_git(&path, &["merge", "branch-b", "-m", "Merge branch-b into main"]);

        Self { dir, path }
    }

    #[allow(dead_code)]
    pub fn git_output(&self, args: &[&str]) -> String {
        let output = Command::new("git")
            .current_dir(&self.path)
            .args(args)
            .output()
            .expect("failed to execute git command");

        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }
}

impl Default for TestRepo {
    fn default() -> Self {
        Self::new()
    }
}

fn run_git(dir: &PathBuf, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(dir)
        .args(args)
        .env("GIT_AUTHOR_DATE", "2024-01-15T10:00:00")
        .env("GIT_COMMITTER_DATE", "2024-01-15T10:00:00")
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "protocol.file.allow")
        .env("GIT_CONFIG_VALUE_0", "always")
        .output()
        .expect("failed to execute git command");

    if !output.status.success() {
        panic!(
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_repo() {
        let repo = TestRepo::new();
        assert!(repo.path.join(".git").exists());
        assert!(repo.path.join("README.md").exists());
        assert!(repo.path.join("src/main.rs").exists());
        assert!(repo.path.join("src/lib.rs").exists());
        assert!(repo.path.join("docs/guide.md").exists());
        assert!(repo.path.join("data.bin").exists());
    }

    #[test]
    fn test_with_history() {
        let repo = TestRepo::with_history();
        assert!(repo.path.join(".git").exists());

        let output = Command::new("git")
            .current_dir(&repo.path)
            .args(["log", "--oneline"])
            .output()
            .expect("failed to get git log");

        let log = String::from_utf8_lossy(&output.stdout);
        let commit_count = log.lines().count();
        assert!(commit_count >= 5, "Expected at least 5 commits, got {}", commit_count);
    }

    #[test]
    fn test_with_submodules() {
        let repo = TestRepo::with_submodules();
        assert!(repo.path.join(".git").exists());
        assert!(repo.path.join(".gitmodules").exists());
        assert!(repo.path.join("vendor/submodule").exists());
    }

    #[test]
    fn test_with_attributes() {
        let repo = TestRepo::with_attributes();
        assert!(repo.path.join(".git").exists());
        assert!(repo.path.join(".gitignore").exists());
        assert!(repo.path.join(".gitattributes").exists());

        let output = Command::new("git")
            .current_dir(&repo.path)
            .args(["status", "--porcelain"])
            .output()
            .expect("failed to get git status");

        let status = String::from_utf8_lossy(&output.stdout);
        assert!(!status.contains("build/output.o"), "build artifacts should be ignored");
        assert!(!status.contains("logs/app.log"), "logs should be ignored");
        assert!(!status.contains(".env"), ".env should be ignored");
    }
}
