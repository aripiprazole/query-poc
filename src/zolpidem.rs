use crate::eszopiclone::Fail;
use paste::paste;
use std::path::PathBuf;

macro_rules! query {
    ($name:ident: ($($args:ty),+ $(,)?) -> $rt:ty => $block:expr) => {
        #[derive(Hash, Clone, Debug)]
        pub struct $name($($args),+, crate::id::Id<$name>);

        impl crate::eszopiclone::Query for $name {
            type Output = $rt;

            #[allow(unused_variables)]
            fn execute(&self, compiler: &mut crate::eszopiclone::Compiler) -> crate::eszopiclone::Result<Self::Output> {
                let f: &dyn Fn(&mut crate::eszopiclone::Compiler, Self) -> crate::eszopiclone::Result<Self::Output> = &$block;
                f(compiler, self.clone())
            }
        }

        paste! {
            impl $name {
                #[allow(dead_code)]
                pub fn new($([<_ ${index(0)}>]: $args),+) -> Self {
                    $name($([<_ ${index(0)}>] as $args),+, crate::id::id())
                }
            }
        }
    };
    ($name:ident: ($($args:ty),+ $(,)?) -> $rt:ty :> $block:expr) => {
        query!($name: ($($args),+) -> $rt => |_, q| $block(q.clone()));
    }
}

query!(Source: (PathBuf) -> String :> |Source(ref path, ..)| {
    let contents = std::fs::read_to_string(path)
    .map_err(|_| Fail::UnboundModule(path.clone()))?;

    Ok(contents)
});

query!(Module: (PathBuf) -> String => |compiler, Module(ref path, ..)| {
    let contents = compiler.query(Source::new(path.clone()))?;

    Ok(contents.to_string())
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eszopiclone::Compiler;

    #[test]
    fn it_works() {
        let mut compiler = Compiler::default();

        println!("--> Fetch resource");
        dbg!(compiler
            .query(Source::new(PathBuf::from("src/main.rs")))
            .unwrap());

        println!("--> Use cache (1)");
        dbg!(compiler
            .query(Source::new(PathBuf::from("src/main.rs")))
            .unwrap());

        println!("--> Fetch resource");
        dbg!(compiler
            .query(Module::new(PathBuf::from("src/main.rs")))
            .unwrap());
    }
}
