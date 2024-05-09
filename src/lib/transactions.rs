use super::account::{Account, ClientId, Number};

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct TransactionId(pub u32);

#[derive(Debug, PartialEq)]
pub enum TransactionError {
    Overdraw(Number, Number),
    RepeatedTransactionId(TransactionId),
    UnknownTransactionId(TransactionId),
    UnknownClientId(ClientId),
    MismatchedClientId(ClientId, ClientId),
    AlreadyDisputed(TransactionId),
    UndisputedDispute(Transaction),
    FrozenTransaction(Transaction),
    Overflow {
        available: Number,
        held: Number,
        transaction_amount: Number,
        maximum: Number,
    },
}
pub type TransactionResult = Result<(), TransactionError>;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Operation {
    Deposit,
    Withdrawal,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum DisputeOperation {
    Dispute,
    Chargeback,
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
    pub fn apply(&self, account: &mut Account) -> TransactionResult {
        if account.locked {
            return Err(TransactionError::FrozenTransaction(
                Transaction::TransactionEntry(*self),
            ));
        }
        match &self.operation {
            Operation::Deposit => self.deposit(account),
            Operation::Withdrawal => self.withdraw(account),
        }
    }
    fn deposit(&self, account: &mut Account) -> TransactionResult {
        match account.available.checked_add(self.amount) {
            Some(value) => {
                account.available = value;
                Ok(())
            }
            None => Err(TransactionError::Overflow {
                available: account.available,
                held: account.held,
                transaction_amount: self.amount,
                maximum: Number::MAX,
            }),
        }
    }
    fn withdraw(&self, account: &mut Account) -> TransactionResult {
        if account.available < self.amount {
            return Err(TransactionError::Overdraw(account.available, self.amount));
        }
        account.available -= self.amount;
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
        self.check_valid_dispute(account, transaction_id, transaction)?;
        match &self.operation {
            DisputeOperation::Dispute => self.dispute(account, transaction),
            DisputeOperation::Resolve => self.resolve(account, transaction),
            DisputeOperation::Chargeback => self.chargeback(account, transaction),
        }
    }
    fn check_valid_dispute(
        &self,
        account: &Account,
        transaction_id: TransactionId,
        transaction: &TransactionEntry,
    ) -> TransactionResult {
        if self.client_id != transaction.client_id {
            return Err(TransactionError::MismatchedClientId(
                self.client_id,
                transaction.client_id,
            ));
        }
        if account.locked {
            return Err(TransactionError::FrozenTransaction(
                Transaction::DisputeEntry(*self),
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
        let new_available = account.available.checked_sub(transaction.amount);
        let new_held = account.held.checked_add(transaction.amount);
        match (new_available, new_held) {
            (Some(available), Some(held)) => {
                account.available = available;
                account.held = held;
                transaction.disputed = true;
                Ok(())
            }
            _ => {
                account.locked = true;
                Err(TransactionError::Overflow {
                    available: account.available,
                    held: account.held,
                    transaction_amount: transaction.amount,
                    maximum: Number::MAX,
                })
            }
        }
    }

    fn resolve(
        &self,
        account: &mut Account,
        transaction: &mut TransactionEntry,
    ) -> TransactionResult {
        let new_available = account.available.checked_add(transaction.amount);
        let new_held = account.held.checked_sub(transaction.amount);
        match (new_available, new_held) {
            (Some(available), Some(held)) => {
                account.available = available;
                account.held = held;
                transaction.disputed = false;
                Ok(())
            }
            _ => {
                account.locked = true;
                Err(TransactionError::Overflow {
                    available: account.available,
                    held: account.held,
                    transaction_amount: transaction.amount,
                    maximum: Number::MAX,
                })
            }
        }
    }

    fn chargeback(
        &self,
        account: &mut Account,
        transaction: &mut TransactionEntry,
    ) -> TransactionResult {
        account.held -= transaction.amount;
        account.locked = true;
        transaction.disputed = false;
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub enum Transaction {
    TransactionEntry(TransactionEntry),
    DisputeEntry(DisputeEntry),
}
