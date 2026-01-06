use anyhow::Result;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

const HOOK_SCRIPT: &str = r#"#!/bin/bash
set -e

# helix-decisions pre-commit hook
# Prevents modifications to accepted decisions without amendment pattern

if [[ "$HELIX_DECISIONS_SKIP_HOOKS" == "1" ]]; then
    exit 0
fi

DECISIONS_DIR=".decisions"
if [[ ! -d "$DECISIONS_DIR" ]]; then
    exit 0
fi

# Get modified .md files in .decisions/
modified=$(git diff --cached --name-only -- "$DECISIONS_DIR"/*.md 2>/dev/null || true)

if [[ -z "$modified" ]]; then
    exit 0
fi

# Use helix-decisions to validate changes
# If helix-decisions is not installed, skip validation with warning
if ! command -v helix-decisions &> /dev/null; then
    echo "Warning: helix-decisions not found, skipping immutability check"
    exit 0
fi

helix-decisions validate-hook --files "$modified"
"#;

pub fn generate_hook_script() -> &'static str {
    HOOK_SCRIPT
}

pub fn install_hook(git_root: &Path, force: bool) -> Result<()> {
    let hooks_dir = git_root.join(".git").join("hooks");
    let hook_path = hooks_dir.join("pre-commit");

    if !hooks_dir.exists() {
        anyhow::bail!("Not a git repository: .git/hooks not found");
    }

    if hook_path.exists() && !force {
        anyhow::bail!(
            "pre-commit hook already exists at {}\n\
             Use --force to overwrite, or manually integrate the hook.",
            hook_path.display()
        );
    }

    fs::write(&hook_path, HOOK_SCRIPT)?;

    let mut perms = fs::metadata(&hook_path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&hook_path, perms)?;

    Ok(())
}

pub fn print_hook_warning() {
    let warning = r#"
This command will:
  * Create .git/hooks/pre-commit in this repository
  * Block commits that modify accepted decisions (status: accepted)
  * Allow new decisions with 'amends: [id]' references
  * This only affects THIS repository

To bypass this hook on a specific commit, use:
  git commit --no-verify

To disable the hook entirely, delete .git/hooks/pre-commit
"#;
    eprintln!("{warning}");
}

pub fn confirm_installation() -> Result<bool> {
    eprint!("Continue? [y/N] ");
    io::stderr().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().eq_ignore_ascii_case("y"))
}

pub fn uninstall_hook(git_root: &Path) -> Result<bool> {
    let hook_path = git_root.join(".git").join("hooks").join("pre-commit");

    if !hook_path.exists() {
        return Ok(false);
    }

    let content = fs::read_to_string(&hook_path)?;
    if !content.contains("helix-decisions") {
        anyhow::bail!(
            "pre-commit hook exists but was not created by helix-decisions.\n\
             Manually remove it if you want to uninstall."
        );
    }

    fs::remove_file(&hook_path)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_git_repo() -> TempDir {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join(".git/hooks")).unwrap();
        temp
    }

    #[test]
    fn test_install_hook() {
        let temp = setup_git_repo();
        install_hook(temp.path(), false).unwrap();

        let hook_path = temp.path().join(".git/hooks/pre-commit");
        assert!(hook_path.exists());

        let content = fs::read_to_string(&hook_path).unwrap();
        assert!(content.contains("helix-decisions"));

        let perms = fs::metadata(&hook_path).unwrap().permissions();
        assert_eq!(perms.mode() & 0o777, 0o755);
    }

    #[test]
    fn test_install_hook_already_exists() {
        let temp = setup_git_repo();
        let hook_path = temp.path().join(".git/hooks/pre-commit");
        fs::write(&hook_path, "existing hook").unwrap();

        let result = install_hook(temp.path(), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_install_hook_force() {
        let temp = setup_git_repo();
        let hook_path = temp.path().join(".git/hooks/pre-commit");
        fs::write(&hook_path, "existing hook").unwrap();

        install_hook(temp.path(), true).unwrap();

        let content = fs::read_to_string(&hook_path).unwrap();
        assert!(content.contains("helix-decisions"));
    }

    #[test]
    fn test_uninstall_hook() {
        let temp = setup_git_repo();
        install_hook(temp.path(), false).unwrap();

        let removed = uninstall_hook(temp.path()).unwrap();
        assert!(removed);

        let hook_path = temp.path().join(".git/hooks/pre-commit");
        assert!(!hook_path.exists());
    }

    #[test]
    fn test_uninstall_hook_not_exists() {
        let temp = setup_git_repo();
        let removed = uninstall_hook(temp.path()).unwrap();
        assert!(!removed);
    }
}
