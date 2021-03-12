use crate::sanitizer::SanitizerError;
use proc_macro2::Span;
use std::convert::TryFrom;
use syn::Ident;
use syn::{GenericArgument, PathArguments, Type, TypePath};

static INT_TYPES: [&str; 10] = [
    "u8", "u16", "u32", "u64", "i8", "i16", "i32", "i63", "isize", "usize",
];

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum TypeOrNested {
    // field, type
    Type(Ident, TypeIdent),
    Nested(Ident, Ident),
}

impl TypeOrNested {
    pub fn set_type(&mut self, new_type: TypeIdent) {
        if let Self::Type(_, old_type) = self {
            *old_type = new_type
        }
    }
}

fn is_option(typepath: TypePath) -> Result<(bool, Ident), SanitizerError> {
    if let Some(path) = typepath.path.segments.last() {
        if path.ident == "Option" {
            match &path.arguments {
                PathArguments::AngleBracketed(params) => {
                    let type_wrapped = &params.args;
                    match &type_wrapped[0] {
                        GenericArgument::Type(ty) => match ty {
                            Type::Path(inner_type_path) => {
                                if let Some(inner_type) = inner_type_path.path.segments.last() {
                                    return Ok((true, inner_type.clone().ident));
                                }
                            }
                            _ => panic!("Invalid wrapper type for Option<T>"),
                        },
                        _ => return Err(SanitizerError::OnlyOptionTSupported),
                    }
                }
                _ => panic!("No wrapper in type"),
            };
        }
    }
    Ok((false, Ident::new("_", Span::call_site())))
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct TypeIdent {
    pub ident: Ident,
    pub is_int: bool,
    pub is_option: bool,
}

impl TypeIdent {
    pub fn new(ident: Ident, is_int: bool, is_option: bool) -> Self {
        Self {
            ident,
            is_int,
            is_option,
        }
    }
    pub fn is_string(&self) -> bool {
        self.ident == "String"
    }
    pub fn is_int(&self) -> bool {
        self.is_int
    }
    pub fn is_string_or_int(&self) -> bool {
        self.is_int || self.ident == "String"
    }

    pub fn ident(&self) -> Ident {
        self.ident.clone()
    }
}

impl Default for TypeIdent {
    fn default() -> Self {
        TypeIdent::new(Ident::new("_", Span::call_site()), false, false)
    }
}

impl TryFrom<Type> for TypeIdent {
    type Error = SanitizerError;
    fn try_from(type_ident: Type) -> Result<Self, Self::Error> {
        match type_ident {
            Type::Path(type_path) => {
                if let Some(last_segment) = type_path.path.segments.last() {
                    let ident = last_segment.clone().ident;
                    let is_option = is_option(type_path)?;
                    let option_wrapper = is_option.1;
                    if is_option.0 {
                        Ok(TypeIdent::new(
                            option_wrapper.clone(),
                            INT_TYPES.contains(&option_wrapper.clone().to_string().as_str()),
                            true,
                        ))
                    } else {
                        Ok(TypeIdent::new(
                            ident.clone(),
                            INT_TYPES.contains(&ident.to_string().as_str()),
                            false,
                        ))
                    }
                } else {
                    Err(SanitizerError::InvalidFieldType)
                }
            }
            _ => Err(SanitizerError::InvalidFieldType),
        }
    }
}
