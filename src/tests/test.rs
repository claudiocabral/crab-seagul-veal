use crate::app::process_file;
use crab::account::Account;
use crab::account::ClientId;
use std::fs::read_to_string;

// TODO: The serialization to CSV method here is different from the one used in main. These should
// match to prevent breaking changes in serialization in main from happening silently.
// TODO: output files need to be sorted and we're not verifying the CSV header for correctness.
#[test]
fn check_csv_files() {
    let files = [
        "00-bad_header",
        "01-bad_record",
        "02-sample",
        "03-10k_records",
    ];
    for file in files {
        let input_file = format!("src/tests/data/{file}-input.csv");
        let output_file = format!("src/tests/data/{file}-output.csv");
        let ledger = process_file(&input_file, false);
        let mut results: Vec<(ClientId, Account)> = ledger.into_iter().collect();
        let references: Vec<String> = read_to_string(output_file)
            .unwrap() // panic on possible file-reading errors
            .lines() // split the string into an iterator of string slices
            .map(String::from) // make each slice into a string
            .skip(1)
            .collect();
        results.sort_by_key(|(key, _)| *key);
        for ((key, account), reference) in results.into_iter().zip(references) {
            let serialized = format!(
                "{},{:.4},{:.4},{:.4},{}",
                key.0,
                account.available(),
                account.held(),
                account.total(),
                account.locked(),
            );
            assert_eq!(serialized, reference, "mismatched result on file {file}");
        }
    }
}
