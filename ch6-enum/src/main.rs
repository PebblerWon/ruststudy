enum IpAddrKind {
    v4(u8, u8, u8, u8),
    v6(String),
}
struct IpAddr {
    kind: IpAddrKind,
    address: String,
}
#[derive(Debug)]
enum UsState {
    Alabama,
    Alaska,
}
impl UsState {
    fn existed_in(&self, year: u32) -> bool {
        match self {
            UsState::Alabama => year >= 1819,
            UsState::Alaska => year >= 1959,
        }
    }
}
enum Coin {
    Penny,
    Nickel,
    Dime,
    Quarter(UsState),
}
fn value_in_cents(coin: Coin) -> u32 {
    match coin {
        Coin::Penny => {
            println!("Lucky penny");
            1
        }
        Coin::Nickel => 5,
        Coin::Dime => 10,
        Coin::Quarter(state) => {
            println!("State quarter from {state:?}!");
            25
        }
    }
}

fn plus_one(x: Option<i32>) -> Option<i32> {
    match x {
        Some(i) => Some(i + 1),
        None => None,
    }
}

fn add_fancy_hat() {}
fn remove_fancy_hat() {}
fn reroll() {}
fn move_player(num_spaces: u8) {}

fn play() {
    let dice_cell = 9;
    match dice_cell {
        3 => add_fancy_hat(),
        7 => remove_fancy_hat(),
        // other => move_player(other),
        // _ => reroll(),
        _ => (),
    }
}

fn iflet() {
    let config_max = Some(3);

    if let Some(max) = config_max {
        println!("The maximum value is: {max}");
    }
}

fn describe_state_quarter() -> Option<String> {
    let coin = Coin::Quarter(UsState::Alabama);

    // let state = if let Coin::Quarter(state) = coin {
    //     state
    // } else {
    //     return None;
    // };

    let Coin::Quarter(state) = coin else {
        return None;
    };

    if state.existed_in(1900) {
        Some(format!("{state:?} is pretty old, for America!"))
    } else {
        Some(format!("{state:?} is relatively new."))
    }
}
fn main() {
    let four = IpAddrKind::v4;
    let six = IpAddrKind::v6;

    let home = IpAddrKind::v4(127, 0, 0, 1);
    let loopback = IpAddrKind::v6(String::from("::1"));
    println!("Hello, world!");

    let t = value_in_cents(Coin::Quarter((UsState::Alabama)));

    let b = plus_one(Some(5));
    let b2 = plus_one(None);
    println!("plusone {:?},{:?}", b, b2);
    let c = describe_state_quarter();
    println!("c: {:?}", c)
}
