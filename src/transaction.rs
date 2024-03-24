use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub kind: TransactionKind,
    #[serde(rename = "client")]
    pub client_id: u16,
    pub tx: u32,
    pub amount: Option<f32>,
    #[serde(skip, default = "TransactionState::default")]
    pub state: TransactionState,
}
impl Transaction {
    pub fn is_valid(&self) -> bool {
        use TransactionKind::*;
        match self.kind {
            Withdrawal | Deposit => self.amount.filter(|x| x >= &0.0).is_some(),
            _ => self.amount.is_none(),
        }
    }
    pub fn with_state(self, state: TransactionState) -> Self {
        Transaction { state, ..self }
    }
    pub fn set_state(&mut self, state: TransactionState) -> Result<(), Error> {
        //@TODO here could be implemented check of state transition
        self.state = state;
        Ok(())
    }
    pub fn can_dispute(&self) -> bool {
        self.state == TransactionState::Completed && self.amount.is_some()
    }
    pub fn can_resolve(&self) -> bool {
        self.state == TransactionState::Dispute && self.amount.is_some()
    }
    pub fn can_chargeback(&self) -> bool {
        self.state == TransactionState::Dispute && self.amount.is_some()
    }
}
impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{type: {:?},client: {},tx: {},amount: {}, state: {:?}}}",
            self.kind,
            self.client_id,
            self.tx,
            self.amount.unwrap_or_default(),
            self.state
        )
    }
}
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TransactionKind {
    Withdrawal,
    Deposit,
    Dispute,
    Resolve,
    Chargeback,
}
impl TransactionKind {
    pub fn is_deposit(&self) -> bool {
        *self == Self::Deposit
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Default)]
pub enum TransactionState {
    #[default]
    Processing,
    Dispute,
    Completed,
    Resolved,
    Chargeback,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionError {
    NotFound,
    UnExpectedAmount,
    Dispute,
    Resolve,
    Chargeback,
}

impl Display for TransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dispute => write!(f, "Transaction cannot be disputed"),
            Self::Resolve => write!(f, "Transaction has incorrect state and cannot be resolved"),
            Self::Chargeback => write!(f, "Transaction has incorrent state and cannot be charged back"),
            Self::NotFound => write!(f, "Transaction not found"),
            Self::UnExpectedAmount => write!(f, "Transaction has unexpected amount. Either it is deposit/withdrawal without amount or disput/resolve/chargeback with amount."),
        }
    }
}
impl From<TransactionError> for Error {
    fn from(value: TransactionError) -> Self {
        Self::Transaction(value)
    }
}
