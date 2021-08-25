use std::{
    fmt::{self, Display, Formatter},
    marker::PhantomData,
};

use serde_json::Value;

use crate::types::ScalarType;

/// An error parsing an scalar.
///
/// This type is generic over T as it uses T's type name when converting to a
/// regular error.
#[derive(Debug)]
pub struct ParseError<T> {
    message: String,
    phantom: PhantomData<T>,
}

impl<T> Display for ParseError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl<T: ScalarType> ParseError<T> {
    fn new(message: String) -> Self {
        Self {
            message,
            phantom: PhantomData,
        }
    }

    /// The expected input type did not match the actual input type.
    #[must_use]
    pub fn expected_type(actual: Value) -> Self {
        Self::new(format!(
            r#"Expected input type "{}", found {}."#,
            T::type_name(),
            actual
        ))
    }

    /// A custom error message.
    ///
    /// Any type that implements `Display` is automatically converted to this if
    /// you use the `?` operator.
    #[must_use]
    pub fn custom(msg: impl Display) -> Self {
        Self::new(format!(r#"failed to parse "{}": {}"#, T::type_name(), msg))
    }

    /// Propagate the error message to a different type.
    pub fn propagate<U: ScalarType>(self) -> ParseError<U> {
        if T::type_name() != U::type_name() {
            ParseError::new(format!(
                r#"{} (occurred while parsing "{}")"#,
                self.message,
                U::type_name()
            ))
        } else {
            ParseError::new(self.message)
        }
    }
}
