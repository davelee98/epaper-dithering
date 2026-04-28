use std::fmt;

/// Errors that can occur during dithering operations.
#[derive(Debug, PartialEq)]
pub enum DitherError {
    UnknownColorScheme(u8),
    UnknownDitherMode(u8),
}

impl fmt::Display for DitherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DitherError::UnknownColorScheme(v) => write!(f, "unknown color scheme: {v}"),
            DitherError::UnknownDitherMode(v) => write!(f, "unknown dither mode: {v}"),
        }
    }
}

impl std::error::Error for DitherError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_unknown_scheme() {
        let e = DitherError::UnknownColorScheme(42);
        let msg = e.to_string();
        assert!(msg.contains("42"), "message should include the bad value: {msg}");
    }
}
