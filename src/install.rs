use std::{
    env::current_exe,
    fs::{read_to_string, write},
    path::{Path, PathBuf},
};

#[cfg(unix)]
use std::{
    fs::{set_permissions, Permissions},
    os::unix::fs::PermissionsExt,
};

const PRE_COMMIT_PATH: &str = ".git/hooks/pre-commit";
const BESAFE_BLOCK_START: &str = "# >>> besafe >>>";
const BESAFE_BLOCK_END: &str = "# <<< besafe <<<";

pub fn install_hook() {
    let path = PathBuf::from(PRE_COMMIT_PATH);
    let existing_hook = read_to_string(&path).unwrap_or_default();

    let current_path = current_exe()
        .map(|path| normalize_executable_path(path.as_path()))
        .unwrap_or_default();

    let hook_contents = upsert_besafe_hook(&existing_hook, &current_path);

    if hook_contents == existing_hook {
        println!("besafe is already installed in this repository!");
        return;
    }

    write(&path, hook_contents).expect("Failed to install pre-commit hook!");
    ensure_hook_executable(&path);

    println!("Successfully installed pre-commit hook!");
}

fn normalize_executable_path(path: &Path) -> String {
    let current_path = path.to_string_lossy().to_string();

    if cfg!(windows) {
        let path = current_path.replace('\\', "/");
        if let Some((drive, rest)) = path.split_once(':') {
            let drive = drive.to_lowercase();
            let rest = rest.trim_start_matches('/');
            format!("/{}/{}", drive, rest)
        } else {
            path
        }
    } else {
        current_path
    }
}

fn upsert_besafe_hook(existing_hook: &str, executable_path: &str) -> String {
    let without_block = remove_besafe_block(existing_hook);
    let without_legacy_lines = remove_legacy_besafe_lines(&without_block);
    let mut hook = ensure_shebang(&without_legacy_lines);
    let besafe_block = render_besafe_block(executable_path);

    hook = hook.trim_end().to_string();
    format!("{hook}\n\n{besafe_block}\n")
}

fn ensure_shebang(hook: &str) -> String {
    let trimmed = hook.trim_start_matches('\n');

    if trimmed.starts_with("#!/") {
        trimmed.to_string()
    } else if trimmed.trim().is_empty() {
        "#!/bin/sh".to_string()
    } else {
        format!("#!/bin/sh\n\n{trimmed}")
    }
}

fn remove_besafe_block(hook: &str) -> String {
    let mut result = hook.to_string();

    while let Some(start) = result.find(BESAFE_BLOCK_START) {
        let Some(end_relative) = result[start..].find(BESAFE_BLOCK_END) else {
            break;
        };
        let end = start + end_relative + BESAFE_BLOCK_END.len();
        let end = if result[end..].starts_with('\n') {
            end + 1
        } else {
            end
        };
        result.replace_range(start..end, "");
    }

    result
}

fn remove_legacy_besafe_lines(hook: &str) -> String {
    hook.lines()
        .filter(|line| !is_legacy_besafe_line(line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn is_legacy_besafe_line(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return false;
    }

    let candidate = trimmed.trim_matches(|c| c == '"' || c == '\'');
    let normalized = candidate.replace('\\', "/").to_lowercase();
    normalized.ends_with("/besafe") || normalized.ends_with("/besafe.exe")
}

fn render_besafe_block(executable_path: &str) -> String {
    let quoted_path = shell_single_quote(executable_path);

    format!(
        "{BESAFE_BLOCK_START}
BESAFE_BIN={quoted_path}
if [ -x \"$BESAFE_BIN\" ]; then
  \"$BESAFE_BIN\"
elif command -v besafe >/dev/null 2>&1; then
  besafe
else
  echo \"[besafe] Could not find an executable besafe binary. Run 'besafe install'.\" >&2
  exit 1
fi
{BESAFE_BLOCK_END}"
    )
}

fn shell_single_quote(value: &str) -> String {
    if value.is_empty() {
        "''".to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\"'\"'"))
    }
}

#[cfg(unix)]
fn ensure_hook_executable(path: &Path) {
    set_permissions(path, Permissions::from_mode(0o755))
        .expect("Failed to set pre-commit hook as executable!");
}

#[cfg(not(unix))]
fn ensure_hook_executable(_: &Path) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upserts_hook_with_besafe_block() {
        let hook = upsert_besafe_hook("", "/tmp/bin/besafe");

        assert!(hook.starts_with("#!/bin/sh"));
        assert!(hook.contains(BESAFE_BLOCK_START));
        assert!(hook.contains("BESAFE_BIN='/tmp/bin/besafe'"));
    }

    #[test]
    fn removes_legacy_path_only_hook_line() {
        let existing = "#!/bin/sh\n\n/Users/me/bin/besafe\n\necho \"keep me\"";
        let hook = upsert_besafe_hook(existing, "/tmp/bin/besafe");

        assert!(!hook.contains("/Users/me/bin/besafe"));
        assert!(hook.contains("echo \"keep me\""));
        assert_eq!(hook.matches(BESAFE_BLOCK_START).count(), 1);
    }

    #[test]
    fn upsert_is_idempotent() {
        let first = upsert_besafe_hook("#!/bin/sh\n\necho test\n", "/tmp/bin/besafe");
        let second = upsert_besafe_hook(&first, "/tmp/bin/besafe");

        assert_eq!(first, second);
        assert_eq!(second.matches(BESAFE_BLOCK_START).count(), 1);
    }

    #[test]
    fn escapes_paths_with_quotes() {
        let hook = upsert_besafe_hook("", "/tmp/it's/besafe");
        assert!(hook.contains("BESAFE_BIN='/tmp/it'\"'\"'s/besafe'"));
    }
}
