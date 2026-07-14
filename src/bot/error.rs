use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum BotError {
    OtherError(anyhow::Error),
}

impl Display for BotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<A: 'static + Error + Send + Sync> From<A> for BotError {
    fn from(value: A) -> Self {
        Self::OtherError(anyhow::Error::new(value))
    }
}
