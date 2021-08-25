use darling::{util::PathList, FromMeta};
use inflector::Inflector;
use proc_macro2::{Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::{Attribute, Ident, Lit, Meta, Result};

pub(crate) fn get_crate_name(internal: bool) -> TokenStream {
    if internal {
        quote! { crate }
    } else {
        let name = match crate_name("poem") {
            Ok(FoundCrate::Name(name)) => name,
            Ok(FoundCrate::Itself) | Err(_) => "poem".to_string(),
        };
        let name = Ident::new(&name, Span::call_site());
        quote!(#name)
    }
}

#[derive(Debug, Copy, Clone, FromMeta)]
pub(crate) enum RenameRule {
    #[darling(rename = "lowercase")]
    Lower,
    #[darling(rename = "UPPERCASE")]
    Upper,
    #[darling(rename = "PascalCase")]
    Pascal,
    #[darling(rename = "camelCase")]
    Camel,
    #[darling(rename = "snake_case")]
    Snake,
    #[darling(rename = "SCREAMING_SNAKE_CASE")]
    ScreamingSnake,
}

impl RenameRule {
    pub(crate) fn rename(&self, name: impl AsRef<str>) -> String {
        match self {
            Self::Lower => name.as_ref().to_lowercase(),
            Self::Upper => name.as_ref().to_uppercase(),
            Self::Pascal => name.as_ref().to_pascal_case(),
            Self::Camel => name.as_ref().to_camel_case(),
            Self::Snake => name.as_ref().to_snake_case(),
            Self::ScreamingSnake => name.as_ref().to_screaming_snake_case(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum RenameTarget {
    Type,
    EnumItem,
    Field,
    Argument,
}

impl RenameTarget {
    fn rule(&self) -> RenameRule {
        match self {
            RenameTarget::Type => RenameRule::Pascal,
            RenameTarget::EnumItem => RenameRule::ScreamingSnake,
            RenameTarget::Field => RenameRule::Camel,
            RenameTarget::Argument => RenameRule::Camel,
        }
    }

    pub(crate) fn rename(&self, name: impl AsRef<str>) -> String {
        self.rule().rename(name)
    }
}

pub(crate) trait RenameRuleExt {
    fn rename(&self, name: impl AsRef<str>, target: RenameTarget) -> String;
}

impl RenameRuleExt for Option<RenameRule> {
    fn rename(&self, name: impl AsRef<str>, target: RenameTarget) -> String {
        self.unwrap_or(target.rule()).rename(name)
    }
}

#[derive(Debug)]
pub(crate) enum DefaultValue {
    Default,
    Value(Lit),
}

impl FromMeta for DefaultValue {
    fn from_word() -> darling::Result<Self> {
        Ok(DefaultValue::Default)
    }

    fn from_value(value: &Lit) -> darling::Result<Self> {
        Ok(DefaultValue::Value(value.clone()))
    }
}

#[derive(FromMeta)]
pub(crate) struct ConcreteType {
    pub(crate) name: String,
    pub(crate) params: PathList,
}

pub(crate) fn get_description(attrs: &[Attribute]) -> Result<Option<String>> {
    let mut full_docs = String::new();
    for attr in attrs {
        match attr.parse_meta()? {
            Meta::NameValue(nv) if nv.path.is_ident("doc") => {
                if let Lit::Str(doc) = nv.lit {
                    let doc = doc.value();
                    let doc_str = doc.trim();
                    if !full_docs.is_empty() {
                        full_docs += "\n";
                    }
                    full_docs += doc_str;
                }
            }
            _ => {}
        }
    }
    Ok(if full_docs.is_empty() {
        None
    } else {
        Some(full_docs)
    })
}

pub(crate) fn get_summary_and_description(
    attrs: &[Attribute],
) -> Result<(Option<String>, Option<String>)> {
    let mut summary = String::new();
    let mut full_docs = String::new();
    for attr in attrs {
        match attr.parse_meta()? {
            Meta::NameValue(nv) if nv.path.is_ident("doc") => {
                if let Lit::Str(doc) = nv.lit {
                    let doc = doc.value();
                    let doc_str = doc.trim();
                    if !doc_str.is_empty() {
                        if summary.is_empty() {
                            summary = doc_str.to_string();
                        } else {
                            if !full_docs.is_empty() {
                                full_docs += "\n";
                            }
                            full_docs += doc_str;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok((
        if summary.is_empty() {
            None
        } else {
            Some(summary)
        },
        if full_docs.is_empty() {
            None
        } else {
            Some(full_docs)
        },
    ))
}
