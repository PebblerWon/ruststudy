mod front_of_house;

fn deliver_order() {}

mod back_of_house {
    fn fix_incorrect_order() {
        cook_order();
        super::deliver_order();

        super::front_of_house::hosting::add_to_waitlist();
    }

    fn cook_order() {}

    pub struct Breakfast {
        pub toast: String,
        seasonal_fruit: String,
    }
    impl Breakfast {
        pub fn summer(toast: &str) -> Breakfast {
            Breakfast {
                toast: String::from(toast),
                seasonal_fruit: String::from("peaches"),
            }
        }
    }

    pub enum Appetizer {
        Soup,
        Salad,
    }
}

use front_of_house::hosting;
pub fn eat_at_restaurant() {
    // absolute path

    hosting::add_to_waitlist();

    // relative path

    front_of_house::hosting::add_to_waitlist();

    let mut meal = back_of_house::Breakfast::summer("Rye");

    meal.toast = String::from("Wheat");

    println!("{}", meal.toast);

    // cat't
    // meal.seasonal_fruit = String::from("blueberries");

    let o1 = back_of_house::Appetizer::Soup;
    let o2 = back_of_house::Appetizer::Salad;
}

mod customer {
    use crate::front_of_house::hosting;

    pub fn b() {
        hosting::add_to_waitlist();
    }
}
