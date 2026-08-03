use std::rc::Rc;
use std::cell::RefCell;

struct Owner {
    name: String
}

struct Gadegt {
    id: usize,
    owner: Rc<Owner>
}
fn main() {
    
   let s = Rc::new(RefCell::new("我很善变，还拥有多个主人".to_string()));

    let s1 = s.clone();
    let s2 = s.clone();
    // let mut s2 = s.borrow_mut();
    s2.borrow_mut().push_str(", oh yeah!");

    println!("{:?}\n{:?}\n{:?}", s, s1, s2);
}