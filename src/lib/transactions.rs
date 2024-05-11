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
    UndisputedTransaction(TransactionId),
    AccountError(ClientId, AccountError),
    InvalidAmount(TransactionId, Number),
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

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub enum TransactionState {
    #[default]
    Ok,
    Disputed,
    Chargedback,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Transaction {
    client_id: ClientId,
    amount: Number,
    state: TransactionState,
    operation: Operation,
}

impl Transaction {
    pub fn new(client_id: ClientId, amount: Number, operation: Operation) -> Self {
        Self {
            amount,
            client_id,
            operation,
            state: TransactionState::default(),
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
    pub fn state(&self) -> TransactionState {
        self.state
    }
    pub fn dispute(&mut self, account: &mut Account) -> TransactionResult {
        account
            .dispute(self.amount)
            .map_err(|err| TransactionError::AccountError(self.client_id(), err))?;
        self.state = TransactionState::Disputed;
        Ok(())
    }

    pub fn resolve(&mut self, account: &mut Account) -> TransactionResult {
        account
            .resolve(self.amount)
            .map_err(|err| TransactionError::AccountError(self.client_id(), err))?;
        self.state = TransactionState::Ok;
        Ok(())
    }

    pub fn chargeback(&mut self, account: &mut Account) -> TransactionResult {
        account.chargeback(self.amount);
        self.state = TransactionState::Chargedback;
        Ok(())
    }

    pub fn check_valid_dispute(
        &self,
        transaction_id: TransactionId,
        transaction: &Transaction,
    ) -> TransactionResult {
        if transaction.operation != Operation::Deposit {
            return Err(TransactionError::AlreadyDisputed(transaction_id));
        }
        if self.client_id != transaction.client_id {
            return Err(TransactionError::MismatchedClientId(
                self.client_id,
                transaction.client_id,
            ));
        }

        // this could be condensed in a single clever if block, but I think this is more readable
        if self.operation == Operation::Dispute {
            if transaction.state != TransactionState::Ok {
                return Err(TransactionError::AlreadyDisputed(transaction_id));
            }
        } else if transaction.state != TransactionState::Disputed {
            return Err(TransactionError::UndisputedTransaction(transaction_id));
        }
        Ok(())
    }
}
