use super::super::{
    account::ClientId, account::Number, ledger::Ledger, transactions::DisputeEntry,
    transactions::DisputeOperation, transactions::Operation, transactions::Transaction,
    transactions::TransactionEntry, transactions::TransactionId,
};
use super::*;

type TransactionList = Vec<(TransactionId, Transaction)>;
fn process_transactions(ledger: &mut Ledger, transactions: &TransactionList) {
    for (i, t) in transactions.iter().enumerate() {
        let (id, transaction) = t;
        let res = ledger.process_transaction(*id, transaction);
        assert!(
            res.is_ok(),
            "transaction '{}' result is not ok: {:?}",
            i,
            res.unwrap_err()
        );
    }
}

#[test]
fn test_simple_dispute() {
    let mut ledger = Ledger::new();
    let transactions: Vec<(TransactionId, Transaction)> = vec![
        (
            TransactionId(1),
            Transaction::TransactionEntry(TransactionEntry {
                client_id: ClientId(1),
                amount: Number::from_num(50.0),
                operation: Operation::Deposit,
                disputed: false,
            }),
        ),
        (
            TransactionId(2),
            Transaction::TransactionEntry(TransactionEntry {
                client_id: ClientId(1),
                amount: Number::from_num(20.0),
                operation: Operation::Deposit,
                disputed: false,
            }),
        ),
        (
            TransactionId(1),
            Transaction::DisputeEntry(DisputeEntry {
                client_id: ClientId(1),
                operation: DisputeOperation::Dispute,
            }),
        ),
    ];
    process_transactions(&mut ledger, &transactions);
    assert_eq!(ledger.accounts.get(&ClientId(1)).unwrap().available, 20.0);
    assert_eq!(ledger.accounts.get(&ClientId(1)).unwrap().held, 50.0);
    assert_eq!(ledger.transactions.len(), 2);
    let locked_transaction = ledger.transactions.get(&TransactionId(1));
    assert!(locked_transaction.is_some());
    assert!(locked_transaction.unwrap().disputed);
}

#[test]
fn test_dispute_after_withdraw() {
    let mut ledger = Ledger::new();
    let transactions: Vec<(TransactionId, Transaction)> = vec![
        (
            TransactionId(1),
            Transaction::TransactionEntry(TransactionEntry {
                client_id: ClientId(1),
                amount: Number::from_num(1.0),
                operation: Operation::Deposit,
                disputed: false,
            }),
        ),
        (
            TransactionId(2),
            Transaction::TransactionEntry(TransactionEntry {
                client_id: ClientId(1),
                amount: Number::from_num(1.0),
                operation: Operation::Withdrawal,
                disputed: false,
            }),
        ),
        (
            TransactionId(1),
            Transaction::DisputeEntry(DisputeEntry {
                client_id: ClientId(1),
                operation: DisputeOperation::Dispute,
            }),
        ),
    ];
    process_transactions(&mut ledger, &transactions);
    assert_eq!(ledger.accounts.get(&ClientId(1)).unwrap().available, 20.0);
    assert_eq!(ledger.accounts.get(&ClientId(1)).unwrap().held, 50.0);
    assert_eq!(ledger.transactions.len(), 2);
    let locked_transaction = ledger.transactions.get(&TransactionId(1));
    assert!(locked_transaction.is_some());
    assert!(locked_transaction.unwrap().disputed);
}
