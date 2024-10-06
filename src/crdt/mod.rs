pub mod lww;

pub trait CRDT {
    type State;

    fn merge(&mut self, other: Self::State);
    fn take(self) -> Self::State;
}

pub trait CRDTExt: CRDT {
    fn merge_into(self, other: &mut Self)
    where
        Self: Sized,
    {
        other.merge(self.take())
    }
}

impl<C> CRDTExt for C where C: CRDT {}
