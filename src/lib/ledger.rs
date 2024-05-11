use super::{
    account::Account, account::ClientId, transactions::Operation, transactions::Transaction,
    transactions::TransactionError, transactions::TransactionId, transactions::TransactionResult,
};

use std::collections::HashMap;

pub struct Ledger {
    accounts: HashMap<ClientId, Account>,
    transactions: HashMap<TransactionId, Transaction>,
}

impl Default for Ledger {
    fn default() -> Self {
        Self::new()
    }
}

impl Ledger {
    pub fn new() -> Ledger {
        Ledger {
            accounts: HashMap::with_capacity(u16::MAX as usize),
            transactions: HashMap::with_capacity(128),
        }
    }

    pub fn get_transaction_and_account_mut(
        &mut self,
        transaction_id: TransactionId,
        client_id: ClientId,
    ) -> Result<(&mut Transaction, &mut Account), TransactionError> {
        let maybe_disputed_transaction = self.transactions.get_mut(&transaction_id);
        if maybe_disputed_transaction.is_none() {
            return Err(TransactionError::UnknownTransactionId(transaction_id));
        }
        let maybe_account = self.accounts.get_mut(&client_id);
        if maybe_account.is_none() {
            return Err(TransactionError::UnknownClientId(client_id));
        }
        Ok((maybe_disputed_transaction.unwrap(), maybe_account.unwrap()))
    }
    pub fn get_or_insert_account_mut(&mut self, client_id: ClientId) -> &mut Account {
        self.accounts.entry(client_id).or_default()
    }

    fn id_exists(&self, transaction_id: TransactionId) -> TransactionResult {
        match self.transactions.contains_key(&transaction_id) {
            true => Err(TransactionError::RepeatedTransactionId(transaction_id)),
            false => Ok(()),
        }
    }
    pub fn apply_transaction(
        &mut self,
        transaction_id: TransactionId,
        transaction: &Transaction,
    ) -> TransactionResult {
        match transaction.operation() {
            Operation::Deposit => {
                self.id_exists(transaction_id)?;
                self.transactions.insert(transaction_id, *transaction);
                let account = self.get_or_insert_account_mut(transaction.client_id());
                match account.deposit(transaction.amount()) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(TransactionError::AccountError(err)),
                }
            }
            Operation::Withdrawal => {
                self.id_exists(transaction_id)?;
                self.transactions.insert(transaction_id, *transaction);
                let account = self.get_or_insert_account_mut(transaction.client_id());
                match account.withdraw(transaction.amount()) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(TransactionError::AccountError(err)),
                }
            }
            Operation::Dispute => {
                let (disputed_transaction, account) =
                    self.get_transaction_and_account_mut(transaction_id, transaction.client_id())?;
                transaction.check_valid_dispute(transaction_id, disputed_transaction)?;
                disputed_transaction.dispute(account)
            }
            Operation::Resolve => {
                let (disputed_transaction, account) =
                    self.get_transaction_and_account_mut(transaction_id, transaction.client_id())?;
                transaction.check_valid_dispute(transaction_id, disputed_transaction)?;
                disputed_transaction.resolve(account)
            }
            Operation::Chargeback => {
                let (disputed_transaction, account) =
                    self.get_transaction_and_account_mut(transaction_id, transaction.client_id())?;
                transaction.check_valid_dispute(transaction_id, disputed_transaction)?;
                disputed_transaction.chargeback(account)
            }
        }
    }
}

impl IntoIterator for Ledger {
    type Item = (ClientId, Account);

    type IntoIter = std::collections::hash_map::IntoIter<ClientId, Account>;

    fn into_iter(self) -> Self::IntoIter {
        self.accounts.into_iter()
    }
}

#[cfg(test)]
#[path = "tests/transaction_tests.rs"]
mod tests;
