// Medium fixture: 4 entities — User + impls, Status enum + impls, validate fn, UserBuilder + impl

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

pub enum Status {
    Active,
    Inactive,
    Suspended,
}

impl Status {
    pub fn label(&self) -> &str {
        match self {
            Status::Active => "active",
            Status::Inactive => "inactive",
            Status::Suspended => "suspended",
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self, Status::Active)
    }
}

pub fn validate_email(email: &str) -> bool {
    email.contains('@') && email.contains('.')
}

pub struct UserBuilder {
    pub name: String,
    pub email: String,
    pub status: Option<Status>,
}

impl UserBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            email: String::new(),
            status: None,
        }
    }

    pub fn email(mut self, email: &str) -> Self {
        self.email = email.to_string();
        self
    }

    pub fn status(mut self, status: Status) -> Self {
        self.status = Some(status);
        self
    }

    pub fn build(self) -> User {
        User {
            name: self.name,
            email: self.email,
            status: self.status.unwrap_or(Status::Active),
        }
    }
}
