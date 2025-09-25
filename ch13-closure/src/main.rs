#[derive(Debug)]
enum ShirtColor {
    Red,
    Blue,
}
#[derive(Debug)]
struct Rectangle {
    width: u32,
    height: u32,
}
struct Inventory {
    shirts: Vec<ShirtColor>,
}

impl Inventory {
    fn giveaway(&self, user_preference: Option<ShirtColor>) -> ShirtColor {
        user_preference.unwrap_or_else(|| self.most_stocked())
    }

    fn most_stocked(&self) -> ShirtColor {
        let mut num_red = 0;
        let mut num_blue = 0;

        for color in &self.shirts {
            match color {
                ShirtColor::Red => num_red += 1,
                ShirtColor::Blue => num_blue += 1,
            }
        }

        if num_red > num_blue {
            ShirtColor::Red
        } else {
            ShirtColor::Blue
        }
    }
}

fn main() {
    let inv = Inventory {
        shirts: vec![ShirtColor::Red, ShirtColor::Red, ShirtColor::Blue],
    };

    let bob = Some(ShirtColor::Blue);
    let tom = None;

    println!("Bob gets {:?}", inv.giveaway(bob));
    println!("Tom gets {:?}", inv.giveaway(tom));

    let mut list = vec![1, 2, 3, 4];

    let mut only_borrows = || list.push(5);

    only_borrows();
    println!("After calling closure: {list:?}");

    let mut l2 = [
        Rectangle {
            width: 10,
            height: 20,
        },
        Rectangle {
            width: 8,
            height: 20,
        },
        Rectangle {
            width: 7,
            height: 20,
        },
        Rectangle {
            width: 5,
            height: 20,
        },
    ];

    l2.sort_by_key(|s| s.width);
    println!("{l2:?}");

    for val in l2.iter() {
        println!("{val:?}");
    }
    let mut a = create_string();
    a.push_str("world");
    println!("{a}")
}

fn create_string() -> String {
    let a = String::from("hello");
    a
}
