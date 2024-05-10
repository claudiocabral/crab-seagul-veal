#![allow(warnings)]

use super::{
    account::Account, account::ClientId, transactions::DisputeEntry, transactions::Operation,
    transactions::Transaction, transactions::TransactionEntry, transactions::TransactionError,
    transactions::TransactionId, transactions::TransactionResult,
};

use std::collections::HashMap;

pub enum TransactionType {
    Dispute,
    Resolve,
    Chargeback,
}

pub struct Ledger {
    accounts: HashMap<ClientId, Account>,
    transactions: HashMap<TransactionId, TransactionEntry>,
}

impl Ledger {
    pub fn new() -> Ledger {
        Ledger {
            accounts: HashMap::with_capacity(u16::MAX as usize),
            transactions: HashMap::with_capacity(128),
        }
    }

    pub fn apply_dispute(
        &mut self,
        transaction_id: TransactionId,
        dispute: &DisputeEntry,
    ) -> TransactionResult {
        let maybe_disputed_transaction = self.transactions.get_mut(&transaction_id);
        if maybe_disputed_transaction.is_none() {
            return Err(TransactionError::UnknownTransactionId(transaction_id));
        }
        let mut disputed_transaction = maybe_disputed_transaction.unwrap();
        let maybe_account = self.accounts.get_mut(&dispute.client_id);
        if maybe_account.is_none() {
            return Err(TransactionError::UnknownClientId(dispute.client_id));
        }
        let account = maybe_account.unwrap();
        dispute.apply(account, transaction_id, disputed_transaction)
    }

    pub fn apply_transaction(
        &mut self,
        transaction_id: TransactionId,
        transaction: &TransactionEntry,
    ) -> TransactionResult {
        if self.transactions.get(&transaction_id).is_some() {
            return Err(TransactionError::RepeatedTransactionId(transaction_id));
        }
        if (transaction.operation == Operation::Deposit) {
            self.transactions.insert(transaction_id, *transaction);
        }
        let mut account = self
            .accounts
            .entry(transaction.client_id)
            .or_insert_with(|| Account {
                ..Default::default()
            });
        transaction.apply(account)
    }

    pub fn process_transaction(
        &mut self,
        transaction_id: TransactionId,
        entry: &Transaction,
    ) -> Result<(), TransactionError> {
        match (entry) {
            Transaction::TransactionEntry(e) => self.apply_transaction(transaction_id, e),
            Transaction::DisputeEntry(e) => self.apply_dispute(transaction_id, e),
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
