use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn taskflow_cmd(temp_dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("taskflow").unwrap();
    cmd.env("TASKFLOW_DATA_DIR", temp_dir.path());
    cmd
}

fn get_id_from_stdout(add_stdout: &str) -> String {
    add_stdout
        .split('(')
        .nth(1)
        .and_then(|s| s.split(')').next())
        .unwrap()
        .to_string()
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
    let id = get_id_from_stdout(&add_stdout);

    taskflow_cmd(&temp)
        .arg("delete")
        .arg(&id)
        .arg("--force")
        .assert()
        .success()
        .stdout(predicate::str::contains("已删除"));
}

#[test]
fn test_delete_confirm_y() {
    let temp = TempDir::new().unwrap();

    let assert = taskflow_cmd(&temp)
        .arg("add")
        .arg("待删除")
        .assert()
        .success();
    let add_stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let id = get_id_from_stdout(&add_stdout);

    let mut cmd = taskflow_cmd(&temp);

    cmd.arg("delete").arg(&id);
    cmd.write_stdin("y\n");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("已删除"));
}

#[test]
fn test_export() {
    let temp = TempDir::new().unwrap();
    let csv_path = temp.path().join("test.csv");
    taskflow_cmd(&temp)
        .arg("export")
        .arg("--output")
        .arg(csv_path.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("已导出任务到"));
    assert!(csv_path.exists());
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

#[test]
fn test_update() {
    let temp = TempDir::new().unwrap();

    let assert = taskflow_cmd(&temp)
        .arg("add")
        .arg("新增")
        .assert()
        .success();
    let add_stdout = String::from_utf8_lossy(&assert.get_output().stdout);

    let id = get_id_from_stdout(&add_stdout);

    taskflow_cmd(&temp)
        .arg("update")
        .arg(&id)
        .arg("--title")
        .arg("待修改")
        .arg("--status")
        .arg("done")
        .arg("--priority")
        .arg("low")
        .arg("--tag")
        .arg("1,2")
        .assert()
        .success()
        .stdout(predicate::str::contains("任务已更新"))
        .stdout(predicate::str::contains("待修改"))
        .stdout(predicate::str::contains("已完成"))
        .stdout(predicate::str::contains("[低]"));

    taskflow_cmd(&temp)
        .arg("update")
        .arg("badid")
        .assert()
        .failure()
        .stderr(predicate::str::contains("任务不存在"));
}

#[test]
fn test_search() {
    let temp = TempDir::new().unwrap();

    taskflow_cmd(&temp)
        .arg("add")
        .arg("新增")
        .arg("--description")
        .arg("描述")
        .assert()
        .success();

    taskflow_cmd(&temp)
        .arg("search")
        .arg("描述")
        .assert()
        .success()
        .stdout(predicate::str::contains("搜索到 1 条结果"))
        .stdout(predicate::str::contains("新增"));
}

#[test]
fn test_stats() {
    let temp = TempDir::new().unwrap();
    taskflow_cmd(&temp)
        .arg("stats")
        .assert()
        .success()
        .stdout(predicate::str::contains("总任务数：0"));
    for _ in 0..3 {
        taskflow_cmd(&temp)
            .arg("add")
            .arg("新增")
            .arg("--description")
            .arg("描述")
            .assert()
            .success();
    }

    taskflow_cmd(&temp)
        .arg("stats")
        .assert()
        .success()
        .stdout(predicate::str::contains("总任务数：3"));
}

#[test]
fn test_list_is_empty() {
    let temp = TempDir::new().unwrap();

    taskflow_cmd(&temp)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("暂无任务"));
}
