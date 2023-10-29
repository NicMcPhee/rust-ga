use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::push_state::parsing::{ExecStackInput, StacksInput};

pub fn derive_has_stack(
    struct_ident: &Ident,
    stacks: &StacksInput,
    _exec_stack: &ExecStackInput,
) -> TokenStream {
    // let mut stacks_to_derive_for = stacks
    let stacks_to_derive_for = stacks
        .iter()
        .map(|(ident, (_, ty))| (ident, ty))
        .collect::<Vec<_>>();
    // if let Some((ident, _, ty)) = &exec_stack {
    //     stacks_to_derive_for.push((ident, ty));
    // }

    stacks_to_derive_for
            .into_iter()
            .map(|(ident, ty)| {
                quote! {
                    #[automatically_derived]
                    impl ::push::push_vm::stack::HasStack<<#ty as ::push::push_vm::stack::StackType>::Type> for #struct_ident {
                        fn stack<U: ::push::push_vm::stack::TypeEq<This = <#ty as ::push::push_vm::stack::StackType>::Type>>(&self) -> &#ty {
                            &self.#ident
                        }

                        fn stack_mut<U: ::push::push_vm::stack::TypeEq<This = <#ty as ::push::push_vm::stack::StackType>::Type>>(&mut self) -> &mut #ty {
                            &mut self.#ident
                        }
                    }
                }
            })
            .collect::<proc_macro2::TokenStream>()
}