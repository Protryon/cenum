extern crate proc_macro;
use crate::proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::*;

#[proc_macro_attribute]
pub fn cenum(_metadata: TokenStream, input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    impl_cenum(&ast)
}

fn impl_cenum(ast: &DeriveInput) -> TokenStream {
    let mut repr = "u32".to_string();
    for attr in &ast.attrs {
        if attr.path.segments.len() == 1 {
            let segment = &attr.path.segments[0];
            if &*segment.ident.to_string() == "repr" {
                let inner_repr = attr.tokens.to_string();
                if inner_repr.starts_with("(") && inner_repr.ends_with(")") {
                    let inner_repr = &inner_repr[1..inner_repr.len() - 1];
                    match inner_repr {
                        "u8" | "u16" | "u32" | "u64" | "u128" | "i8" | "i16" | "i32" | "i64"
                        | "i128" => repr = inner_repr.to_string(),
                        _ => {
                            return quote_spanned! {
                                segment.ident.span() =>
                                compile_error!("invalid repr for cenum (expected u* or i*)");
                            }
                            .into();
                        }
                    }
                }
            }
        }
    }

    let name = &ast.ident;
    let variants = match &ast.data {
        Data::Enum(DataEnum { variants, .. }) => variants.into_iter().collect::<Vec<&Variant>>(),
        _ => panic!("not deriving cenum on an enum"),
    };
    if variants
        .iter()
        .any(|variant| variant.fields != Fields::Unit)
    {
        panic!("cannot have cenum trait on enums with fields")
    }
    let mut pairs: Vec<(String, i128)> = vec![];
    //todo: use better discriminant parsing to avoid limiting us to bounds of i128
    let mut current_discriminant = 0;
    let mut first_variant = true;
    for variant in &variants {
        let is_first_variant = first_variant;
        first_variant = false;
        let discriminant = match &variant.discriminant {
            Some((
                _,
                Expr::Lit(ExprLit {
                    lit: Lit::Int(lit_int),
                    ..
                }),
            )) => {
                let discriminant = lit_int.base10_parse::<i128>().unwrap();
                if !is_first_variant && discriminant < current_discriminant {
                    return quote_spanned! {
                        variant.discriminant.as_ref().unwrap().1.span() =>
                        compile_error!("attempted to reuse discriminant");
                    }
                    .into();
                }
                current_discriminant = discriminant + 1;
                discriminant
            }
            Some((
                _,
                Expr::Unary(ExprUnary {
                    op: UnOp::Neg(_),
                    expr,
                    ..
                }),
            )) => match &**expr {
                Expr::Lit(ExprLit {
                    lit: Lit::Int(lit_int),
                    ..
                }) => {
                    let discriminant = -lit_int.base10_parse::<i128>().unwrap();
                    if !is_first_variant && discriminant < current_discriminant {
                        return quote_spanned! {
                            variant.discriminant.as_ref().unwrap().1.span() =>
                            compile_error!("attempted to reuse discriminant");
                        }
                        .into();
                    }
                    current_discriminant = discriminant + 1;
                    discriminant
                }
                _ => {
                    return quote_spanned! {
                        variant.discriminant.as_ref().unwrap().1.span() =>
                        compile_error!("expected integer literal as discriminant");
                    }
                    .into();
                }
            },
            Some(_) => {
                return quote_spanned! {
                    variant.discriminant.as_ref().unwrap().1.span() =>
                    compile_error!("expected integer literal as discriminant");
                }
                .into();
            }
            None => {
                if is_first_variant {
                    current_discriminant = 0;
                }
                let discriminant = current_discriminant;
                current_discriminant += 1;
                discriminant
            }
        };
        pairs.push((variant.ident.to_string(), discriminant));
    }

    let pairs_formatted = format!(
        "match value {{ {}\n_ => None, }}",
        pairs
            .iter()
            .map(|(key, value)| format!("{} => Some({}::{}),", value, name.to_string(), key))
            .collect::<Vec<String>>()
            .join("\n")
    );
    let pairs_parsed: ExprMatch = parse_str(&pairs_formatted).unwrap();

    let repr_ident = Ident::new(&*repr, name.span());

    let gen = quote! {

        #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
        #ast

        impl #name {
            fn into_primitive(self) -> #repr_ident {
                self as #repr_ident
            }

            fn from_primitive(value: #repr_ident) -> Option<Self> {
                #pairs_parsed
            }
        }

    };
    gen.into()
}
