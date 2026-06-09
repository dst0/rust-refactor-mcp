// Usage file: references entities from medium.rs

use crate::medium::{User, Status, validate_email, UserBuilder};

pub fn create_admin(name: &str) -> User {
    UserBuilder::new(name)
        .email("admin@example.com")
        .status(Status::Active)
        .build()
}

pub fn check_user(u: &User) -> bool {
    u.is_active() && validate_email(&u.email)
}

pub fn list_statuses() -> Vec<&'static str> {
    vec![Status::Active.label(), Status::Inactive.label(), Status::Suspended.label()]
}

pub fn suspend_user(u: &mut User) {
    u.status = Status::Suspended;
    println!("{} is now {}", u.name, u.status.label());
}
