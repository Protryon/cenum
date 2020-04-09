extern crate proc_macro;
use crate::proc_macro::TokenStream;
use quote::quote;
use syn::*;

#[proc_macro_attribute]
pub fn cenum(_metadata: TokenStream, input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    impl_cenum(&ast)
}

fn impl_cenum(ast: &DeriveInput) -> TokenStream {
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
    let mut pairs: Vec<(String, i64)> = vec![];
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
                let discriminant = lit_int.base10_parse::<i64>().unwrap();
                if !is_first_variant && discriminant < current_discriminant {
                    panic!("attempted to reuse discriminant");
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
                })
            )) => {
                match &**expr {
                    Expr::Lit(ExprLit {
                        lit: Lit::Int(lit_int),
                        ..
                    }) => {
                        let discriminant = -lit_int.base10_parse::<i64>().unwrap();
                        if !is_first_variant && discriminant < current_discriminant {
                            panic!("attempted to reuse discriminant");
                        }
                        current_discriminant = discriminant + 1;
                        discriminant
                    },
                    _ => panic!("expected integer literal as discriminant")
                }
            },
            Some(_) => panic!("expected integer literal as discriminant"),
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
        "[{}]",
        pairs
            .iter()
            .map({ |(key, value)| format!("({}::{}, {})", name.to_string(), key, value) })
            .collect::<Vec<String>>()
            .join(", ")
    );
    let pairs_parsed: ExprArray = parse_str(&pairs_formatted).unwrap();

    let data_name = Ident::new(
        &format!("__{}_data", name.to_string()).to_uppercase(),
        name.span(),
    );
    let cache_name = Ident::new(
        &format!("__{}_cache", name.to_string()).to_uppercase(),
        name.span(),
    );
    let icache_name = Ident::new(
        &format!("__{}_icache", name.to_string()).to_uppercase(),
        name.span(),
    );
    let get_cache_name = Ident::new(&format!("__{}_get_cache", name.to_string()), name.span());
    let get_icache_name = Ident::new(&format!("__{}_get_icache", name.to_string()), name.span());

    let gen = quote! {

        #[derive(PartialEq, Eq, Hash, Clone, Debug)]
        #ast

        static #data_name: &'static [(#name, i64)] = &#pairs_parsed;
        static #cache_name: ::std::sync::atomic::AtomicPtr<::std::collections::HashMap<#name, i64>> = ::std::sync::atomic::AtomicPtr::new(::std::ptr::null_mut());
        static #icache_name: ::std::sync::atomic::AtomicPtr<::std::collections::HashMap<i64, #name>> = ::std::sync::atomic::AtomicPtr::new(::std::ptr::null_mut());

        #[allow(non_snake_case)]
        fn #get_cache_name() -> &'static ::std::collections::HashMap<#name, i64> {
            unsafe {
                if #cache_name.load(::std::sync::atomic::Ordering::Relaxed).is_null() {
                    let mut map_built = Box::new(::std::collections::HashMap::new());
                    for (key, value) in #data_name {
                        map_built.insert(key.clone(), *value);
                    }
                    let map_built = Box::into_raw(map_built);
                    if !#cache_name.compare_and_swap(::std::ptr::null_mut(), map_built, ::std::sync::atomic::Ordering::Relaxed).is_null() {
                        drop(Box::from_raw(map_built));
                    }
                }
                return #cache_name.load(::std::sync::atomic::Ordering::Relaxed).as_ref().unwrap();
            }
        }

        #[allow(non_snake_case)]
        fn #get_icache_name() -> &'static ::std::collections::HashMap<i64, #name> {
            unsafe {
                if #icache_name.load(::std::sync::atomic::Ordering::Relaxed).is_null() {
                    let mut map_built = Box::new(::std::collections::HashMap::new());
                    for (key, value) in #data_name {
                        map_built.insert(*value, key.clone());
                    }
                    let map_built = Box::into_raw(map_built);
                    if !#icache_name.compare_and_swap(::std::ptr::null_mut(), map_built, ::std::sync::atomic::Ordering::Relaxed).is_null() {
                        drop(Box::from_raw(map_built));
                    }
                }
                return #icache_name.load(::std::sync::atomic::Ordering::Relaxed).as_ref().unwrap();
            }
        }

        impl Cenum for #name {
            fn to_primitive(&self) -> i64 {
                return *#get_cache_name().get(self).unwrap();
            }

            fn from_primitive(value: i64) -> #name {
                return #get_icache_name().get(&value).unwrap().clone();
            }

            fn is_discriminant(value: i64) -> bool {
                return #get_icache_name().get(&value).is_some();
            }
        }

        impl ::cenum::num::ToPrimitive for #name {
            fn to_i64(&self) -> Option<i64> {
                Some(self.to_primitive() as i64)
            }

            fn to_u64(&self) -> Option<u64> {
                Some(self.to_primitive() as u64)
            }
        }


    };
    gen.into()
}
