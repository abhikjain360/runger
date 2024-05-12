use std::{num::NonZeroUsize, path::Path};

use mlua::{Lua, Table};

use crate::Result;

#[derive(Debug, Clone)]
pub struct Config {
    pub required_columns: NonZeroUsize,
    pub column_margin: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid field: required_columns must be greater than 0")]
    ZeroRequiredColumns,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // SAFETY: it is not zero
            required_columns: unsafe { NonZeroUsize::new_unchecked(3) },
            column_margin: 0,
        }
    }
}

impl Config {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let mut config = Self::default();

        if !path.as_ref().exists() {
            return Ok(config);
        }

        let lua = Lua::new();
        let table: Table = lua.load(path.as_ref()).eval()?;

        match table.get::<_, Option<usize>>("required_columns")? {
            Some(0) => return Err(Error::ZeroRequiredColumns.into()),
            Some(val) => {
                config.required_columns =
                    NonZeroUsize::new(val).ok_or(Error::ZeroRequiredColumns)?;
            }
            _ => {}
        };

        if let Some(val) = table.get::<_, Option<usize>>("column_margin")? {
            config.column_margin = val;
        };

        Ok(config)
    }
}
