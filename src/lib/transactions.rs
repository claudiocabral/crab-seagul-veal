use super::account::{Account, ClientId, Number};
use crate::account::AccountError;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct TransactionId(pub u32);

#[derive(Debug)]
pub enum TransactionError {
    RepeatedTransactionId(TransactionId),
    UnknownTransactionId(TransactionId),
    UnknownClientId(ClientId),
    MismatchedClientId(ClientId, ClientId),
    AlreadyDisputed(TransactionId),
    UndisputedDispute(Transaction),
    FrozenTransaction(Transaction),
    AccountError(AccountError),
    Overflow {
        available: Number,
        held: Number,
        transaction_amount: Number,
    },
    Underflow {
        available: Number,
        held: Number,
        transaction_amount: Number,
    },
}
pub type TransactionResult = Result<(), TransactionError>;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Operation {
    Deposit,
    Withdrawal,
    Chargeback,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum DisputeOperation {
    Dispute,
    Resolve,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TransactionEntry {
    pub client_id: ClientId,
    pub amount: Number,
    pub disputed: bool,
    pub operation: Operation,
}

impl TransactionEntry {
    fn dispute(&mut self, account: &mut Account) -> TransactionResult {
        match account.dispute(self.amount) {
            Ok(_) => {
                self.disputed = true;
                Ok(())
            }
            Err(err) => Err(TransactionError::AccountError(err)),
        }
    }

    fn resolve(&mut self, account: &mut Account) -> TransactionResult {
        match account.resolve(self.amount) {
            Ok(_) => {
                self.disputed = false;
                Ok(())
            }
            Err(err) => Err(TransactionError::AccountError(err)),
        }
    }

    pub fn chargeback(&mut self, account: &mut Account) -> TransactionResult {
        account.chargeback(self.amount);
        self.disputed = false;
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DisputeEntry {
    pub client_id: ClientId,
    pub operation: DisputeOperation,
}

impl DisputeEntry {
    pub fn apply(
        &self,
        account: &mut Account,
        transaction_id: TransactionId,
        transaction: &mut TransactionEntry,
    ) -> TransactionResult {
        self.check_valid_dispute(transaction_id, transaction)?;
        match &self.operation {
            DisputeOperation::Dispute => self.dispute(account, transaction),
            DisputeOperation::Resolve => self.resolve(account, transaction),
        }
    }
    pub fn check_valid_dispute(
        &self,
        transaction_id: TransactionId,
        transaction: &TransactionEntry,
    ) -> TransactionResult {
        if self.client_id != transaction.client_id {
            return Err(TransactionError::MismatchedClientId(
                self.client_id,
                transaction.client_id,
            ));
        }

        // this could be condensed in a single clever if block, but I think this is more readable
        if self.operation == DisputeOperation::Dispute {
            if transaction.disputed {
                return Err(TransactionError::AlreadyDisputed(transaction_id));
            }
        } else {
            if !transaction.disputed {
                return Err(TransactionError::UndisputedDispute(
                    Transaction::DisputeEntry(*self),
                ));
            }
        }
        Ok(())
    }

    fn dispute(
        &self,
        account: &mut Account,
        transaction: &mut TransactionEntry,
    ) -> TransactionResult {
        transaction.dispute(account)
    }

    fn resolve(
        &self,
        account: &mut Account,
        transaction: &mut TransactionEntry,
    ) -> TransactionResult {
        transaction.resolve(account)
    }
}

#[derive(Debug, PartialEq)]
pub enum Transaction {
    TransactionEntry(TransactionEntry),
    DisputeEntry(DisputeEntry),
}
