use crate::medium::Status;
pub struct User {
    pub name: String,
    pub status: Status,
    pub email: String,
}

impl User {
    pub fn new(name: &str, email: &str) -> Self {
        Self {
            name: name.to_string(),
            status: Status::Active,
            email: email.to_string(),
        }
    }

    pub fn deactivate(&mut self) {
        self.status = Status::Inactive;
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, Status::Active)
    }
}

impl std::fmt::Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({}) [{:?}]", self.name, self.email, self.status)
    }
}

