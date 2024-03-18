use std::{num::NonZeroUsize, path::Path};

use mlua::{Lua, Table};

use crate::Result;

#[derive(Debug, Clone)]
pub struct Config {
    pub required_columns: NonZeroUsize,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid field: required_columns must be greater than 0")]
    ZeroRequiredColumns,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            required_columns: unsafe { NonZeroUsize::new_unchecked(3) },
        }
    }
}

impl Config {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let default = Self::default();

        if !path.as_ref().exists() {
            return Ok(default);
        }

        let lua = Lua::new();
        let config: Table = lua.load(path.as_ref()).eval()?;

        let required_columns = match config.get::<_, Option<usize>>("required_columns")? {
            Some(0) => return Err(Error::ZeroRequiredColumns.into()),
            None => default.required_columns,
            Some(val) => NonZeroUsize::new(val).ok_or_else(|| Error::ZeroRequiredColumns)?,
        };

        Ok(Self { required_columns })
    }
}
