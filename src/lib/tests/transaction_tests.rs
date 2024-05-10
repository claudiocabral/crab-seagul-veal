use super::super::{
    account::ClientId, account::Number, ledger::Ledger, transactions::Operation,
    transactions::Transaction, transactions::TransactionId,
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
        /*
        assert!(
            res.is_ok(),
            "transaction '{}' result is not ok: {:?}",
            i,
            res.unwrap_err()
        )
        */
    })
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
    assert_eq!(ledger.transactions.len(), 2);
    let locked_transaction = ledger.transactions.get(&TransactionId(1));
    assert!(locked_transaction.is_some());
    assert!(locked_transaction.unwrap().disputed());
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
