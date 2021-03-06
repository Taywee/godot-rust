use proc_macro::TokenStream;
use proc_macro2::Literal;

use super::repr::Repr;
use super::{DeriveData, ToVariantTrait};

pub(crate) fn expand_to_variant(
    trait_kind: ToVariantTrait,
    derive_data: DeriveData,
) -> TokenStream {
    let DeriveData {
        ident,
        repr,
        mut generics,
    } = derive_data;

    let trait_path = trait_kind.trait_path();
    let to_variant_fn = trait_kind.to_variant_fn();
    let to_variant_receiver = trait_kind.to_variant_receiver();

    for param in generics.type_params_mut() {
        param.default = None;
    }

    let return_expr = match repr {
        Repr::Struct(var_repr) => {
            let destructure_pattern = var_repr.destructure_pattern();
            let to_variant = var_repr.to_variant(trait_kind);
            quote! {
                {
                    let #ident #destructure_pattern = self;
                    #to_variant
                }
            }
        }
        Repr::Enum(variants) => {
            if variants.is_empty() {
                quote! {
                    unreachable!("this is an uninhabitable enum");
                }
            } else {
                let match_arms = variants
                    .iter()
                    .map(|(var_ident, var_repr)| {
                        let destructure_pattern = var_repr.destructure_pattern();
                        let to_variant = var_repr.to_variant(trait_kind);
                        let var_ident_string = format!("{}", var_ident);
                        let var_ident_string_literal = Literal::string(&var_ident_string);
                        quote! {
                            #ident::#var_ident #destructure_pattern => {
                                let __dict = ::gdnative::core_types::Dictionary::new();
                                let __key = ::gdnative::core_types::GodotString::from(#var_ident_string_literal).to_variant();
                                let __value = #to_variant;
                                __dict.insert(&__key, &__value);
                                __dict.into_shared().to_variant()
                            }
                        }
                    });

                quote! {
                    match &self {
                        #( #match_arms ),*
                    }
                }
            }
        }
    };

    let where_clause = &generics.where_clause;

    let result = quote! {
        #[allow(unused_variables)]
        impl #generics #trait_path for #ident #generics #where_clause {
            fn #to_variant_fn(#to_variant_receiver) -> ::gdnative::core_types::Variant {
                use #trait_path;
                use ::gdnative::core_types::FromVariant;

                #return_expr
            }
        }
    };

    result.into()
}
