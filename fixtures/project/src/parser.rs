use crate::error::Error;

pub trait Parser {
    type Output;
    fn parse(&self, input: &str) -> Result<Self::Output, Error>;
    fn name(&self) -> &str;
}

