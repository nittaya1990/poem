use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;
use syn::{AttributeArgs, FnArg, ItemFn, Result};

use super::utils::get_crate_name;

#[derive(FromMeta, Default)]
#[darling(default)]
struct HandlerArgs {
    internal: bool,
}

pub(crate) fn generate(args: AttributeArgs, input: TokenStream) -> Result<TokenStream> {
    let args = match HandlerArgs::from_list(&args) {
        Ok(args) => args,
        Err(err) => return Ok(err.write_errors().into()),
    };

    let crate_name = get_crate_name(args.internal);
    let item_fn = syn::parse::<ItemFn>(input)?;
    let vis = &item_fn.vis;
    let ident = &item_fn.sig.ident;
    let call_await = if item_fn.sig.asyncness.is_some() {
        Some(quote::quote!(.await))
    } else {
        None
    };

    let mut extractors = Vec::new();
    let mut args = Vec::new();
    for (idx, input) in item_fn.sig.inputs.clone().into_iter().enumerate() {
        if let FnArg::Typed(pat) = input {
            let ty = &pat.ty;
            let id = quote::format_ident!("p{}", idx);
            args.push(id.clone());
            extractors.push(quote! {
                let #id = match <#ty as #crate_name::FromRequest>::from_request(&req, &mut body).await {
                    Ok(value) => value,
                    Err(err) => return err.as_response(),
                };
            });
        }
    }

    let expanded = quote! {
        #[allow(non_camel_case_types)]
        #vis struct #ident;

        #[#crate_name::async_trait]
        impl #crate_name::Endpoint for #ident {
            type Output = #crate_name::Response;

            async fn call(&self, mut req: #crate_name::Request) -> Self::Output {
                let (req, mut body) = req.split();
                #(#extractors)*
                #item_fn
                #crate_name::IntoResponse::into_response(#ident(#(#args),*)#call_await)
            }
        }
    };

    Ok(expanded.into())
}
