use std::marker::PhantomData;

#[derive(Hash, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Copy)]
pub struct Is<A, B>(PhantomData<(A, B)>);

#[inline]
pub fn refl<A: Sized>() -> Is<A, A> {
    Is::REFL
}

impl<A: Sized, B: Sized> Is<A, B> {
    const REFL: Is<A, B> = Is(PhantomData);

    pub fn cast(self, value: B) -> A {
        unsafe {
            let new_value = std::mem::transmute_copy::<B, A>(&value);
            std::mem::forget(value);
            new_value
        }
    }

    pub fn cast_ref<'a, 'b>(&self, value: &'a B) -> &'b A {
        unsafe {
            std::mem::transmute_copy(&value)
        }
    }
}
