use super::transactions::TransactionError;
use super::transactions::TransactionResult;
use fixed::types::extra::*;
use fixed::FixedU64;

pub type Number = FixedU64<U14>;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Default)]
pub struct ClientId(pub u16);

#[derive(Copy, Clone, Default)]
pub struct Account {
    pub available: Number,
    pub held: Number,
    pub locked: bool,
}

impl Account {
    pub fn total(&self) -> Number {
        self.available + self.held
    }
    pub fn deposit(&mut self, amount: Number) -> TransactionResult {
        match self.available.checked_add(amount) {
            Some(value) => {
                self.available = value;
                Ok(())
            }
            None => Err(TransactionError::Overflow {
                available: self.available,
                held: self.held,
                transaction_amount: amount,
            }),
        }
    }
    pub fn withdraw(&mut self, amount: Number) -> TransactionResult {
        if self.available < amount {
            return Err(TransactionError::Underflow {
                available: self.available,
                held: self.held,
                transaction_amount: amount,
            });
        }
        self.available -= amount;
        Ok(())
    }
}

#[cfg(test)]
mod account_tests {
    use super::Number;

    #[test]
    fn verify_precision() {
        /*
         * these asserts guarantee that we meet the minimum precision required
         * and give us a quick overview of the highest value we support.
         * Changing the precision will trigger a failure and require these tests to be
         * updated to reflect the new delta and maximum values.
         * the current maximum value is over 1 quadrilion and should be enough to support
         * accounts even in extremely inflated currencies
         */
        assert!(Number::DELTA <= 0.0001);
        assert_eq!(Number::MAX.floor(), 1_125_899_906_842_623.0);
    }
}
