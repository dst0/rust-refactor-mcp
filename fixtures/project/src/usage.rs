use crate::simple::{Point, greet};
use crate::user::User;

pub fn demo() {
    let _p = Point::new(1.0, 2.0);
    let u = User::new("Alice", "alice@example.com");
    println!("{}", greet(&u.name));
    println!("User: {}", u);
}
