use std::fmt::Display;

use serde::Serialize;

use crate::error::Error;

#[derive(Debug, Serialize)]
pub struct Account {
    client_id: u16,
    available: f32,
    held: f32,
    total: f32,
    locked: bool,
}
impl Account {
    pub fn new(client_id: u16) -> Self {
        Account {
            client_id,
            available: 0.0,
            held: 0.0,
            total: 0.0,
            locked: false,
        }
    }
    // @TODO since amount is f32 that is can be negative
    // I need to make sure that for deposit it is always positive
    // also think maybe that could be one method for withdraw as well with negative amount
    pub fn deposit(&mut self, amount: f32) -> Result<(), Error> {
        self.check_locked()?;
        log::debug!("Deposit to cleint #{}, amount: {}", self.client_id, amount);
        self.available += amount;
        self.clalc_total();
        log::debug!("Account state {}", self);
        Ok(())
    }
    pub fn withdraw(&mut self, amount: f32) -> Result<(), Error> {
        self.check_locked()?;
        log::debug!(
            "Withdraw from cleint #{}, amount: {}",
            self.client_id,
            amount
        );
        let ret = if self.available >= amount {
            self.available -= amount;
            Ok(())
        } else {
            Err(AccountError::Withdraw.into())
        };
        self.clalc_total();
        log::debug!("Account state {}", self);
        ret
    }
    /// `is_deposit` notify if current transaction is Deposite or Withdraw
    pub fn dispute(&mut self, amount: f32, is_deposit: bool) -> Result<(), Error> {
        self.check_locked()?;
        log::debug!("Dispute cleint #{} with amount: {}", self.client_id, amount);
        let ret = if is_deposit {
            if self.available >= amount {
                self.available -= amount;
                self.held += amount;
                Ok(())
            } else {
                Err(AccountError::Dispute.into())
            }
        } else {
            self.held += amount;
            Ok(())
        };
        self.clalc_total();
        log::debug!("Account state {}", self);
        ret
    }

    /// `is_deposit` notify if current transaction is Deposite or Withdraw
    pub fn resolve(&mut self, amount: f32) -> Result<(), Error> {
        self.check_locked()?;
        let ret = if self.held >= amount {
            self.held -= amount;
            self.available += amount;
            Ok(())
        } else {
            Err(AccountError::Resolve.into())
        };
        self.clalc_total();
        log::debug!("Account state {}", self);
        ret
    }

    /// `is_deposit` notify if current transaction is Deposite or Withdraw
    pub fn chargeback(&mut self, amount: f32) -> Result<(), Error> {
        self.check_locked()?;
        let ret = if self.available >= amount {
            self.available -= amount;
            self.lock();
            Ok(())
        } else {
            Err(AccountError::Chargeback.into())
        };
        self.clalc_total();
        log::debug!("Account state {}", self);
        ret
    }
    pub fn check_locked(&self) -> Result<(), Error> {
        log::debug!(
            "check client #{} is locked: {}",
            self.client_id,
            self.locked
        );
        if self.locked {
            Err(AccountError::Locked.into())
        } else {
            Ok(())
        }
    }
    fn clalc_total(&mut self) {
        self.total = self.available + self.held;
    }
    pub fn to_csv_row(&self) -> String {
        format!(
            "{},{},{},{},{}",
            self.client_id, self.available, self.held, self.total, self.locked
        )
    }
    fn lock(&mut self) {
        self.locked = true;
    }
}
impl Display for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ client_id: {}, available: {}, held: {}, total: {}, locked: {:?} }}",
            self.client_id, self.available, self.held, self.total, self.locked
        )
    }
}
#[derive(Debug, PartialEq)]
pub enum AccountError {
    Locked,
    Withdraw,
    Dispute,
    Resolve,
    Chargeback,
}
impl Display for AccountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Locked => write!(f, "Account is locked"),
            Self::Withdraw => write!(f, "Account has not enough money available to withdraw"),
            Self::Dispute => write!(f, "Account has not enough money available to dispute"),
            Self::Resolve => write!(f, "Account has not enough money available to resolve"),
            Self::Chargeback => write!(f, "Account has not enough money available to chargeback"),
        }
    }
}
impl From<AccountError> for Error {
    fn from(value: AccountError) -> Self {
        Self::Account(value)
    }
}
#[cfg(test)]
mod tests {
    use crate::account::AccountError;

    use super::Account;

    #[test]
    fn test_deposit() {
        let amount = 1.0;
        let mut acc = Account::new(1);

        let result = acc.deposit(amount);
        assert_eq!(Ok(()), result);
        assert_eq!(amount, acc.total);
        assert_eq!(amount, acc.available);
        assert_eq!(0.0, acc.held);
    }

    #[test]
    fn test_withdraw() {
        let amount = 2.0;
        let total = 4.0;
        let mut acc = Account::new(1);
        let _ = acc.deposit(total);

        let result = acc.withdraw(amount);
        assert_eq!(Ok(()), result);
        assert_eq!(total - amount, acc.total);
        assert_eq!(total - amount, acc.available);
        assert_eq!(0.0, acc.held);
        assert!(!acc.locked);
    }
    #[test]
    fn test_withdraw_err() {
        let amount = 5.0;
        let total = 4.0;
        let mut acc = Account::new(1);
        let _ = acc.deposit(total);

        let result = acc.withdraw(amount);
        assert_eq!(Err(AccountError::Withdraw.into()), result);

        assert_eq!(total, acc.total);
        assert_eq!(total, acc.available);
        assert_eq!(0.0, acc.held);
        assert!(!acc.locked);
    }
    #[test]
    fn test_dispute_deposit() {
        let amount = 2.0;
        let total = 4.0;
        let mut acc = Account::new(1);
        let _ = acc.deposit(total);
        let result = acc.dispute(amount, true);
        assert_eq!(Ok(()), result);
        assert_eq!(total, acc.total);
        assert_eq!(total - amount, acc.available);
        assert_eq!(amount, acc.held);
        assert!(!acc.locked);
    }
    #[test]
    fn test_dispute_withdraw() {
        let amount = 2.0;
        let total = 1.0;
        let mut acc = Account::new(1);
        let _ = acc.deposit(total);
        let result = acc.dispute(amount, false);
        assert_eq!(Ok(()), result);
        assert_eq!(total + amount, acc.total);
        assert_eq!(total, acc.available);
        assert_eq!(amount, acc.held);
        assert!(!acc.locked);
    }
    #[test]
    fn test_dispute_err() {
        let amount = 5.0;
        let total = 4.0;
        let mut acc = Account::new(1);
        let _ = acc.deposit(total);

        let result = acc.dispute(amount, true);
        assert_eq!(Err(AccountError::Dispute.into()), result);
        assert_eq!(total, acc.total);
        assert_eq!(total, acc.available);
        assert_eq!(0.0, acc.held);
        assert!(!acc.locked);
    }
    #[test]
    fn test_resolve() {
        let deposit = 4.0;
        let withdraw = 2.0;
        let mut acc = Account::new(1);
        let _ = acc.deposit(deposit);
        let _ = acc.withdraw(withdraw);
        let _ = acc.dispute(withdraw, false);
        let result = acc.resolve(withdraw);
        assert_eq!(Ok(()), result);
        assert_eq!(deposit, acc.total);
        assert_eq!(deposit, acc.available);
        assert_eq!(0.0, acc.held);
        assert!(!acc.locked);
    }
    #[test]
    fn test_resolve_err() {
        let resolve = 5.0;
        let dispute = 3.0;
        let total = 4.0;
        let mut acc = Account::new(1);
        let _ = acc.deposit(total);
        let _ = acc.dispute(dispute, true);
        let result = acc.resolve(resolve);
        assert_eq!(Err(AccountError::Resolve.into()), result);
        assert_eq!(total, acc.total);
        assert_eq!(total - dispute, acc.available);
        assert_eq!(dispute, acc.held);
        assert!(!acc.locked);
    }
    #[test]
    fn test_chargeback() {
        let amount = 2.0;
        let total = 4.0;
        let mut acc = Account::new(1);
        let _ = acc.deposit(total);
        let _ = acc.dispute(amount, true);
        let _ = acc.resolve(amount);
        let result = acc.chargeback(amount);
        assert_eq!(Ok(()), result);
        assert_eq!(total - amount, acc.total);
        assert_eq!(total - amount, acc.available);
        assert_eq!(0.0, acc.held);
        assert!(acc.locked);
    }
    #[test]
    fn test_chargeback_err() {
        let resolve = 5.0;
        let dispute = 3.0;
        let total = 4.0;
        let mut acc = Account::new(1);
        let _ = acc.deposit(total);
        let _ = acc.dispute(dispute, true);
        let result = acc.chargeback(resolve);
        assert_eq!(Err(AccountError::Chargeback.into()), result);
        assert_eq!(total, acc.total);
        assert_eq!(total - dispute, acc.available);
        assert_eq!(dispute, acc.held);
        assert!(!acc.locked);
    }
}
