use super::TransactionResult;
use crate::{
    account::num, account::ClientId, account::Number, ledger::Ledger, transactions::Operation,
    transactions::Transaction, transactions::TransactionError, transactions::TransactionId,
};

type TransactionList = Vec<(TransactionId, Transaction)>;

fn process_transactions<'a>(
    ledger: &'a mut Ledger,
    transactions: &'a TransactionList,
) -> impl Iterator<Item = TransactionResult> + 'a {
    transactions.iter().map(move |t| {
        let (id, transaction) = t;
        ledger.apply_transaction(*id, transaction)
    })
}

#[test]
fn simple_deposit() {
    let mut ledger = Ledger::new();
    let transactions: Vec<(TransactionId, Transaction)> = vec![(
        TransactionId(1),
        Transaction::new(ClientId(1), num!(50.0), Operation::Deposit),
    )];
    process_transactions(&mut ledger, &transactions)
        .enumerate()
        .for_each(|(i, res)| {
            assert!(
                res.is_ok(),
                "transaction '{}' result is not ok: {:?}",
                i,
                res.unwrap_err()
            )
        });
    assert_eq!(
        ledger.accounts.get(&ClientId(1)).unwrap().available(),
        num!(50.0)
    );
    assert_eq!(ledger.accounts.get(&ClientId(1)).unwrap().held(), num!(0.0));
    assert!(!ledger.accounts.get(&ClientId(1)).unwrap().locked());
    assert_eq!(ledger.transactions.len(), 1);
    let transaction = ledger.transactions.get(&TransactionId(1)).unwrap();
    assert!(!transaction.disputed());
}

#[test]
fn simple_withdrawal() {
    let mut ledger = Ledger::new();
    let transactions: Vec<(TransactionId, Transaction)> = vec![
        (
            TransactionId(1),
            Transaction::new(ClientId(1), num!(1.0), Operation::Deposit),
        ),
        (
            TransactionId(2),
            Transaction::new(ClientId(1), num!(0.9999), Operation::Withdrawal),
        ),
    ];
    process_transactions(&mut ledger, &transactions)
        .enumerate()
        .for_each(|(i, res)| {
            assert!(
                res.is_ok(),
                "transaction '{}' result is not ok: {:?}",
                i,
                res.unwrap_err()
            )
        });
    assert_eq!(
        ledger.accounts.get(&ClientId(1)).unwrap().available(),
        num!(0.0001)
    );
    assert_eq!(ledger.accounts.get(&ClientId(1)).unwrap().held(), num!(0.0));
    assert!(!ledger.accounts.get(&ClientId(1)).unwrap().locked());
    assert_eq!(ledger.transactions.len(), 2);
    let transaction = ledger.transactions.get(&TransactionId(1)).unwrap();
    assert!(!transaction.disputed());
}

#[test]
fn simple_dispute() {
    let mut ledger = Ledger::new();
    let transactions: Vec<(TransactionId, Transaction)> = vec![
        (
            TransactionId(1),
            Transaction::new(ClientId(1), num!(50.0), Operation::Deposit),
        ),
        (
            TransactionId(2),
            Transaction::new(ClientId(1), num!(20.0), Operation::Deposit),
        ),
        (
            TransactionId(1),
            Transaction::new(ClientId(1), Number::ZERO, Operation::Dispute),
        ),
    ];
    process_transactions(&mut ledger, &transactions)
        .enumerate()
        .for_each(|(i, res)| {
            assert!(
                res.is_ok(),
                "transaction '{}' result is not ok: {:?}",
                i,
                res.unwrap_err()
            )
        });
    assert_eq!(
        ledger.accounts.get(&ClientId(1)).unwrap().available(),
        num!(20.0)
    );
    assert_eq!(
        ledger.accounts.get(&ClientId(1)).unwrap().held(),
        num!(50.0)
    );
    assert!(!ledger.accounts.get(&ClientId(1)).unwrap().locked());
    assert_eq!(ledger.transactions.len(), 2);
    let transaction = ledger.transactions.get(&TransactionId(1)).unwrap();
    assert!(transaction.disputed());
}

#[test]
fn simple_resolve() {
    let mut ledger = Ledger::new();
    let transactions: Vec<(TransactionId, Transaction)> = vec![
        (
            TransactionId(1),
            Transaction::new(ClientId(1), num!(35.0), Operation::Deposit),
        ),
        (
            TransactionId(2),
            Transaction::new(ClientId(1), num!(35.0), Operation::Deposit),
        ),
        (
            TransactionId(2),
            Transaction::new(ClientId(1), Number::ZERO, Operation::Dispute),
        ),
        (
            TransactionId(2),
            Transaction::new(ClientId(1), Number::ZERO, Operation::Resolve),
        ),
    ];
    process_transactions(&mut ledger, &transactions)
        .enumerate()
        .for_each(|(i, res)| {
            assert!(
                res.is_ok(),
                "transaction '{}' result is not ok: {:?}",
                i,
                res.unwrap_err()
            )
        });
    assert_eq!(
        ledger.accounts.get(&ClientId(1)).unwrap().available(),
        num!(70.0)
    );
    assert_eq!(ledger.accounts.get(&ClientId(1)).unwrap().held(), num!(0.0));
    assert!(!ledger.accounts.get(&ClientId(1)).unwrap().locked());
    assert_eq!(ledger.transactions.len(), 2);
    let transaction = ledger.transactions.get(&TransactionId(2)).unwrap();
    assert!(!transaction.disputed());
}

#[test]
fn cant_resolve_undisputed_transaction() {
    let mut ledger = Ledger::new();
    let deposit = Transaction::new(ClientId(1), num!(0.01), Operation::Deposit);
    let transaction_id = TransactionId(1);
    let _ = ledger.apply_transaction(transaction_id, &deposit);
    let res = ledger.apply_transaction(
        transaction_id,
        &Transaction::new(ClientId(1), Number::ZERO, Operation::Resolve),
    );
    assert_eq!(
        res.unwrap_err(),
        TransactionError::UndisputedTransaction(transaction_id)
    );
    assert_eq!(
        ledger.accounts.get(&ClientId(1)).unwrap().available(),
        num!(0.01)
    );
    assert_eq!(ledger.accounts.get(&ClientId(1)).unwrap().held(), num!(0.0));
    assert!(!ledger.accounts.get(&ClientId(1)).unwrap().locked());
    assert_eq!(ledger.transactions.len(), 1);
}

#[test]
fn cant_chargeback_undisputed_transaction() {
    let mut ledger = Ledger::new();
    let deposit = Transaction::new(ClientId(1), num!(0.01), Operation::Deposit);
    let transaction_id = TransactionId(1);
    let _ = ledger.apply_transaction(transaction_id, &deposit);
    let res = ledger.apply_transaction(
        transaction_id,
        &Transaction::new(ClientId(1), Number::ZERO, Operation::Chargeback),
    );
    assert_eq!(
        res.unwrap_err(),
        TransactionError::UndisputedTransaction(transaction_id)
    );
    assert_eq!(
        ledger.accounts.get(&ClientId(1)).unwrap().available(),
        num!(0.01)
    );
    assert_eq!(ledger.accounts.get(&ClientId(1)).unwrap().held(), num!(0.0));
    assert!(!ledger.accounts.get(&ClientId(1)).unwrap().locked());
    assert_eq!(ledger.transactions.len(), 1);
}

#[test]
fn simple_chargeback() {
    let mut ledger = Ledger::new();
    let transactions: Vec<(TransactionId, Transaction)> = vec![
        (
            TransactionId(1),
            Transaction::new(ClientId(1), num!(40.0), Operation::Deposit),
        ),
        (
            TransactionId(2),
            Transaction::new(ClientId(1), num!(20.0), Operation::Deposit),
        ),
        (
            TransactionId(2),
            Transaction::new(ClientId(1), Number::ZERO, Operation::Dispute),
        ),
        (
            TransactionId(2),
            Transaction::new(ClientId(1), Number::ZERO, Operation::Chargeback),
        ),
    ];
    process_transactions(&mut ledger, &transactions)
        .enumerate()
        .for_each(|(i, res)| {
            assert!(
                res.is_ok(),
                "transaction '{}' result is not ok: {:?}",
                i,
                res.unwrap_err()
            )
        });
    assert_eq!(
        ledger.accounts.get(&ClientId(1)).unwrap().available(),
        num!(40.0)
    );
    assert_eq!(ledger.accounts.get(&ClientId(1)).unwrap().held(), num!(0.0));
    assert!(ledger.accounts.get(&ClientId(1)).unwrap().locked());
    assert_eq!(ledger.transactions.len(), 2);
    let transaction = ledger.transactions.get(&TransactionId(2)).unwrap();
    assert!(!transaction.disputed());
}

#[test]
fn dispute_without_funds() {
    let mut ledger = Ledger::new();
    let transactions: Vec<(TransactionId, Transaction)> = vec![
        (
            TransactionId(1),
            Transaction::new(ClientId(1), num!(1.0), Operation::Deposit),
        ),
        (
            TransactionId(2),
            Transaction::new(ClientId(1), num!(1.0), Operation::Withdrawal),
        ),
        (
            TransactionId(1),
            Transaction::new(ClientId(1), Number::ZERO, Operation::Dispute),
        ),
    ];
    let res = process_transactions(&mut ledger, &transactions).all(|res| res.is_ok());
    assert!(res);
    assert_eq!(
        ledger.accounts.get(&ClientId(1)).unwrap().available(),
        num!(-1.0)
    );
    assert_eq!(ledger.accounts.get(&ClientId(1)).unwrap().held(), num!(1.0));
    assert!(!ledger.accounts.get(&ClientId(1)).unwrap().locked());
}

#[test]
fn cant_withdrawal_with_same_id() {
    let mut ledger = Ledger::new();
    let _ = ledger.apply_transaction(
        TransactionId(0),
        &Transaction::new(ClientId(1), Number::ONE, Operation::Deposit),
    );
    let _ = ledger.apply_transaction(
        TransactionId(1),
        &Transaction::new(ClientId(1), num!(0.5), Operation::Withdrawal),
    );
    let res = ledger.apply_transaction(
        TransactionId(1),
        &Transaction::new(ClientId(1), num!(0.5), Operation::Withdrawal),
    );
    assert_eq!(
        res.err().unwrap(),
        TransactionError::RepeatedTransactionId(TransactionId(1))
    );
    assert_eq!(
        ledger.accounts.get(&ClientId(1)).unwrap().available(),
        num!(0.5)
    );
}

#[test]
fn cant_deposit_with_same_id() {
    let mut ledger = Ledger::new();
    let _ = ledger.apply_transaction(
        TransactionId(0),
        &Transaction::new(ClientId(1), Number::ONE, Operation::Deposit),
    );
    let res = ledger.apply_transaction(
        TransactionId(0),
        &Transaction::new(ClientId(1), num!(0.5), Operation::Deposit),
    );
    assert_eq!(
        res.err().unwrap(),
        TransactionError::RepeatedTransactionId(TransactionId(0))
    );
    assert_eq!(
        ledger.accounts.get(&ClientId(1)).unwrap().available(),
        Number::ONE
    );
}
