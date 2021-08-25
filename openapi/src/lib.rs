#![forbid(unsafe_code)]
#![deny(private_in_public, unreachable_pub)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]

mod error;
mod types;

pub use error::ParseError;
#[doc(hidden)]
pub use serde_json;
pub use types::SchemaType;
