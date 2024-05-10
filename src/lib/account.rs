use fixed::types::extra::*;
use fixed::FixedU64;

pub type Number = FixedU64<U14>;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Default)]
pub struct ClientId(pub u16);

#[derive(Debug)]
pub enum AccountError {
    Overflow {
        available: Number,
        held: Number,
        transaction_amount: Number,
    },
    Underflow {
        available: Number,
        held: Number,
        transaction_amount: Number,
    },
    FrozenAccount(Account),
}

pub type AccountResult = Result<(), AccountError>;

#[derive(Copy, Clone, Default, Debug)]
pub struct Account {
    pub available: Number,
    pub held: Number,
    pub locked: bool,
}

impl Account {
    pub fn total(&self) -> Number {
        self.available + self.held
    }
    pub fn check_locked(&mut self) -> AccountResult {
        match self.locked {
            true => Err(AccountError::FrozenAccount(*self)),
            false => Ok(()),
        }
    }
    pub fn deposit(&mut self, amount: Number) -> AccountResult {
        self.check_locked()?;
        match self.available.checked_add(amount) {
            Some(value) => {
                self.available = value;
                Ok(())
            }
            None => Err(AccountError::Overflow {
                available: self.available,
                held: self.held,
                transaction_amount: amount,
            }),
        }
    }
    pub fn withdraw(&mut self, amount: Number) -> AccountResult {
        self.check_locked()?;
        if self.available < amount {
            return Err(AccountError::Underflow {
                available: self.available,
                held: self.held,
                transaction_amount: amount,
            });
        }
        self.available -= amount;
        Ok(())
    }
    pub fn dispute(&mut self, amount: Number) -> AccountResult {
        self.check_locked()?;
        let new_available = self.available.checked_sub(amount);
        let new_held = self.held.checked_add(amount);
        match (new_available, new_held) {
            (Some(available), Some(held)) => {
                self.available = available;
                self.held = held;
                Ok(())
            }
            (Some(_), None) | (None, None) => {
                self.locked = true;
                Err(AccountError::Underflow {
                    available: self.available,
                    held: self.held,
                    transaction_amount: amount,
                })
            }
            (None, Some(_)) => {
                self.locked = true;
                Err(AccountError::Overflow {
                    available: self.available,
                    held: self.held,
                    transaction_amount: amount,
                })
            }
        }
    }
    pub fn chargeback(&mut self, amount: Number) {
        self.held -= amount;
        self.locked = true;
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
