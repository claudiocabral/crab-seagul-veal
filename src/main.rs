use clap::Parser;
use std::{fs, io, sync::mpsc, thread};

use crab::account::{ClientId, Number};
use crab::ledger::Ledger;
use crab::transactions::{Operation, Transaction, TransactionEntry, TransactionId};

fn create_reader(path: &String) -> csv::Reader<io::BufReader<fs::File>> {
    let file = fs::File::open(path).unwrap();
    let reader = io::BufReader::new(file);
    csv::Reader::from_reader(reader)
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Arguments {
    filename: String,
    #[arg(short, long, default_value_t = false)]
    debug: bool,
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

#[derive(serde::Deserialize)]
struct CsvTransactionRecord {
    #[serde(rename = "type")]
    tx_type: TransactionType,
    client: u16,
    tx: u32,
    amount: Option<f64>,
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
    match ledger.process_transaction(transaction_id, transaction) {
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
        let amount = record.amount.unwrap_or(0.0);
        match record.tx_type {
            TransactionType::Deposit => process(
                ledger,
                transaction_id,
                &Transaction::TransactionEntry(TransactionEntry {
                    client_id: ClientId(record.client),
                    amount: Number::from_num(amount),
                    operation: Operation::Deposit,
                    disputed: false,
                }),
                debug,
            ),
            TransactionType::Withdrawal => process(
                ledger,
                transaction_id,
                &Transaction::TransactionEntry(TransactionEntry {
                    client_id: ClientId(record.client),
                    amount: Number::from_num(amount),
                    operation: Operation::Withdrawal,
                    disputed: false,
                }),
                debug,
            ),
            TransactionType::Dispute => process(
                ledger,
                transaction_id,
                &Transaction::TransactionEntry(TransactionEntry {
                    client_id: ClientId(record.client),
                    amount: Number::from_num(amount),
                    operation: Operation::Dispute,
                    disputed: false,
                }),
                debug,
            ),
            TransactionType::Resolve => process(
                ledger,
                transaction_id,
                &Transaction::TransactionEntry(TransactionEntry {
                    client_id: ClientId(record.client),
                    amount: Number::ZERO,
                    operation: Operation::Resolve,
                    disputed: false,
                }),
                debug,
            ),
            TransactionType::Chargeback => process(
                ledger,
                transaction_id,
                &Transaction::TransactionEntry(TransactionEntry {
                    client_id: ClientId(record.client),
                    operation: Operation::Chargeback,
                    amount: Number::from_num(amount),
                    disputed: false,
                }),
                debug,
            ),
        }
    }
}

fn main() -> std::thread::Result<()> {
    let args = Arguments::parse();
    let debug = args.debug;
    let mut reader = create_reader(&args.filename);
    let (tx, rx) = mpsc::channel();
    let handler = thread::spawn(move || {
        let mut ledger = Ledger::new();
        process_transactions(rx, debug, &mut ledger);
        ledger
    });
    for result in reader.deserialize::<CsvTransactionRecord>() {
        if result.is_ok() {
            let _ = tx.send(result.unwrap());
        }
    }
    drop(tx);
    let ledger = handler.join()?;
    let mut writer = csv::WriterBuilder::new().from_writer(io::BufWriter::new(io::stdout()));
    for (key, account) in ledger {
        let val = CsvAccountRecord {
            client: key.0,
            available: account.available().to_string(),
            held: account.held().to_string(),
            total: account.total().to_string(),
            locked: account.locked(),
        };
        let _ = writer.serialize(val);
    }
    Ok(())
}
