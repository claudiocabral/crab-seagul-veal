use super::super::{
    account::ClientId, account::Number, ledger::Ledger, transactions::Operation,
    transactions::Transaction, transactions::TransactionError, transactions::TransactionId,
};
use super::TransactionResult;

type TransactionList = Vec<(TransactionId, Transaction)>;

fn process_transactions<'a>(
    ledger: &'a mut Ledger,
    transactions: &'a TransactionList,
) -> impl Iterator<Item = TransactionResult> + 'a {
    transactions.into_iter().map(move |t| {
        let (id, transaction) = t;
        ledger.apply_transaction(*id, transaction)
    })
}

#[test]
fn test_simple_deposit() {
    let mut ledger = Ledger::new();
    let transactions: Vec<(TransactionId, Transaction)> = vec![(
        TransactionId(1),
        Transaction::new(ClientId(1), Number::from_num(50.0), Operation::Deposit),
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
    assert_eq!(ledger.accounts.get(&ClientId(1)).unwrap().available(), 50.0);
    assert_eq!(ledger.accounts.get(&ClientId(1)).unwrap().held(), 0.0);
    assert_eq!(ledger.accounts.get(&ClientId(1)).unwrap().locked(), false);
    assert_eq!(ledger.transactions.len(), 1);
    let transaction = ledger.transactions.get(&TransactionId(1)).unwrap();
    assert_eq!(transaction.disputed(), false);
}

#[test]
fn test_simple_withdrawal() {
    let mut ledger = Ledger::new();
    let transactions: Vec<(TransactionId, Transaction)> = vec![
        (
            TransactionId(1),
            Transaction::new(ClientId(1), Number::from_num(1.0), Operation::Deposit),
        ),
        (
            TransactionId(2),
            Transaction::new(ClientId(1), Number::from_num(0.9999), Operation::Withdrawal),
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
        Number::from_num(0.0001)
    );
    assert_eq!(ledger.accounts.get(&ClientId(1)).unwrap().held(), 0.0);
    assert_eq!(ledger.accounts.get(&ClientId(1)).unwrap().locked(), false);
    assert_eq!(ledger.transactions.len(), 2);
    let transaction = ledger.transactions.get(&TransactionId(1)).unwrap();
    assert_eq!(transaction.disputed(), false);
}

#[test]
fn test_simple_dispute() {
    let mut ledger = Ledger::new();
    let transactions: Vec<(TransactionId, Transaction)> = vec![
        (
            TransactionId(1),
            Transaction::new(ClientId(1), Number::from_num(50.0), Operation::Deposit),
        ),
        (
            TransactionId(2),
            Transaction::new(ClientId(1), Number::from_num(20.0), Operation::Deposit),
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
    assert_eq!(ledger.accounts.get(&ClientId(1)).unwrap().available(), 20.0);
    assert_eq!(ledger.accounts.get(&ClientId(1)).unwrap().held(), 50.0);
    assert_eq!(ledger.accounts.get(&ClientId(1)).unwrap().locked(), false);
    assert_eq!(ledger.transactions.len(), 2);
    let transaction = ledger.transactions.get(&TransactionId(1)).unwrap();
    assert_eq!(transaction.disputed(), true);
}

#[test]
fn test_simple_resolve() {
    let mut ledger = Ledger::new();
    let transactions: Vec<(TransactionId, Transaction)> = vec![
        (
            TransactionId(1),
            Transaction::new(ClientId(1), Number::from_num(35.0), Operation::Deposit),
        ),
        (
            TransactionId(2),
            Transaction::new(ClientId(1), Number::from_num(35.0), Operation::Deposit),
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
    assert_eq!(ledger.accounts.get(&ClientId(1)).unwrap().available(), 70.0);
    assert_eq!(ledger.accounts.get(&ClientId(1)).unwrap().held(), 0.0);
    assert_eq!(ledger.accounts.get(&ClientId(1)).unwrap().locked(), false);
    assert_eq!(ledger.transactions.len(), 2);
    let transaction = ledger.transactions.get(&TransactionId(2)).unwrap();
    assert_eq!(transaction.disputed(), false);
}

#[test]
fn test_simple_chargeback() {
    let mut ledger = Ledger::new();
    let transactions: Vec<(TransactionId, Transaction)> = vec![
        (
            TransactionId(1),
            Transaction::new(ClientId(1), Number::from_num(40.0), Operation::Deposit),
        ),
        (
            TransactionId(2),
            Transaction::new(ClientId(1), Number::from_num(20.0), Operation::Deposit),
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
    assert_eq!(ledger.accounts.get(&ClientId(1)).unwrap().available(), 40.0);
    assert_eq!(ledger.accounts.get(&ClientId(1)).unwrap().held(), 0.0);
    assert_eq!(ledger.accounts.get(&ClientId(1)).unwrap().locked(), true);
    assert_eq!(ledger.transactions.len(), 2);
    let transaction = ledger.transactions.get(&TransactionId(2)).unwrap();
    assert!(transaction.disputed() == false);
}

#[test]
fn test_dispute_after_withdraw() {
    let mut ledger = Ledger::new();
    let transactions: Vec<(TransactionId, Transaction)> = vec![
        (
            TransactionId(1),
            Transaction::new(ClientId(1), Number::from_num(1.0), Operation::Deposit),
        ),
        (
            TransactionId(2),
            Transaction::new(ClientId(1), Number::from_num(1.0), Operation::Withdrawal),
        ),
        (
            TransactionId(1),
            Transaction::new(ClientId(1), Number::ZERO, Operation::Dispute),
        ),
    ];
    let res = process_transactions(&mut ledger, &transactions).all(|res| res.is_ok());
    assert_eq!(res, false);
    assert_eq!(ledger.accounts.get(&ClientId(1)).unwrap().available(), 0.0);
    assert_eq!(ledger.accounts.get(&ClientId(1)).unwrap().held(), 0.0);
    assert_eq!(ledger.accounts.get(&ClientId(1)).unwrap().locked(), true);
}

#[test]
fn test_cant_withdrawal_with_same_id() {
    let mut ledger = Ledger::new();
    let _ = ledger.apply_transaction(
        TransactionId(0),
        &Transaction::new(ClientId(1), Number::ONE, Operation::Deposit),
    );
    let _ = ledger.apply_transaction(
        TransactionId(1),
        &Transaction::new(ClientId(1), Number::from_num(0.5), Operation::Withdrawal),
    );
    let res = ledger.apply_transaction(
        TransactionId(1),
        &Transaction::new(ClientId(1), Number::from_num(0.5), Operation::Withdrawal),
    );
    assert_eq!(
        res.err().unwrap(),
        TransactionError::RepeatedTransactionId(TransactionId(1))
    );
    assert_eq!(ledger.accounts.get(&ClientId(1)).unwrap().available(), 0.5,);
}

#[test]
fn test_cant_deposit_with_same_id() {
    let mut ledger = Ledger::new();
    let _ = ledger.apply_transaction(
        TransactionId(0),
        &Transaction::new(ClientId(1), Number::ONE, Operation::Deposit),
    );
    let res = ledger.apply_transaction(
        TransactionId(0),
        &Transaction::new(ClientId(1), Number::from_num(0.5), Operation::Deposit),
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
