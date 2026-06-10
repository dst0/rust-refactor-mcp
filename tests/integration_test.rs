//! Integration tests: copy fixture project → extract entity → verify compilation.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/project")
}

fn copy_fixture() -> PathBuf {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let tmp = std::env::temp_dir().join(format!(
        "rust_refactor_integration_{}_{}",
        std::process::id(),
        ts
    ));
    std::fs::create_dir_all(&tmp).unwrap();
    cp_r(fixture_dir(), &tmp);
    tmp
}

fn cp_r(src: PathBuf, dst: &Path) {
    for entry in std::fs::read_dir(&src).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let new_path = dst.join(path.file_name().unwrap());
        if path.is_dir() {
            std::fs::create_dir_all(&new_path).ok();
            cp_r(path, &new_path);
        } else {
            std::fs::copy(&path, &new_path).ok();
        }
    }
}

fn cargo_check(project_dir: &PathBuf) -> bool {
    let output = Command::new("cargo")
        .arg("check")
        .current_dir(project_dir)
        .output()
        .unwrap();
    if !output.status.success() {
        eprintln!(
            "cargo check failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    output.status.success()
}

fn run_extract(file_path: &str, entity_name: &str, target_folder: &str) {
    let bin = Command::new("cargo")
        .args(["run", "--"])
        .args([file_path, entity_name, target_folder])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("cargo run failed");
    if !bin.status.success() {
        panic!(
            "Extraction failed for {}: {}\n{}",
            entity_name,
            String::from_utf8_lossy(&bin.stderr),
            String::from_utf8_lossy(&bin.stdout)
        );
    }
}

#[test]
fn extract_point_from_simple() {
    let project = copy_fixture();
    let src_dir = project.join("src");

    assert!(
        cargo_check(&project),
        "Fixture should compile before extraction"
    );

    let file_path = src_dir.join("simple.rs");
    run_extract(
        file_path.to_str().unwrap(),
        "Point",
        src_dir.to_str().unwrap(),
    );

    // point.rs created
    let point_file = src_dir.join("point.rs");
    assert!(point_file.exists(), "point.rs should exist");
    let point_content = fs::read_to_string(&point_file).unwrap();
    assert!(point_content.contains("pub struct Point"));

    // simple.rs updated — Point removed, greet remains
    let simple_content = fs::read_to_string(&file_path).unwrap();
    assert!(!simple_content.contains("struct Point"));
    assert!(simple_content.contains("pub fn greet"));

    // usage.rs updated — Point import redirected
    let usage_content = fs::read_to_string(src_dir.join("usage.rs")).unwrap();
    assert!(
        usage_content.contains("use crate::point::Point"),
        "usage.rs: {usage_content}"
    );
    assert!(usage_content.contains("greet"));

    // lib.rs updated
    let lib_content = fs::read_to_string(src_dir.join("lib.rs")).unwrap();
    assert!(lib_content.contains("pub mod point"));

    // Project compiles after extraction
    assert!(
        cargo_check(&project),
        "Project should compile after extraction\nsimple.rs:\n{}",
        simple_content
    );

    std::fs::remove_dir_all(&project).ok();
}

#[test]
fn extract_user_from_user() {
    let project = copy_fixture();
    let src_dir = project.join("src");

    assert!(
        cargo_check(&project),
        "Fixture should compile before extraction"
    );

    let file_path = src_dir.join("user.rs");
    run_extract(
        file_path.to_str().unwrap(),
        "User",
        src_dir.to_str().unwrap(),
    );

    // medium.rs updated — User import redirected
    let medium_content = fs::read_to_string(src_dir.join("medium.rs")).unwrap();
    assert!(
        medium_content.contains("use crate::user::User"),
        "medium.rs: {medium_content}"
    );

    // usage.rs updated
    let usage_content = fs::read_to_string(src_dir.join("usage.rs")).unwrap();
    assert!(
        usage_content.contains("use crate::user::User"),
        "usage.rs: {usage_content}"
    );

    // lib.rs updated
    let lib_content = fs::read_to_string(src_dir.join("lib.rs")).unwrap();
    assert!(lib_content.contains("pub mod user"));

    assert!(
        cargo_check(&project),
        "Project should compile after User extraction"
    );
    std::fs::remove_dir_all(&project).ok();
}

#[test]
fn extract_status_from_medium() {
    let project = copy_fixture();
    let src_dir = project.join("src");

    assert!(
        cargo_check(&project),
        "Fixture should compile before extraction"
    );

    let file_path = src_dir.join("medium.rs");
    run_extract(
        file_path.to_str().unwrap(),
        "Status",
        src_dir.to_str().unwrap(),
    );

    // status.rs created
    let status_file = src_dir.join("status.rs");
    assert!(status_file.exists(), "status.rs should exist");
    let status_content = fs::read_to_string(&status_file).unwrap();
    assert!(status_content.contains("pub enum Status"));

    // user.rs updated — Status import redirected
    let user_content = fs::read_to_string(src_dir.join("user.rs")).unwrap();
    assert!(
        user_content.contains("use crate::status::Status"),
        "user.rs: {user_content}"
    );

    // medium.rs updated — Status removed, UserBuilder remains
    let medium_content = fs::read_to_string(&file_path).unwrap();
    assert!(!medium_content.contains("pub enum Status"));
    assert!(medium_content.contains("pub struct UserBuilder"));

    assert!(
        cargo_check(&project),
        "Project should compile after Status extraction"
    );
    std::fs::remove_dir_all(&project).ok();
}

#[test]
fn extract_greet_from_simple() {
    let project = copy_fixture();
    let src_dir = project.join("src");

    assert!(
        cargo_check(&project),
        "Fixture should compile before extraction"
    );

    let file_path = src_dir.join("simple.rs");
    run_extract(
        file_path.to_str().unwrap(),
        "greet",
        src_dir.to_str().unwrap(),
    );

    // greet_impl.rs created
    let greet_file = src_dir.join("greet_impl.rs");
    assert!(greet_file.exists(), "greet_impl.rs should exist");
    let greet_content = fs::read_to_string(&greet_file).unwrap();
    assert!(greet_content.contains("pub fn greet"));

    // simple.rs updated — greet re-exported
    let simple_content = fs::read_to_string(&file_path).unwrap();
    assert!(
        simple_content.contains("pub mod greet_impl"),
        "simple.rs should contain 'pub mod greet_impl', but got:\n{}",
        simple_content
    );
    assert!(
        simple_content.contains("pub use crate::greet_impl::greet"),
        "simple.rs should contain 'pub use crate::greet_impl::greet', but got:\n{}",
        simple_content
    );

    // Project compiles after extraction
    assert!(
        cargo_check(&project),
        "Project should compile after greet extraction"
    );
    std::fs::remove_dir_all(&project).ok();
}
