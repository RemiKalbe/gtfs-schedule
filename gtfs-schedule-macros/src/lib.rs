use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(StringWrapper)]
pub fn string_wrapper_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Ensure the struct has exactly one field
    match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => &fields.unnamed[0],
            _ => {
                panic!("StringWrapper can only be derived for tuple structs with exactly one field")
            }
        },
        _ => panic!("StringWrapper can only be derived for tuple structs"),
    };

    let expanded = quote! {
        impl #name {
            /// Returns a reference to self.
            /// This method is useful when you need to explicitly work with the wrapper type.
            pub fn as_wrapper(&self) -> &#name {
                self
            }
        }

        impl std::ops::Deref for #name {
            type Target = String;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for #name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl AsRef<str> for #name {
            fn as_ref(&self) -> &str {
                self.0.as_ref()
            }
        }

        impl From<String> for #name {
            fn from(s: String) -> Self {
                #name(s)
            }
        }

        impl From<&str> for #name {
            fn from(s: &str) -> Self {
                #name(s.to_string())
            }
        }

        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl PartialEq for #name {
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }

        impl Eq for #name {}

        // Implement Hash to allow use in HashSet and HashMap
        impl std::hash::Hash for #name {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.0.hash(state);
            }
        }

        impl Clone for #name {
            fn clone(&self) -> Self {
                #name(self.0.clone())
            }
        }
    };

    TokenStream::from(expanded)
}
