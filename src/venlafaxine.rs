use std::fmt::Debug;
use std::hash::Hash;
use std::mem::{transmute, transmute_copy};
use std::os::raw::c_void;
use std::path::PathBuf;

use intmap::{Entry, IntMap};

use crate::refl::{refl, Is};

pub type Result<T, E = String> = std::result::Result<T, E>;

#[derive(Debug, Hash, Clone)]
pub struct ConcreteModule;

#[derive(Debug, Hash, Clone)]
pub struct AbstractModule;

#[derive(Debug, Hash, Clone)]
pub enum Query<A: Sized = *mut c_void> {
    Source(Is<A, String>, PathBuf),
    Dependencies(Is<A, Vec<PathBuf>>, PathBuf),
    Module(Is<A, ConcreteModule>, PathBuf),
    AbstractModule(Is<A, AbstractModule>, PathBuf),
}

#[derive(Debug, Hash, Clone)]
pub enum Resource<A: Sized = *mut c_void> {
    Source(Is<A, String>, String),
    Dependencies(Is<A, Vec<PathBuf>>, Vec<PathBuf>),
    Module(Is<A, ConcreteModule>, ConcreteModule),
    AbstractModule(Is<A, AbstractModule>, AbstractModule),
    Nil(Is<A, ()>),
}

#[derive(Default)]
pub struct Compiler {
    pub queries: IntMap<*mut Resource>,
}

impl Compiler {
    pub fn execute<A>(&mut self, resource: *mut Resource<A>, query: Query<A>) -> Result<&A>
    where
        A: Sized + Hash + Clone + Debug,
    {
        match query {
            Query::Source(refl, path) => {
                println!("[Query] Fetching dependency");
                let contents = std::fs::read_to_string(path).map_err(|_| "Couldn't find module")?;

                Ok(&Resource::Source(refl, contents).update(resource))
            }
            Query::Dependencies(_, _) => todo!(),
            Query::Module(_, _) => todo!(),
            Query::AbstractModule(_, _) => todo!(),
        }
    }

    pub fn query<A: Sized + Hash + Clone + Debug>(&mut self, query: Query<A>) -> Result<&A> {
        let resource = self.get_resource(&query);
        let value = unsafe { resource.as_ref().unwrap() };

        if matches!(value, Resource::Nil(..)) {
            println!("[Cache] can't find a reference for query: {:?}", query);
            return self.execute(resource, query);
        }

        println!("[Cache] using working cache for {:?}", query);
        return Ok(value.extract());
    }

    pub fn get_resource<A: Sized + Hash>(&mut self, query: &Query<A>) -> *mut Resource<A> {
        let hash = fxhash::hash64(&query);

        unsafe {
            match self.queries.entry(hash) {
                Entry::Occupied(entry) => transmute_copy(entry.get()),
                Entry::Vacant(entry) => {
                    let new_ptr: *mut Resource<()> = Box::leak(Box::new(Resource::Nil(refl())));
                    entry.insert(transmute(new_ptr.clone()));
                    transmute(new_ptr.clone())
                }
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
            Resource::Nil(..) => panic!("It's not supposed to trigger this panic"),
        }
    }

    pub fn unwrap<'a>(ptr: *mut Resource<A>) -> &'a A {
        unsafe { ptr.as_ref().unwrap().extract() }
    }

    fn update<'a>(self, ptr: *mut Resource<A>) -> &'a A {
        unsafe {
            ptr.write(self)
        };

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

        println!("--> Fetch resource");
        dbg!(compiler
            .query(Query::Source(refl(), PathBuf::from("src/main.rs")))
            .unwrap());

        println!("--> Use cache (1)");
        dbg!(compiler
            .query(Query::Source(refl(), PathBuf::from("src/main.rs")))
            .unwrap());

        println!("--> Use cache (2)");
        dbg!(compiler
            .query(Query::Source(refl(), PathBuf::from("src/main.rs")))
            .unwrap());
    }
}
