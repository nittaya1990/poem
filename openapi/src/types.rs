use serde_json::Value;

use super::ParseError;

/// Represents a OpenApi type.
pub trait Type: Sized + Send + Sync {
    fn type_name() -> &'static str;

    fn parse(value: Value) -> Result<Self, ParseError<Self>>;

    fn to_value(&self) -> Value;
}

/// Represents a OpenApi schema.
pub trait SchemaType: Type {}
