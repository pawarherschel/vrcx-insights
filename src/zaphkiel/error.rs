#[derive(Debug, Clone)]
pub enum Error {
    Sqlx,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Sqlx => write!(f, "sqlx error"),
        }
    }
}
