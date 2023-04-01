use std::fs::read_to_string;
use std::path::PathBuf;

use query_poc::Query;

use crate::eszopiclone::{Compiler, Fail, Result};

#[derive(Query)]
#[output(String)]
#[query(execute_source)]
pub struct Source(PathBuf);

pub fn execute_source(_compiler: &mut Compiler, Source(path): &Source) -> Result<String> {
    let contents = read_to_string(path).map_err(|_| Fail::UnboundModule(path.clone()))?;

    Ok(contents)
}
