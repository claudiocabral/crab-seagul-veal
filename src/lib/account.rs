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
