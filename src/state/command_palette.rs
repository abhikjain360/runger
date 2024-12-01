#[cfg_attr(debug_assertions, derive(Debug))]
pub enum CommandPalette {
    Empty,
    Error {
        error: crate::Error,
        show_until: std::time::Instant,
    },
    Typing {
        input: String,
    },
}
