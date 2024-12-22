use std::{num::NonZeroUsize, path::Path};

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Config {
    /// The number of columns that are required to be visible.
    pub required_columns: NonZeroUsize,
    pub column_margin: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid field: required_columns must be greater than 1")]
    InvalidRequiredColumns,
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
    pub fn new(path: impl AsRef<Path>) -> crate::Result<Self> {
        if !path.as_ref().exists() {
            return Ok(Self::default());
        }

        let lua = mlua::Lua::new();
        let table: mlua::Table = lua.load(path.as_ref()).eval()?;

        Self::try_from(table)
    }
}

impl TryFrom<mlua::Table> for Config {
    type Error = crate::Error;

    fn try_from(table: mlua::Table) -> Result<Self, Self::Error> {
        let mut config = Self::default();

        match table.get::<Option<usize>>("required_columns")? {
            Some(..2) => return Err(Error::InvalidRequiredColumns.into()),
            Some(val) => {
                config.required_columns =
                    NonZeroUsize::new(val).ok_or(Error::InvalidRequiredColumns)?;
            }
            _ => {}
        };

        if let Some(val) = table.get::<Option<usize>>("column_margin")? {
            config.column_margin = val;
        };

        Ok(config)
    }
}
