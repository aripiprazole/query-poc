use std::any::Any;
use std::fs::read_to_string;
use std::hash::Hash;
use std::path::PathBuf;
use std::rc::Rc;

use intmap::IntMap;

use crate::eszopiclone::Fail::{InvalidCacheDowncast, UnboundModule};
use crate::id::{id, Id};

pub type Result<T, E = Fail> = std::result::Result<T, E>;

#[derive(Debug, Clone)]
pub struct ConcreteModule(pub PathBuf, pub String);

#[derive(Debug)]
pub enum Fail {
    InvalidCacheDowncast(u64),
    UnboundModule(PathBuf),
}

#[derive(Default)]
pub struct Compiler {
    pub queries: IntMap<Rc<dyn Any>>,
}

#[derive(Debug, Hash, Clone)]
pub struct SourceQuery(pub Id<SourceQuery>, pub PathBuf);

#[derive(Debug, Hash, Clone)]
pub struct ModuleQuery(pub Id<ModuleQuery>, pub PathBuf);

pub trait Query {
    type Output;

    fn execute(&self, compiler: &mut Compiler) -> Result<Self::Output>;
}

impl Query for SourceQuery {
    type Output = String;

    fn execute(&self, _compiler: &mut Compiler) -> Result<Self::Output> {
        let contents = read_to_string(&self.1).map_err(|_| UnboundModule(self.1.clone()))?;

        Ok(contents)
    }
}

impl Query for ModuleQuery {
    type Output = ConcreteModule;

    fn execute(&self, compiler: &mut Compiler) -> Result<Self::Output> {
        let source = compiler.query(SourceQuery(id(), self.1.clone()))?;

        Ok(ConcreteModule(self.1.clone(), source.clone().to_string()))
    }
}

impl Compiler {
    pub fn query<Q: Query + Hash>(&mut self, query: Q) -> Result<Rc<Q::Output>>
    where
        Q::Output: Clone + 'static,
    {
        let hash = fxhash::hash64(&query);
        let resource = self.queries.get(hash);

        match resource {
            Some(resource) => {
                let resource = resource
                    .clone()
                    .downcast::<Q::Output>()
                    .map_err(|_| InvalidCacheDowncast(hash))?;

                Ok(resource)
            }
            None => {
                let output = query.execute(self)?;
                let resource = Rc::new(output);

                self.queries.insert(hash, resource.clone());

                Ok(resource)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut compiler = Compiler::default();

        println!("--> Fetch resource");
        dbg!(compiler
            .query(SourceQuery(id(), PathBuf::from("src/main.rs")))
            .unwrap());

        println!("--> Use cache (1)");
        dbg!(compiler
            .query(SourceQuery(id(), PathBuf::from("src/main.rs")))
            .unwrap());

        println!("--> Fetch resource");
        dbg!(compiler
            .query(ModuleQuery(id(), PathBuf::from("src/main.rs")))
            .unwrap());
    }
}
