fn main() {
    let mut v: Vec<i32> = Vec::new();
    let v2 = vec![1, 2, 3, 4, 5];

    v.push(4);
    v.push(5);
    v.push(6);
    v.push(7);

    // copy
    let third = v2[2];
    println!("third:{third}");

    // ref
    let first = &mut v[2];
    *first = 99;

    println!("Value is: {},third:{}", first, third);

    *(&mut v[2]) = 80;

    println!("{}", &v[2]);

    let b = v.get(2);

    match b {
        Some(v) => {
            println!("The value is: {}", v);
        }
        None => println!("No value found"),
    }

    for i in v2 {
        println!("i:{i}")
    }

    let mut s = String::new();
    let s2 = "initial string";
    let s3 = s2.to_string();
    let mut s4 = String::from("s4");

    s4.push_str(" is  a string");
    println!("string value is {}", s4);

    let s5 = s3 + s2;
    println!("string value is {}", s5);

    let s6 = format!(
        "{}-{}-{}",
        String::from("a"),
        String::from("b"),
        String::from("c")
    );
    println!("string value is {}", s6);

    for char in s6.chars() {
        println!("char: {}", char);
    }

    for byte in s6.bytes() {
        println!("byte: {}", byte);
    }
}
