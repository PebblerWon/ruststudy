use std::fs::File;
use std::io::{self, ErrorKind, Read};

fn read_username_from_file() -> Result<String, io::Error> {
    let mut username_file = File::open("hello.txt")?;
    let mut username = String::new();
    username_file.read_to_string(&mut username)?;
    Ok(username)
}

fn main() {
    // let file_result = File::open("hello.txt");
    // let file = match file_result {
    //     Ok(file) => file,
    //     Err(error) => match error.kind() {
    //         ErrorKind::NotFound => match File::create("hello.txt") {
    //             Ok(create_file) => create_file,
    //             Err(e) => panic!("Problem creating file: {:?}", e),
    //         },
    //         _ => panic!("Problem opening file: {:?}", error),
    //     },
    // };

    // let file2 = File::open("hello.txt").unwrap();

    // let file3 = File::open("hello.txt").expect("hello.txt should be included in this project");

    // let file = File::open("hello.txt");

    read_username_from_file();
}
