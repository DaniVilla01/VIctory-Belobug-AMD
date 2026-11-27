use std::path::Path;
use std::process::Command;

fn victory_in(dir: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_victory"));
    command
        .current_dir(dir)
        .env_remove("VICTORY_CONFIG")
        .env_remove("VICTORY_AUDIO_CAPTURE")
        .env_remove("VICTORY_EMIT_DUMP")
        .env_remove("VICTORY_SHADER_CENSUS");
    command
}

#[test]
fn shaders_list_requires_a_destination_outside_the_repo() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("clone");
    let input = dir.path().join("input");
    let cache = dir.path().join("cache");
    std::fs::create_dir(&repo).unwrap();
    std::fs::write(repo.join(victory_core::policy::ROOT_MARKER), "").unwrap();
    std::fs::create_dir(&input).unwrap();

    let run = |list: &Path| {
        victory_in(&repo)
            .arg("shaders")
            .arg(&input)
            .arg("--list")
            .arg(list)
            .env("VICTORY_CACHE_DIR", &cache)
            .env("VICTORY_DATA_DIR", dir.path().join("data"))
            .output()
            .unwrap()
    };

    let report = Path::new("failures.txt");
    let refused = run(report);
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert!(!refused.status.success(), "{refused:?}");
    assert!(stderr.contains("inside the VIctory source tree"), "{stderr}");
    assert!(!repo.join(report).exists());
    assert!(!cache.exists(), "the report must be checked before cache setup");

    let outside = dir.path().join(report);
    let allowed = run(&outside);
    assert!(allowed.status.success(), "{allowed:?}");
    assert!(outside.is_file());
}

#[test]
fn translate_rejects_a_shader_census_in_the_repo() {
    let repo = tempfile::tempdir().unwrap();
    let repo = repo.path();
    std::fs::write(repo.join(victory_core::policy::ROOT_MARKER), "").unwrap();
    std::fs::create_dir(repo.join("input")).unwrap();

    let output = victory_in(repo)
        .args(["translate", "input"])
        .env("VICTORY_SHADER_CENSUS", "census.txt")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "{output:?}");
    assert!(stderr.contains("inside the VIctory source tree"), "{stderr}");
    assert!(!repo.join("census.txt").exists());
}
