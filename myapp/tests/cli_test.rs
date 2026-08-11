use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn taskflow_cmd(temp_dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("taskflow").unwrap();
    cmd.env("TASKFLOW_DATA_DIR", temp_dir.path());
    cmd
}

#[test]
fn test_add_and_list() {
    let temp = TempDir::new().unwrap();
    taskflow_cmd(&temp)
        .arg("add")
        .arg("集成测试任务")
        .assert()
        .success()
        .stdout(predicate::str::contains("创建成功"));
    taskflow_cmd(&temp)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("集成测试任务"));
}

#[test]
fn test_add_empty_title() {
    let temp = TempDir::new().unwrap();
    taskflow_cmd(&temp)
        .arg("add")
        .arg("")
        .assert()
        .failure()
        .stderr(predicate::str::contains("标题不能为空"));
}

#[test]
fn test_delete_with_force() {
    let temp = TempDir::new().unwrap();
    // 先创建，从 add 成功输出中提取 ID（格式：✓ 任务创建成功：(xxxxxxxx)[中] 待删除 (未完成)）
    let assert = taskflow_cmd(&temp)
        .arg("add")
        .arg("待删除")
        .assert()
        .success();
    let add_stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let id = add_stdout
        .split('(')
        .nth(1)
        .and_then(|s| s.split(')').next())
        .unwrap()
        .to_string();
    taskflow_cmd(&temp)
        .arg("delete")
        .arg(&id)
        .arg("--force")
        .assert()
        .success()
        .stdout(predicate::str::contains("已删除"));
}

#[test]
fn test_export_unsupported_format() {
    let temp = TempDir::new().unwrap();
    taskflow_cmd(&temp)
        .arg("export")
        .arg("--format")
        .arg("json")
        .assert()
        .failure()
        .stderr(predicate::str::contains("不支持的导出格式"));
}
