use crate::{account::Account, error::Error, transaction::*};
use std::cell::RefCell;
use std::collections::HashMap;
thread_local! {
    /// Account table
    static ACC: RefCell<HashMap<u16, Account>> = Default::default();
    /// Transaction table
    static TX: RefCell<HashMap<u16, HashMap<u32, Transaction>>> = Default::default();
}
///Represents implementation of DB that contains account and transaction information
#[derive(Debug, Default)]
pub struct Db {}
impl Db {
    pub fn process(&self, tx: Transaction) -> Result<(), Error> {
        match tx.kind {
            TransactionKind::Deposit if tx.amount.is_some() => {
                self.account_deposit(tx.client_id, tx.amount.unwrap())?;
                self.set_tx(tx.client_id, tx.with_state(TransactionState::Completed));
                Ok(())
            }
            TransactionKind::Withdrawal if tx.amount.is_some() => {
                self.account_withdraw(tx.client_id, tx.amount.unwrap())?;
                self.set_tx(tx.client_id, tx.with_state(TransactionState::Completed));
                Ok(())
            }
            TransactionKind::Dispute if tx.amount.is_none() => {
                if let Some(t) = self.get_tx(&tx.client_id, &tx.tx) {
                    if t.can_dispute() {
                        self.account_dispute(tx.client_id, t.amount.unwrap(), t.kind.is_deposit())?;
                        self.set_tx(tx.client_id, t.with_state(TransactionState::Dispute));
                        Ok(())
                    } else {
                        Err(TransactionError::Dispute.into())
                    }
                } else {
                    Err(TransactionError::NotFound.into())
                }
            }
            TransactionKind::Resolve if tx.amount.is_none() => {
                if let Some(t) = self.get_tx(&tx.client_id, &tx.tx) {
                    if t.can_resolve() {
                        self.account_resolve(tx.client_id, t.amount.unwrap())?;
                        self.set_tx(tx.client_id, t.with_state(TransactionState::Resolved));
                    } else {
                        return Err(TransactionError::Resolve.into());
                    }
                }
                Ok(())
            }
            TransactionKind::Chargeback if tx.amount.is_none() => {
                if let Some(t) = self.get_tx(&tx.client_id, &tx.tx) {
                    if t.can_chargeback() {
                        self.account_chargeback(tx.client_id, t.amount.unwrap())?;
                        self.set_tx(tx.client_id, t.with_state(TransactionState::Chargeback));
                    } else {
                        return Err(TransactionError::Chargeback.into());
                    }
                }
                Ok(())
            }
            _ => Err(TransactionError::UnExpectedAmount.into()),
        }
    }
    fn get_tx(&self, client_id: &u16, tx_id: &u32) -> Option<Transaction> {
        TX.with_borrow_mut(|db| {
            db.get(&client_id)
                .map(|c| c.get(&tx_id))
                .flatten()
                .map(|t| t.clone())
        })
    }
    fn set_tx(&self, client_id: u16, tx: Transaction) -> Option<Transaction> {
        TX.with_borrow_mut(|db| db.entry(client_id).or_default().insert(tx.tx, tx))
    }
    fn account_deposit(&self, client_id: u16, amount: f32) -> Result<(), Error> {
        ACC.with_borrow_mut(|db| {
            db.entry(client_id)
                .or_insert(Account::new(client_id))
                .deposit(amount)
        })
    }
    fn account_withdraw(&self, client_id: u16, amount: f32) -> Result<(), Error> {
        ACC.with_borrow_mut(|db| {
            db.entry(client_id)
                .or_insert(Account::new(client_id))
                .withdraw(amount)
        })
    }
    fn account_dispute(&self, client_id: u16, amount: f32, is_deposit: bool) -> Result<(), Error> {
        ACC.with_borrow_mut(|db| {
            db.entry(client_id)
                .or_insert(Account::new(client_id))
                .dispute(amount, is_deposit)
        })
    }
    fn account_resolve(&self, client_id: u16, amount: f32) -> Result<(), Error> {
        ACC.with_borrow_mut(|db| {
            db.entry(client_id)
                .or_insert(Account::new(client_id))
                .resolve(amount)
        })
    }
    fn account_chargeback(&self, client_id: u16, amount: f32) -> Result<(), Error> {
        ACC.with_borrow_mut(|db| {
            db.entry(client_id)
                .or_insert(Account::new(client_id))
                .chargeback(amount)
        })
    }
    pub fn accounts(&self) -> HashMap<u16, Account> {
        ACC.take()
    }
    pub fn transactions(&self) -> HashMap<u16, HashMap<u32, Transaction>> {
        TX.take()
    }
    pub fn clean(&self) {
        ACC.set(Default::default());
        TX.set(Default::default());
    }
}

#[cfg(test)]
mod tests {
    use super::{Transaction, TransactionKind, TransactionState, TX};
    use crate::{account::AccountError, db::Db, transaction::TransactionError};
    const CLIENT_ID: u16 = 1;
    const DE_ID: u32 = 1;
    const WI_ID: u32 = 2;

    fn get_deposit_tx() -> Transaction {
        Transaction {
            amount: Some(4.0),
            client_id: CLIENT_ID,
            tx: DE_ID,
            state: TransactionState::Processing,
            kind: TransactionKind::Deposit,
        }
    }
    fn get_withdraw_tx() -> Transaction {
        Transaction {
            amount: Some(1.0),
            client_id: CLIENT_ID,
            tx: WI_ID,
            state: TransactionState::Processing,
            kind: TransactionKind::Withdrawal,
        }
    }
    fn get_dispute_tx() -> Transaction {
        Transaction {
            amount: None,
            client_id: CLIENT_ID,
            tx: WI_ID,
            state: TransactionState::Processing,
            kind: TransactionKind::Dispute,
        }
    }
    fn get_resolve_tx() -> Transaction {
        Transaction {
            amount: None,
            client_id: CLIENT_ID,
            tx: WI_ID,
            state: TransactionState::Processing,
            kind: TransactionKind::Resolve,
        }
    }
    fn get_chargeback_tx() -> Transaction {
        Transaction {
            amount: None,
            client_id: CLIENT_ID,
            tx: WI_ID,
            state: TransactionState::Processing,
            kind: TransactionKind::Chargeback,
        }
    }
    fn transactions_len() -> usize {
        TX.with_borrow(|db| db.get(&CLIENT_ID).map(|t| t.len()))
            .unwrap_or(0)
    }

    #[test]
    fn test_deposit_process() {
        let tx = get_deposit_tx();
        assert!(tx.is_valid());
        let db = Db::default();
        db.clean();

        let result = db.process(tx);
        assert_eq!(Ok(()), result);
        let txs = db.transactions();
        assert_eq!(1, txs.len());
        let acc_tx = txs.get(&CLIENT_ID);
        assert!(acc_tx.is_some());
        let acc_txs = acc_tx.unwrap();
        assert_eq!(1, acc_txs.len());
        let acc_tx = acc_txs.get(&DE_ID);
        assert!(acc_tx.is_some());
        let acc_tx = acc_tx.unwrap();
        assert_eq!(
            acc_tx,
            &get_deposit_tx().with_state(TransactionState::Completed)
        );
    }
    #[test]
    fn test_withdraw_process() {
        let tx = get_deposit_tx();
        assert!(tx.is_valid());
        let db = Db::default();
        db.clean();

        let result = db.process(tx);
        assert_eq!(Ok(()), result);
        let tx = get_withdraw_tx();
        let result = db.process(tx);
        assert_eq!(Ok(()), result);
        let txs = db.transactions();
        assert_eq!(1, txs.len());
    }
    #[test]
    fn test_withdraw_process_err() {
        let db = Db::default();
        db.clean();

        let tx = get_withdraw_tx();
        assert!(tx.is_valid());
        let result = db.process(tx);
        assert_eq!(Err(AccountError::Withdraw.into()), result);
        let txs = db.transactions();
        assert_eq!(0, txs.len());
    }
    #[test]
    fn test_disput_process() {
        let db = Db::default();
        db.clean();

        let tx = get_deposit_tx();
        assert!(tx.is_valid());
        let result = db.process(tx);
        assert_eq!(Ok(()), result);
        assert_eq!(1, transactions_len());
        let tx = get_withdraw_tx();
        assert!(tx.is_valid());
        let result = db.process(tx);
        assert_eq!(Ok(()), result);
        assert_eq!(2, transactions_len());

        let tx = get_dispute_tx();
        assert!(tx.is_valid());
        let result = db.process(tx);
        assert_eq!(Ok(()), result);
        assert_eq!(2, transactions_len());

        let tx = get_dispute_tx();
        assert!(tx.is_valid());
        let result = db.process(tx);
        assert_eq!(Err(TransactionError::Dispute.into()), result);
        assert_eq!(2, transactions_len());
    }
    #[test]
    fn test_resolve_process() {
        let db = Db::default();
        db.clean();

        let tx = get_deposit_tx();
        assert!(tx.is_valid());
        let result = db.process(tx);
        assert_eq!(Ok(()), result);
        assert_eq!(1, transactions_len());
        let tx = get_withdraw_tx();
        assert!(tx.is_valid());
        let result = db.process(tx);
        assert_eq!(Ok(()), result);
        assert_eq!(2, transactions_len());

        let tx = get_dispute_tx();
        assert!(tx.is_valid());
        let result = db.process(tx);
        assert_eq!(Ok(()), result);
        assert_eq!(2, transactions_len());

        let tx = get_resolve_tx();
        assert!(tx.is_valid());
        let result = db.process(tx);
        assert_eq!(Ok(()), result);
        assert_eq!(2, transactions_len());

        let tx = get_resolve_tx();
        assert!(tx.is_valid());
        let result = db.process(tx);
        assert_eq!(Err(TransactionError::Resolve.into()), result);
        assert_eq!(2, transactions_len());
    }
    #[test]
    fn test_chargeback_process() {
        let db = Db::default();
        db.clean();

        let tx = get_deposit_tx();
        assert!(tx.is_valid());
        let result = db.process(tx);
        assert_eq!(Ok(()), result);
        assert_eq!(1, transactions_len());
        let tx = get_withdraw_tx();
        assert!(tx.is_valid());
        let result = db.process(tx);
        assert_eq!(Ok(()), result);
        assert_eq!(2, transactions_len());

        let tx = get_dispute_tx();
        assert!(tx.is_valid());
        let result = db.process(tx);
        assert_eq!(Ok(()), result);
        assert_eq!(2, transactions_len());

        let tx = get_chargeback_tx();
        assert!(tx.is_valid());
        let result = db.process(tx);
        assert_eq!(Ok(()), result);
        assert_eq!(2, transactions_len());

        let tx = get_chargeback_tx();
        assert!(tx.is_valid());
        let result = db.process(tx);
        assert_eq!(Err(TransactionError::Chargeback.into()), result);
        assert_eq!(2, transactions_len());
    }
}
