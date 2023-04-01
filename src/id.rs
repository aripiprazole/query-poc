use std::any::TypeId;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Copy)]
pub struct Id<A>(PhantomData<A>);

pub fn id<A>() -> Id<A> {
    Id::REFL
}

impl<A> Id<A> {
    const REFL: Id<A> = Id(PhantomData);
}

impl<A: 'static> Hash for Id<A> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        TypeId::of::<A>().hash(state);
    }
}
