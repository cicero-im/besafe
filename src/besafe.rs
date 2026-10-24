use crate::utils::{code_to_enum, Status};
use std::process::{exit, Command};

const FORBIDDEN_EXACT_FILES: [&str; 2] = [".stripe-config", ".env.local"];

enum FileDecision {
    Block(String),
    Warn(String),
}

fn is_gitignored(file_name: &str) -> bool {
    let mut check_ignore = Command::new("git");
    check_ignore.arg("check-ignore").arg("-q").arg(file_name);
    check_ignore.status().map(|s| s.success()).unwrap_or(false)
}

fn file_basename(file_path: &str) -> &str {
    file_path.rsplit('/').next().unwrap_or(file_path)
}

fn is_forbidden_file(file_path: &str) -> bool {
    let file_name = file_basename(file_path);
    if FORBIDDEN_EXACT_FILES.contains(&file_name) {
        return true;
    }

    file_name.starts_with(".env") && file_name != ".env.example"
}

fn evaluate_file(file_name: &str, status: Status, ignored: bool) -> Option<FileDecision> {
    if !is_forbidden_file(file_name) {
        return None;
    }

    match status {
        Status::Staged => Some(FileDecision::Block(format!(
            "[besafe] {} file is staged! Unstage it and add it to .gitignore before committing.",
            file_name
        ))),
        Status::Modified => Some(FileDecision::Block(format!(
            "[besafe] {} file has changes and is not in .gitignore! Remove it and add it to .gitignore before committing.",
            file_name
        ))),
        Status::StagedModified => Some(FileDecision::Block(format!(
            "[besafe] {} file staged! Unstage it and add it to .gitignore before committing.",
            file_name
        ))),
        Status::Untracked => {
            if ignored {
                None
            } else {
                Some(FileDecision::Warn(format!(
                    "[besafe] {} file is not in .gitignore! Consider adding it.",
                    file_name
                )))
            }
        }
        Status::Unknown => Some(FileDecision::Warn(format!(
            "[besafe] {} file found in git history but status cannot be determined! Make sure it doesn't contain any sensitive data before committing",
            file_name
        ))),
    }
}

pub fn besafe() {
    let mut gs = Command::new("git");
    gs.arg("status").arg("-s");

    let output_raw = gs.output().unwrap();
    let output = String::from_utf8_lossy(&output_raw.stdout);
    let files: Vec<String> = output
        .split("\n")
        .map(|i| i.to_owned())
        .filter(|i| !i.is_empty())
        .collect();

    for file in files.iter() {
        let mut file_split = file.split_whitespace();
        let code = file_split.next().expect("Failed to get file status!");
        let file_name = file_split.last().expect("Failed to get file info!");
        let status = code_to_enum(code);
        let ignored = is_gitignored(file_name);

        if let Some(decision) = evaluate_file(file_name, status, ignored) {
            match decision {
                FileDecision::Block(message) => {
                    println!("{}", message);
                    exit(1);
                }
                FileDecision::Warn(message) => println!("{}", message),
            }
        }
    }
    println!("No unsafe secret file problems found!");
}

#[cfg(test)]
mod tests {
    use super::{evaluate_file, is_forbidden_file, FileDecision};
    use crate::utils::Status;

    #[test]
    fn rejects_sensitive_env_files() {
        assert!(is_forbidden_file(".env"));
        assert!(is_forbidden_file(".env.production"));
        assert!(is_forbidden_file("apps/web/.env.local"));
        assert!(!is_forbidden_file(".env.example"));
    }

    #[test]
    fn rejects_stripe_config() {
        assert!(is_forbidden_file(".stripe-config"));
        assert!(is_forbidden_file("apps/api/.stripe-config"));
    }

    #[test]
    fn allows_non_forbidden_files() {
        assert!(!is_forbidden_file("README.md"));
        assert!(!is_forbidden_file("apps/api/config.ts"));
    }

    #[test]
    fn blocks_staged_sensitive_files() {
        let decision = evaluate_file(".env", Status::Staged, false);
        assert!(matches!(decision, Some(FileDecision::Block(_))));
    }

    #[test]
    fn blocks_modified_sensitive_files() {
        let decision = evaluate_file(".env.local", Status::Modified, false);
        assert!(matches!(decision, Some(FileDecision::Block(_))));
    }

    #[test]
    fn blocks_staged_modified_sensitive_files() {
        let decision = evaluate_file(".stripe-config", Status::StagedModified, false);
        assert!(matches!(decision, Some(FileDecision::Block(_))));
    }

    #[test]
    fn warns_for_untracked_sensitive_file_when_not_ignored() {
        let decision = evaluate_file(".env", Status::Untracked, false);
        assert!(matches!(decision, Some(FileDecision::Warn(_))));
    }

    #[test]
    fn skips_untracked_sensitive_file_when_ignored() {
        let decision = evaluate_file(".env", Status::Untracked, true);
        assert!(decision.is_none());
    }

    #[test]
    fn warns_for_unknown_sensitive_status() {
        let decision = evaluate_file(".env", Status::Unknown, false);
        assert!(matches!(decision, Some(FileDecision::Warn(_))));
    }

    #[test]
    fn skips_non_sensitive_file() {
        let decision = evaluate_file("README.md", Status::Staged, false);
        assert!(decision.is_none());
    }
}
