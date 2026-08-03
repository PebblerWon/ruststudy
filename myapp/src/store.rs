use crate::models::{Task};
use std::io::{self};
use std::path::PathBuf;


pub trait Store {
  fn load(&self) -> Result<Vec<Task>, io::Error> ;
  fn save(&self,tasks:Vec<Task>)-> Result<(), io::Error>;
}

pub struct JsonFileStore {
  file_path: PathBuf,
}

impl JsonFileStore {
  pub fn new() -> Result<JsonFileStore, io::Error> {
    let home_dir = dirs::home_dir();
    
    let file_path =  match home_dir {
        Some(home)=>{
         home.join(".taskflow")
        },
        _=>{
          PathBuf::from("./.taskflow")
        },
    };
    
     Ok(JsonFileStore{
        file_path,
      })
  }
}

#[cfg(test)]
mod tests {

  use crate::store::JsonFileStore;
  use std::path::PathBuf;

  #[test]
  fn testJsonFileStore(){
    let a = JsonFileStore::new().unwrap();
    let b = a.file_path.to_str().unwrap();
    assert!(b.contains(".taskflow"));

  } 
}