use std::hash::Hash;
use std::path::PathBuf;

use intmap::{Entry, IntMap};

use crate::refl::Is;

pub type Result<T, E = String> = std::result::Result<T, E>;

#[derive(Hash, Clone)]
pub struct ConcreteModule;

#[derive(Hash, Clone)]
pub struct AbstractModule;

#[derive(Hash, Clone)]
pub enum Query<A: Sized = ()> {
    Source(Is<A, String>, PathBuf),
    Dependencies(Is<A, Vec<PathBuf>>, PathBuf),
    Module(Is<A, ConcreteModule>, PathBuf),
    AbstractModule(Is<A, AbstractModule>, PathBuf),
}

pub enum Resource<A: Sized = ()> {
    Source(Is<A, String>, String),
    Dependencies(Is<A, Vec<PathBuf>>, Vec<PathBuf>),
    Module(Is<A, ConcreteModule>, ConcreteModule),
    AbstractModule(Is<A, AbstractModule>, AbstractModule),
}

#[derive(Default)]
pub struct Compiler {
    pub queries: IntMap<*mut Resource>,
}

impl Compiler {
    pub fn execute<A>(&mut self, resource: *mut Resource<A>, query: Query<A>) -> Result<&A>
    where
        A: Sized + Hash + Clone,
    {
        match query {
            Query::Source(refl, path) => {
                let contents = std::fs::read_to_string(path).map_err(|_| "Couldn't find module")?;

                Ok(&Resource::Source(refl, contents).update(resource))
            }
            Query::Dependencies(_, _) => todo!(),
            Query::Module(_, _) => todo!(),
            Query::AbstractModule(_, _) => todo!(),
        }
    }

    pub fn query<A: Sized + Hash + Clone>(&mut self, query: Query<A>) -> Result<&A> {
        let resource = self.get_resource(&query);
        if !resource.is_null() {
            return Ok(Resource::unwrap(resource));
        }

        self.execute(resource, query)
    }

    pub fn get_resource<A: Sized + Hash>(&mut self, query: &Query<A>) -> *mut Resource<A> {
        use std::mem::transmute;
        use std::ptr::null_mut;

        let hash = fxhash::hash64(&query);

        unsafe {
            match self.queries.entry(hash) {
                Entry::Occupied(entry) => transmute(entry.get()),
                Entry::Vacant(entry) => transmute(entry.insert(null_mut())),
            }
        }
    }
}

impl<A: Sized> Resource<A> {
    pub fn extract(&self) -> &A {
        match self {
            Resource::Source(refl, value) => refl.cast_ref(value),
            Resource::Dependencies(refl, value) => refl.cast_ref(value),
            Resource::Module(refl, value) => refl.cast_ref(value),
            Resource::AbstractModule(refl, value) => refl.cast_ref(value),
        }
    }

    pub fn unwrap<'a>(ptr: *mut Resource<A>) -> &'a A {
        unsafe { ptr.as_ref().unwrap().extract() }
    }

    fn update<'a>(self, ptr: *mut Resource<A>) -> &'a A {
        unsafe { *ptr = self };

        Self::unwrap(ptr)
    }
}

#[cfg(test)]
mod tests {
    use crate::refl::refl;
    use super::*;

    #[test]
    fn it_works() {
        let mut compiler = Compiler::default();

        dbg!(compiler.query(Query::Source(refl(), PathBuf::from("src/main.rs"))).unwrap());
        dbg!(compiler.query(Query::Source(refl(), PathBuf::from("src/main.rs"))).unwrap());
        dbg!(compiler.query(Query::Source(refl(), PathBuf::from("src/main.rs"))).unwrap());
    }
}
