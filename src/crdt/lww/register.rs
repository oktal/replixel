//! This module defines an implementation of a CRDT that uses a Last-Write-Wins strategy to merge
//! states together

use crate::crdt::CRDT;

pub struct LWWState<T> {
    value: Option<T>,

    seq: u64,
}

impl<T> LWWState<T> {
    fn update(&mut self, value: T) -> T {
        let old = self.value.take().expect("register *always* holds a value");
        self.value = Some(value);
        self.seq += 1;
        old
    }
}

pub struct LWWRegister<T> {
    state: LWWState<T>,
}

impl<T> LWWRegister<T> {
    /// Creates a new register that holds `value`
    pub fn new(value: T) -> Self {
        Self {
            state: LWWState {
                value: Some(value),
                seq: 1,
            },
        }
    }

    /// Returns a reference to the current version of the value that this register holds
    pub fn value(&self) -> &T {
        self.state
            .value
            .as_ref()
            .expect("register *always* holds a value")
    }

    /// Update the current value with a new value and return the previous value
    pub fn update(&mut self, value: T) -> T {
        self.state.update(value)
    }

    /// Take the current value of the register
    // TODO(oktal): I don't think this function should return an [`Option`] as it should be an
    // invariant of the type that the state is *NEVER* [`None`]
    pub(crate) fn take(mut self) -> Option<T> {
        self.state.value.take()
    }
}

impl<T> From<T> for LWWRegister<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> CRDT for LWWRegister<T> {
    type State = LWWState<T>;

    fn merge(&mut self, other: Self::State) {
        if self.state.seq >= other.seq {
            return;
        }

        self.state.value = other.value;
    }

    fn take(self) -> Self::State {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use crate::crdt::CRDTExt;

    use super::LWWRegister;

    #[test]
    fn create_with_value() {
        let reg = LWWRegister::new(0xC0FFEE);
        assert_eq!(*reg.value(), 0xC0FFEE)
    }

    #[test]
    fn update_and_returns_old_value() {
        let mut reg = LWWRegister::new(0xC0FFEE);
        let old = reg.update(0xBAD);

        assert_eq!(old, 0xC0FFEE);
        assert_eq!(*reg.value(), 0xBAD);
    }

    #[test]
    fn merge_keeps_the_last() {
        let mut recent = LWWRegister::new(0xC0FFEE);
        let mut oldest = LWWRegister::new(0xBAD);

        // Update recent twice
        recent.update(0xCAFFEE);
        recent.update(0xF00D);

        // Update oldest once
        oldest.update(0xDEAD);

        oldest.merge_into(&mut recent);

        // Recent should not have been overwitten as it's the most recent value
        assert_eq!(*recent.value(), 0xF00D);
    }
}
