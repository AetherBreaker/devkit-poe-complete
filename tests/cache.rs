use std::path::Path;

use devkit_poe_complete::cache;
use devkit_poe_complete::process::{RecordingRunner, SystemRunner};
use devkit_poe_complete::resolve::{Resolved, resolve};

// ---- cache ---------------------------------------------------------------------------------

fn project_with_tasks(names: &[&str]) -> tempfile::TempDir {
  let dir = tempfile::tempdir().unwrap();
  let body: String = names.iter().map(|n| format!("{n} = \"x\"\n")).collect();
  std::fs::write(dir.path().join("pyproject.toml"), format!("[tool.poe.tasks]\n{body}")).unwrap();
  dir
}

fn cached_names(root: &Path, runner: &RecordingRunner, bypass: bool) -> Vec<String> {
  cache::resolve_cached(root, runner, bypass)
    .unwrap()
    .tasks
    .into_iter()
    .map(|t| t.name)
    .collect()
}

#[test]
fn cold_run_populates_the_cache_file_and_warm_run_reads_it() {
  let dir = project_with_tasks(&["a"]);
  let runner = RecordingRunner::new(0);
  assert_eq!(cached_names(dir.path(), &runner, false), ["a"]);
  let cache_file = dir.path().join(".cache").join("devkit-completions.json");
  assert!(cache_file.is_file());

  // Tamper with the cache to prove the warm path reads it rather than re-resolving.
  let mut stored: cache::Stored = serde_json::from_str(&std::fs::read_to_string(&cache_file).unwrap()).unwrap();
  stored.resolved.tasks[0].name = "from-cache".into();
  std::fs::write(&cache_file, serde_json::to_string(&stored).unwrap()).unwrap();
  assert_eq!(cached_names(dir.path(), &runner, false), ["from-cache"]);
}

#[test]
fn editing_pyproject_invalidates() {
  let dir = project_with_tasks(&["a"]);
  let runner = RecordingRunner::new(0);
  cached_names(dir.path(), &runner, false);
  // A different size guarantees invalidation even if the mtime granularity is coarse.
  std::fs::write(dir.path().join("pyproject.toml"), "[tool.poe.tasks]\na = \"x\"\nbb = \"y\"\n").unwrap();
  assert_eq!(cached_names(dir.path(), &runner, false), ["a", "bb"]);
}

#[test]
fn editing_an_include_target_invalidates() {
  let dir = tempfile::tempdir().unwrap();
  std::fs::write(dir.path().join("pyproject.toml"), "[tool.poe]\ninclude = \"x.toml\"\n").unwrap();
  std::fs::write(dir.path().join("x.toml"), "[tool.poe.tasks]\ni = \"1\"\n").unwrap();
  let runner = RecordingRunner::new(0);
  assert_eq!(cached_names(dir.path(), &runner, false), ["i"]);
  std::fs::write(dir.path().join("x.toml"), "[tool.poe.tasks]\ni = \"1\"\njj = \"2\"\n").unwrap();
  assert_eq!(cached_names(dir.path(), &runner, false), ["i", "jj"]);
}

#[test]
fn a_different_devkit_version_invalidates() {
  let dir = project_with_tasks(&["a"]);
  let runner = RecordingRunner::new(0);
  cached_names(dir.path(), &runner, false);
  let cache_file = dir.path().join(".cache").join("devkit-completions.json");
  let mut stored: cache::Stored = serde_json::from_str(&std::fs::read_to_string(&cache_file).unwrap()).unwrap();
  stored.resolved.tasks[0].name = "stale".into();
  stored.fingerprint.devkit_version = "0.0.0".into();
  std::fs::write(&cache_file, serde_json::to_string(&stored).unwrap()).unwrap();
  assert_eq!(cached_names(dir.path(), &runner, false), ["a"]);
}

#[test]
fn bypass_ignores_and_rewrites_the_cache() {
  let dir = project_with_tasks(&["a"]);
  let runner = RecordingRunner::new(0);
  cached_names(dir.path(), &runner, false);
  let cache_file = dir.path().join(".cache").join("devkit-completions.json");
  let mut stored: cache::Stored = serde_json::from_str(&std::fs::read_to_string(&cache_file).unwrap()).unwrap();
  stored.resolved.tasks[0].name = "stale".into();
  std::fs::write(&cache_file, serde_json::to_string(&stored).unwrap()).unwrap();
  assert_eq!(cached_names(dir.path(), &runner, true), ["a"]);
  let after: cache::Stored = serde_json::from_str(&std::fs::read_to_string(&cache_file).unwrap()).unwrap();
  assert_eq!(after.resolved.tasks[0].name, "a");
}

#[test]
fn a_corrupt_cache_file_is_ignored() {
  let dir = project_with_tasks(&["a"]);
  std::fs::create_dir_all(dir.path().join(".cache")).unwrap();
  std::fs::write(dir.path().join(".cache").join("devkit-completions.json"), "{not json").unwrap();
  assert_eq!(cached_names(dir.path(), &RecordingRunner::new(0), false), ["a"]);
}

// ---- parity with the real poe, on this repo ---------------------------------------------

/// The one guard against the Rust resolver silently diverging from poe. Skips (with a
/// note) when this repo's venv has no `poe`, so it never fails for environmental reasons.
#[test]
fn resolved_tasks_match_poe_list_tasks_for_this_repo() {
  let root = Path::new(env!("CARGO_MANIFEST_DIR"));
  let poe = root.join(".venv").join("Scripts").join("poe.exe");
  let poe = if poe.is_file() {
    poe
  } else {
    root.join(".venv").join("bin").join("poe")
  };
  if !poe.is_file() {
    eprintln!("skipping parity test: no venv poe at {}", poe.display());
    return;
  }
  let out = std::process::Command::new(&poe)
    .arg("_list_tasks")
    .current_dir(root)
    .output()
    .unwrap();
  let mut expected: Vec<String> = String::from_utf8_lossy(&out.stdout)
    .split_whitespace()
    .map(str::to_string)
    .collect();
  expected.sort();
  assert!(!expected.is_empty(), "poe listed no tasks");

  let r: Resolved = resolve(root, &SystemRunner).unwrap();
  let mut actual: Vec<String> = r.tasks.into_iter().map(|t| t.name).collect();
  actual.sort();
  assert_eq!(actual, expected);
}
