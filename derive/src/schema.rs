use darling::{ast::Data, util::Ignored, FromDeriveInput, FromField};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{
    ext::IdentExt, Attribute, AttributeArgs, DeriveInput, Error, Generics, Result, Type, Visibility,
};

use crate::utils::{
    get_crate_name, get_description, get_summary_and_description, ConcreteType, DefaultValue,
    RenameRule, RenameRuleExt, RenameTarget,
};

#[derive(FromField)]
#[darling(attributes(oai), forward_attrs(doc))]
struct SchemaField {
    ident: Option<Ident>,
    ty: Type,
    vis: Visibility,
    attrs: Vec<Attribute>,

    #[darling(default)]
    name: Option<String>,
    #[darling(default)]
    default: Option<DefaultValue>,
    #[darling(default)]
    skip: bool,
}

#[derive(FromDeriveInput)]
#[darling(attributes(oai), forward_attrs(doc))]
struct SchemaArgs {
    ident: Ident,
    generics: Generics,
    attrs: Vec<Attribute>,
    data: Data<Ignored, SchemaField>,

    #[darling(default)]
    internal: bool,
    #[darling(default)]
    name: Option<String>,
    #[darling(default)]
    rename_fields: Option<RenameRule>,
    #[darling(default, multiple, rename = "concrete")]
    concretes: Vec<ConcreteType>,
}

pub(crate) fn generate(args: DeriveInput, input: TokenStream) -> Result<TokenStream> {
    let args: SchemaArgs = match SchemaArgs::from_derive_input(&args) {
        Ok(args) => args,
        Err(err) => return Ok(err.write_errors().into()),
    };

    let crate_name = get_crate_name(args.internal);
    let openapi_mod = quote!(#crate_name::service::openapi);
    let (impl_generics, ty_generics, where_clause) = args.generics.split_for_impl();
    let ident = &args.ident;
    let s = match &args.data {
        Data::Struct(s) => s,
        _ => {
            return Err(
                Error::new_spanned(ident, "Schema can only be applied to an struct.").into(),
            )
        }
    };
    let oai_typename = args
        .name
        .clone()
        .unwrap_or_else(|| RenameTarget::Type.rename(ident.to_string()));
    let (summary, description) = get_summary_and_description(&args.attrs)?;
    let mut get_fields = Vec::new();
    let mut fields = Vec::new();

    for field in &s.fields {
        let field_ident = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;

        if field.skip {
            get_fields.push(quote! {
                let #field_ident: #field_ty = ::std::default::Default::default();
            });
            fields.push(ident);
            continue;
        }

        let field_name = field.name.clone().unwrap_or_else(|| {
            args.rename_fields
                .rename(ident.unraw().to_string(), RenameTarget::Field)
        });
        let description = get_description(&field.attrs)?;

        get_fields.push(quote! {
            #[allow(non_snake_case)]
            let #field_ident = #field_ty = #openapi_mod::Type::parse(obj.get(#field_name).cloned())
                .map_err(#openapi_mod::ParseError::propagate)?;
        });
    }

    let expanded = if args.concretes.is_empty() {
        quote! {
            impl #openapi_mod::Type for #ident {
                fn type_name() -> &'static str {
                    #oai_typename
                }

                fn parse(value: #openapi_mod::serde_json::Value) -> ::std::result::Result<Self, #openapi_mod::ParseError<Self>> {
                    #(#get_fields)*
                    ::std::result::Result::Ok(Self { #(#fields),* })
                }

                fn to_value(&self) -> #openapi_mod::serde_json::Value {

                }
            }
        }
    } else {
    };

    Ok(expanded.into())
}
