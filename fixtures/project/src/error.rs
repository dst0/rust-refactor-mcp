pub enum Error {
    Syntax(String),
    Io(String),
    NotFound(String),
}

impl Error {
    pub fn message(&self) -> &str {
        match self {
            Error::Syntax(msg) => msg,
            Error::Io(msg) => msg,
            Error::NotFound(msg) => msg,
        }
    }

    pub fn kind(&self) -> &str {
        match self {
            Error::Syntax(_) => "syntax",
            Error::Io(_) => "io",
            Error::NotFound(_) => "not_found",
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind(), self.message())
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error::{}({})", self.kind(), self.message())
    }
}

