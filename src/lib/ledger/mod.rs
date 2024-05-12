use super::{
    account::Account, account::ClientId, account::Number, transactions::Operation,
    transactions::Transaction, transactions::TransactionError, transactions::TransactionId,
    transactions::TransactionResult, transactions::TransactionState,
};

use std::collections::HashMap;

type AccountMap = HashMap<ClientId, Account>;
type TransactionMap = HashMap<TransactionId, Transaction>;

pub struct Ledger {
    accounts: AccountMap,
    transactions: TransactionMap,
}

impl Default for Ledger {
    fn default() -> Self {
        Self::new()
    }
}

impl Ledger {
    pub fn new() -> Ledger {
        Ledger {
            accounts: AccountMap::with_capacity(u16::MAX as usize),
            transactions: TransactionMap::with_capacity(128),
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
        if self.transactions.contains_key(&transaction_id) {
            Err(TransactionError::RepeatedTransactionId(transaction_id))
        } else {
            Ok(())
        }
    }
    pub fn apply_transaction(
        &mut self,
        transaction_id: TransactionId,
        transaction: &Transaction,
    ) -> TransactionResult {
        if transaction.amount() < Number::ZERO {
            return Err(TransactionError::InvalidAmount(
                transaction_id,
                transaction.amount(),
            ));
        }
        match transaction.operation() {
            Operation::Deposit => {
                self.id_exists(transaction_id)?;
                let account = self.get_or_insert_account_mut(transaction.client_id());
                account
                    .deposit(transaction.amount())
                    .map_err(|err| TransactionError::AccountError(transaction.client_id(), err))?;
                self.transactions.insert(transaction_id, *transaction);
                Ok(())
            }
            Operation::Withdrawal => {
                self.id_exists(transaction_id)?;
                let account = self.get_or_insert_account_mut(transaction.client_id());
                account
                    .withdraw(transaction.amount())
                    .map_err(|err| TransactionError::AccountError(transaction.client_id(), err))?;
                self.transactions.insert(transaction_id, *transaction);
                Ok(())
            }
            Operation::Dispute => {
                let (disputed_transaction, account) =
                    self.get_transaction_and_account_mut(transaction_id, transaction.client_id())?;
                transaction.check_valid_dispute(transaction_id, disputed_transaction)?;
                disputed_transaction.state_matches_or(
                    TransactionState::Ok,
                    TransactionError::AlreadyDisputed(transaction_id),
                )?;
                disputed_transaction.dispute(account)
            }
            Operation::Resolve => {
                let (disputed_transaction, account) =
                    self.get_transaction_and_account_mut(transaction_id, transaction.client_id())?;
                transaction.check_valid_dispute(transaction_id, disputed_transaction)?;
                disputed_transaction.state_matches_or(
                    TransactionState::Disputed,
                    TransactionError::UndisputedTransaction(transaction_id),
                )?;
                disputed_transaction.resolve(account)
            }
            Operation::Chargeback => {
                let (disputed_transaction, account) =
                    self.get_transaction_and_account_mut(transaction_id, transaction.client_id())?;
                transaction.check_valid_dispute(transaction_id, disputed_transaction)?;
                disputed_transaction.state_matches_or(
                    TransactionState::Disputed,
                    TransactionError::UndisputedTransaction(transaction_id),
                )?;
                disputed_transaction.chargeback(account)
            }
        }
    }
}

impl IntoIterator for Ledger {
    type Item = <AccountMap as IntoIterator>::Item;
    type IntoIter = <AccountMap as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.accounts.into_iter()
    }
}

#[cfg(test)]
mod tests;
