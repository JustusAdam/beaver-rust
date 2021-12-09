extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

#[proc_macro_derive(Policied, attributes(policy_protected))]
pub fn policied_derive(input: TokenStream) -> TokenStream {

  let input = syn::parse_macro_input!(input as syn::DeriveInput);

  // get the name of the type we want to implement the trait for
  let name = &input.ident;

  // Find the name of members we need to duplicate
  let mut protected: Vec<(syn::Ident, syn::Ident)> = vec![];

  match input.data {
      // Only process structs
      syn::Data::Struct(ref data_struct) => {
          // Check the kind of fields the struct contains
          match data_struct.fields {
              // Structs with named fields
              syn::Fields::Named(ref fields_named) => {
                  // Iterate over the fields
                  for field in fields_named.named.iter() {
                      // Get attributes #[..] on each field
                      for attr in field.attrs.iter() {
                          // Parse the attribute
                          let meta = attr.parse_meta().unwrap();
                          if meta.name().to_string() == "policy_protected" {
                            match meta {
                              syn::Meta::List(inner_list) => {
                                // Get nested return types #[policy_protected(...)]
                                for ty in inner_list.nested.iter() {
                                  match ty {
                                    syn::NestedMeta::Meta(ty_meta) => {
                                      // Save the protected elements
                                      let attr = field.clone();
                                      protected.push((attr.ident.unwrap(), ty_meta.clone().name()));
                                    }
                                    _ => panic!("Inner list must be type, not string literal"),
                                  }
                                }
                              }
                              _ => panic!("Must have return type in inner list"),
                            }
                          }
                      }
                  }
              }

              // Struct with unnamed fields
              _ => (),
          }
      }

      // Panic when we don't have a struct
      _ => panic!("Must be a struct"),
  }

  let expanded_protected = protected.iter().fold(
    quote!(), |es, (name, ty)| quote! {
      #es
      pub fn #name(&self) -> #ty {
        #ty::make(
          self.#name.clone(),
          self.policy.clone()
        )
      }
    });


  let expanded_derive = quote! {
    #[typetag::serde]
    impl Policied for #name {
      fn get_policy(&self) -> &Box<dyn Policy> { &self.policy }
      fn remove_policy(&mut self) -> () { self.policy = Box::new(NonePolicy); }
    }

    impl #name {
      #expanded_protected
    }
  };

  

  TokenStream::from(expanded_derive)
}
