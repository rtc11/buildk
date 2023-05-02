use std::path::PathBuf;

use crate::project::project;

#[test]
fn main() {
    let project = project(&r#"
[project]
main = "incredible.kt"
"#.parse().unwrap()).unwrap();

    assert_eq!(project.compiled_main_file(), "incredibleKt")
}


#[test]
fn default_path() {
    let project = project(&r#"[project]"#.parse().unwrap()).unwrap();
    assert_eq!(project.path, PathBuf::from(std::env::current_dir().unwrap()))
}

#[test]
fn path() {
    let project = project(&r#"
[project]
path = "/Users"
"#.parse().unwrap()).unwrap();

    assert_eq!(project.path, PathBuf::from("/Users"))
}

#[test]
fn relative_path() {
    let current_path = std::env::current_dir().unwrap();
    let project = project(&r#"
[project]
relative-path = "src"
"#.parse().unwrap()).unwrap();

    assert_eq!(project.path, current_path.join("src"))
}

#[test]
fn default_src() {
    let current_path = std::env::current_dir().unwrap();
    let project = project(&r#"[project]"#.parse().unwrap()).unwrap();
    assert_eq!(project.src, current_path.join("src"))
}

#[test]
fn default_test() {
    let current_path = std::env::current_dir().unwrap();
    let project = project(&r#"[project]"#.parse().unwrap()).unwrap();
    assert_eq!(project.test, current_path.join("test"))
}

#[test]
fn default_out_path() {
    let default_out = std::env::current_dir().unwrap();
    let project = project(&r#"[project]"#.parse().unwrap()).unwrap();
    assert_eq!(project.out.path, default_out.join("out"));
}

#[test]
fn default_out_src() {
    let default_out = std::env::current_dir().unwrap();
    let project = project(&r#"[project]"#.parse().unwrap()).unwrap();
    assert_eq!(project.out.src, default_out.join("out").join("src"));
}

#[test]
fn default_out_test() {
    let default_out = std::env::current_dir().unwrap();
    let project = project(&r#"[project]"#.parse().unwrap()).unwrap();
    assert_eq!(project.out.test, default_out.join("out").join("test"));
}

#[test]
fn default_out_test_report() {
    let default_out = std::env::current_dir().unwrap();
    let project = project(&r#"[project]"#.parse().unwrap()).unwrap();
    assert_eq!(project.out.test_report, default_out.join("out").join("test-report"));
}

#[test]
fn default_out_cache() {
    let default_out = std::env::current_dir().unwrap();
    let project = project(&r#"[project]"#.parse().unwrap()).unwrap();
    assert_eq!(project.out.cache, default_out.join("out").join("cache.json"));
}

#[test]
fn default_out_jar() {
    let default_out = std::env::current_dir().unwrap();
    let project = project(&r#"[project]"#.parse().unwrap()).unwrap();
    assert_eq!(project.out.jar, default_out.join("out").join("app.jar"));
}
