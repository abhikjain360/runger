use std::env;

use runger::{state::State, Result};

fn main() -> Result<()> {
    let mut state = State::new(env::args().nth(1).unwrap().into(), 3.try_into().unwrap())?;
    println!("{state:?}");
    state.move_right()?;
    println!("{state:?}");

    Ok(())
}
