use std::fmt::Display;

use crate::{account::AccountError, transaction::TransactionError};

#[derive(Debug, PartialEq)]
pub enum Error {
    Account(AccountError),
    Transaction(TransactionError),
    ArgsParse,
    Other(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Account(err) => {
                write!(f, "AccountError: {}", err)
            }
            Self::Transaction(err) => {
                write!(f, "Transaction: {}", err)
            }

            Self::ArgsParse => {
                write!(f, "The application expects only one argument, that should be an input csv file with transactions")
            }
            Self::Other(e) => {
                write!(f, "{}", e)
            }
        }
    }
}
