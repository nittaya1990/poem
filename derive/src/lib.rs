//! Macros for poem

#![forbid(unsafe_code)]
#![deny(private_in_public, unreachable_pub)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]

mod handler;
mod utils;

use proc_macro::TokenStream;
use syn::{parse_macro_input, AttributeArgs};

/// Wrap an asynchronous function as an `Endpoint`.
///
/// # Example
///
/// ```ignore
/// #[handler]
/// async fn example() {
/// }
/// ```
#[proc_macro_attribute]
pub fn handler(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    match handler::generate(args, input) {
        Ok(stream) => stream,
        Err(err) => err.into_compile_error().into(),
    }
}
