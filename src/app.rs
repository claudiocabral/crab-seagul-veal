use std::{fs, io, sync::mpsc, thread};

use crab::account::{ClientId, Number};
use crab::ledger::Ledger;
use crab::transactions::{Operation, Transaction, TransactionId};

#[cfg(test)]
#[path = "tests/test.rs"]
mod tests;

fn create_reader(path: &String) -> csv::Reader<io::BufReader<fs::File>> {
    let file = fs::File::open(path).unwrap();
    let reader = io::BufReader::new(file);
    csv::Reader::from_reader(reader)
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

impl From<TransactionType> for Operation {
    fn from(value: TransactionType) -> Self {
        match value {
            TransactionType::Deposit => Operation::Deposit,
            TransactionType::Withdrawal => Operation::Withdrawal,
            TransactionType::Dispute => Operation::Dispute,
            TransactionType::Resolve => Operation::Resolve,
            TransactionType::Chargeback => Operation::Chargeback,
        }
    }
}

#[derive(serde::Deserialize)]
struct CsvTransactionRecord {
    #[serde(rename = "type")]
    tx_type: TransactionType,
    client: u16,
    tx: u32,
    amount: Option<Number>,
}

#[derive(serde::Serialize)]
struct CsvAccountRecord {
    client: u16,
    available: String,
    held: String,
    total: String,
    locked: bool,
}

fn process(
    ledger: &mut Ledger,
    transaction_id: TransactionId,
    transaction: &Transaction,
    print_error: bool,
) {
    match ledger.apply_transaction(transaction_id, transaction) {
        Ok(()) => {}
        Err(err) => {
            if print_error {
                eprintln!("error: {:?}", err);
            }
        }
    };
}

fn process_transactions(
    rx_channel: mpsc::Receiver<CsvTransactionRecord>,
    debug: bool,
    ledger: &mut Ledger,
) {
    while let Ok(record) = rx_channel.recv() {
        let transaction_id = TransactionId(record.tx);
        let amount = record.amount.unwrap_or_default();
        let client_id = ClientId(record.client);
        let operation = Operation::from(record.tx_type);
        process(
            ledger,
            transaction_id,
            &Transaction::new(client_id, amount, operation),
            debug,
        )
    }
}

pub fn process_file(filename: &String, debug: bool) -> Ledger {
    let mut reader = create_reader(filename);
    let (tx, rx) = mpsc::channel();
    let handler = thread::spawn(move || {
        let mut ledger = Ledger::new();
        process_transactions(rx, debug, &mut ledger);
        ledger
    });
    for record in reader.deserialize::<CsvTransactionRecord>().flatten() {
        let _ = tx.send(record);
    }
    drop(tx);
    handler.join().unwrap()
}
pub fn app(filename: &String, debug: bool) {
    let ledger = process_file(filename, debug);
    let mut writer = csv::WriterBuilder::new().from_writer(io::BufWriter::new(io::stdout()));
    for (key, account) in ledger {
        let val = CsvAccountRecord {
            client: key.0,
            available: format!("{:.4}", account.available()),
            held: format!("{:.4}", account.held()),
            total: format!("{:.4}", account.total()),
            locked: account.locked(),
        };
        let _ = writer.serialize(val);
    }
}
