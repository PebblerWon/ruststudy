use std::rc::Rc;

#[derive(Debug)]
enum List {
    Nil,
    Cons(i32, Box<List>),
}
impl Drop for List {
    fn drop(&mut self) {
        println!("Dropping a List!");
    }
}

#[derive(Debug)]
enum List2 {
    Nil,
    Cons(i32, Rc<List2>),
}

pub trait Messenger {
    fn send(&self, msg: &str);
}

pub struct LimitTracker<'a, T: Messenger> {
    messenger: &'a T,
    value: usize,
    max: usize,
}
impl<'a, T> LimitTracker<'a, T>
where
    T: Messenger,
{
    pub fn new(messenger: &'a T, max: usize) -> LimitTracker<T> {
        LimitTracker {
            messenger,
            value: 0,
            max,
        }
    }
    pub fn set_value(&mut self, value: usize) {
        self.value = value;

        let persentage_of_max = self.value as f64 / self.max as f64;

        if (persentage_of_max >= 1.0) {
            self.messenger.send("Error: You are over your quota!");
        } else if (persentage_of_max >= 0.75) {
            self.messenger
                .send("Warning: You've used up over 75% of your quota!");
        }
    }
}
fn main() {
    let b = Box::new(5);

    let a = List::Nil;
    println!("b:{b}");
    println!("a:{:?}", a);

    let d = Rc::new(List2::Cons(
        5,
        Rc::new(List2::Cons(10, Rc::new(List2::Nil))),
    ));

    let e = List2::Cons(3, Rc::clone(&d));
    let f = List2::Cons(4, Rc::clone(&d));
    println!("e:{:?},f:{:?}", e, f);
    println!("strong_count:{}", Rc::strong_count(&d));
}
