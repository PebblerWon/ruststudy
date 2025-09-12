use std::collections::HashMap;

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

    let mut scores = HashMap::new();
    scores.insert(String::from("Blue"), 10);
    scores.insert("Yellow".to_string(), 50);

    let score = scores.get("Blue").copied().unwrap_or(0);
    let score2 = scores.get("Red").copied().unwrap_or(0);
    println!("Blue score is: {:?}, Red Score is {}", score, score2);

    scores.entry("Blue".to_string()).or_insert(21);

    for (k, v) in &mut scores {
        *v += 1;
    }
    for (k, v) in &scores {
        println!("Key: {}, Value: {}", k, v);
    }

    let mut arr = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 9, 8, 8];
    arr.sort();
    println!("middle number is:{}", arr[arr.len() / 2]);

    let mut timesMap = HashMap::new();

    for (n) in arr {
        let key = n.to_string();

        // method 1
        // let v = timesMap.get_mut(&key);
        // if (v.is_some()) {
        //     let tt = v.unwrap();
        //     *tt += 1;
        // } else {
        //     timesMap.insert(key, 1);
        // }

        // method2

        let v = timesMap.get(&key).copied().unwrap_or(0);
        timesMap.insert(key, v + 1);
    }

    let mut maxTimes: i32 = 0;
    let mut target = None;
    for (k, v) in timesMap {
        if (v > maxTimes) {
            maxTimes = v;
            target = Some(k);
        }
    }

    println!("Target is: {:?}", target);
}
