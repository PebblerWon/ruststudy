fn largest(list: &[i32]) -> &i32 {
    let mut largest = &list[0];

    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}

fn longest<'a>(str1: &'a str, str2: &'a str) -> &'a str {
    if str1.len() > str2.len() {
        str1
    } else {
        str2
    }
}

fn calculate_length(s: &String) -> usize {
    return s.len();
}
fn main() {
    let numberlist = vec![7, 6, 5, 4, 3, 2, 1];

    let largest = largest(&numberlist);
    // println!("{}", largest);

    let str1 = String::from("abcaaef");
    let str2 = String::from("deff");
    let result = longest(str1.as_str(), str2.as_str());
    println!("The longest string is: {}", result);
}
