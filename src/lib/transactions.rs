use super::account::{Account, ClientId, Number};
use crate::account::AccountError;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct TransactionId(pub u32);

#[derive(Debug, PartialEq)]
pub enum TransactionError {
    RepeatedTransactionId(TransactionId),
    UnknownTransactionId(TransactionId),
    UnknownClientId(ClientId),
    MismatchedClientId(ClientId, ClientId),
    AlreadyDisputed(TransactionId),
    UndisputedDispute(Transaction),
    UndisputedTransaction(Transaction),
    AccountError(AccountError),
}
pub type TransactionResult = Result<(), TransactionError>;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Operation {
    Deposit,
    Withdrawal,
    Dispute,
    Chargeback,
    Resolve,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Transaction {
    client_id: ClientId,
    amount: Number,
    disputed: bool,
    operation: Operation,
}

impl Transaction {
    pub fn new(client_id: ClientId, amount: Number, operation: Operation) -> Self {
        Self {
            amount,
            client_id,
            operation,
            disputed: false,
        }
    }
    pub fn operation(&self) -> Operation {
        self.operation
    }
    pub fn amount(&self) -> Number {
        self.amount
    }
    pub fn client_id(&self) -> ClientId {
        self.client_id
    }
    pub fn disputed(&self) -> bool {
        self.disputed
    }
    pub fn dispute(&mut self, account: &mut Account) -> TransactionResult {
        match account.dispute(self.amount) {
            Ok(_) => {
                self.disputed = true;
                Ok(())
            }
            Err(err) => Err(TransactionError::AccountError(err)),
        }
    }

    pub fn resolve(&mut self, account: &mut Account) -> TransactionResult {
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

    pub fn check_valid_dispute(
        &self,
        transaction_id: TransactionId,
        transaction: &Transaction,
    ) -> TransactionResult {
        if self.client_id != transaction.client_id {
            return Err(TransactionError::MismatchedClientId(
                self.client_id,
                transaction.client_id,
            ));
        }

        // this could be condensed in a single clever if block, but I think this is more readable
        if self.operation == Operation::Dispute {
            if transaction.disputed {
                return Err(TransactionError::AlreadyDisputed(transaction_id));
            }
        } else {
            if !transaction.disputed {
                return Err(TransactionError::UndisputedTransaction(*self));
            }
        }
        Ok(())
    }
}
