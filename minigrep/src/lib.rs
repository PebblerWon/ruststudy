use std::error::Error;
use std::{env, fs, process};

pub struct Config {
    pub query: String,
    pub file_path: String,
    pub ignore_case: bool,
}
impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("Not enough arguments");
        }
        let ignore_case = env::var("IGNORE_CASE").is_ok();
        Ok(Config {
            query: args[1].clone(),
            file_path: args[2].clone(),
            ignore_case,
        })
    }
}
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.file_path)?;

    println!("{contents}");
    Ok(())
}
