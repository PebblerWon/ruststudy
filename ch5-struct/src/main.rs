#[derive(Debug)]
struct Rectangle {
    width: u32,
    height: u32,
}
impl Rectangle {
    fn area(&self) -> u32 {
        self.width * self.height
    }
    fn can_hold(&self, rect2: &Rectangle) -> bool {
        self.width > rect2.width && self.height > rect2.height
    }

    fn square(size: u32) -> Self {
        Self {
            width: size,
            height: size,
        }
    }
}

fn main() {
    let rect = Rectangle {
        width: 10,
        height: 5,
    };
    let rect2 = Rectangle {
        width: 20,
        height: 10,
    };
    println!("Hello, world!,{}", rect.area());
    println!("Hello, world!,{}", rect2.can_hold(&rect));
    println!("Hello, world!,{:?}", Rectangle::square(3));
}
