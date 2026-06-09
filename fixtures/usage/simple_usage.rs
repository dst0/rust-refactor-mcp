// Usage file: references entities from simple.rs

use crate::simple::{Point, greet};

pub fn main_demo() {
    let p1 = Point::new(0.0, 0.0);
    let p2 = Point::new(3.0, 4.0);
    let dist = p1.distance(&p2);
    println!("Distance: {}", dist);
    println!("{}", p1);
    println!("{}", greet("World"));
}

pub fn midpoint(a: &Point, b: &Point) -> Point {
    Point::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0)
}
