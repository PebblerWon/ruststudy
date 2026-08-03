use std::fmt;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Todo,
    InProgress,
    Done,
}

impl fmt::Display for Status {
  fn fmt(&self,f: &mut fmt::Formatter)-> fmt::Result {
    let d = match self {
      Status::Todo => "待办",
      Status::InProgress => "进行中",
      Status::Done => "已完成",
    };
    write!(f,"{}",d)
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low,
    Medium,
    High,
}

impl fmt::Display for Priority {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      let d = match self {
        Priority::Low=>"低",
        Priority::Medium=>"中",
        Priority::High=>"高",
      };
      write!(f,"{}",d)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn testStatus() {
    let a = Status::Todo;
    let res = serde_json::to_string(&a).unwrap();
    assert_eq!(res,"\"todo\"");

    let a = Status::InProgress;
    let res = serde_json::to_string(&a).unwrap();
    assert_eq!(res,"\"in_progress\"");

    let a = Status::Done;
    let res = serde_json::to_string(&a).unwrap();
    assert_eq!(res,"\"done\"");
  }

    #[test]
  fn testPriority() {
    let a = Priority::Low;
    let res = serde_json::to_string(&a).unwrap();
    assert_eq!(res,"\"low\"");

    let a = Priority::Medium;
    let res = serde_json::to_string(&a).unwrap();
    assert_eq!(res,"\"medium\"");

    let a = Priority::High;
    let res = serde_json::to_string(&a).unwrap();
    assert_eq!(res,"\"high\"");
  }
  
}

