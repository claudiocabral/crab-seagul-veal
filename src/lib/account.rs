pub type Number = rust_decimal::Decimal;
pub use rust_decimal_macros::dec as num;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Default)]
pub struct ClientId(pub u16);

#[derive(Debug, PartialEq)]
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

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Account {
    available: Number,
    held: Number,
    locked: bool,
}

impl Account {
    pub fn total(&self) -> Number {
        self.available + self.held
    }
    pub fn available(&self) -> Number {
        self.available
    }
    pub fn held(&self) -> Number {
        self.held
    }
    pub fn locked(&self) -> bool {
        self.locked
    }
    pub fn check_locked(&mut self) -> AccountResult {
        match self.locked {
            true => Err(AccountError::FrozenAccount(*self)),
            false => Ok(()),
        }
    }
    pub fn deposit(&mut self, amount: Number) -> AccountResult {
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
        let new_available = self.available.checked_sub(amount);
        let new_held = self.held.checked_add(amount);
        match (new_available, new_held) {
            (Some(available), Some(held)) => {
                self.available = available;
                self.held = held;
                Ok(())
            }
            (Some(_), None) | (None, None) => Err(AccountError::Underflow {
                available: self.available,
                held: self.held,
                transaction_amount: amount,
            }),
            (None, Some(_)) => Err(AccountError::Overflow {
                available: self.available,
                held: self.held,
                transaction_amount: amount,
            }),
        }
    }
    pub fn resolve(&mut self, amount: Number) -> AccountResult {
        let new_available = self.available.checked_add(amount);
        let new_held = self.held.checked_sub(amount);
        match (new_available, new_held) {
            (Some(available), Some(held)) => {
                self.available = available;
                self.held = held;
                Ok(())
            }
            (Some(_), None) | (None, None) => Err(AccountError::Overflow {
                available: self.available,
                held: self.held,
                transaction_amount: amount,
            }),
            (None, Some(_)) => Err(AccountError::Underflow {
                available: self.available,
                held: self.held,
                transaction_amount: amount,
            }),
        }
    }
    pub fn chargeback(&mut self, amount: Number) {
        self.held -= amount;
        self.locked = true;
    }
}

#[cfg(test)]
mod account_tests {
    use super::num;
    use super::Number;

    #[test]
    fn verify_precision() {
        let mut a = Number::ZERO;
        for _ in 0..10_000 {
            a += num!(0.0001);
        }
        assert_eq!(a, num!(1.0));
        for _ in 0..10_000 {
            a -= num!(0.0001);
        }
        assert_eq!(a, num!(0.0));
    }
}
