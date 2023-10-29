use quote::quote;
use ident_case_conversions::CaseConversions;
use proc_macro2::{TokenStream, Span};
use syn::{Ident, ext::IdentExt, Visibility, Generics};

use crate::push_state::parsing::{StacksInput, ExecStackInput, stack_attribute_args::StackMarkerFlags, InputInstructionsInput};

pub fn generate_builder(macro_span: Span,struct_ident: &Ident,struct_visibility: &Visibility, struct_generics: &Generics, stacks: &StacksInput, exec_stack: &ExecStackInput, input_instructions: InputInstructionsInput) -> syn::Result<TokenStream> {
            let Some((exec_stack_ident, _, _)) = exec_stack else {
                return Err(syn::Error::new(macro_span, "Need to declare exactly one exec stack using #[stack(exec)] to use the builder feature."))
            };

            let struct_ident_unrawed = struct_ident.unraw();
            let struct_ident_unrawed_snake_case = struct_ident_unrawed.to_snake_case().unraw();
            let utilities_mod_ident = syn::Ident::new_raw(&format!("{struct_ident_unrawed_snake_case}_builder"), proc_macro2::Span::mixed_site());

            let builder_name = syn::Ident::new_raw(&format!("{struct_ident_unrawed}Builder"), proc_macro2::Span::mixed_site());

            let stack_generics = stacks.keys().map(|i| i.unraw().to_pascal_case_spanned(proc_macro2::Span::mixed_site())).collect::<Vec<_>>();

            let stack_generics_with_state_bounds = stack_generics.iter().map(|g| quote!{#g: #utilities_mod_ident::StackState}).collect::<Vec<_>>();

            let stack_generics_with_dataless_bounds = stack_generics.iter().map(|g| quote!{#g: #utilities_mod_ident::Dataless}).collect::<Vec<_>>();

            let default_states = stack_generics.iter().map(|_| quote!{()}).collect::<Vec<_>>();

            let with_size_repeated = stack_generics.iter().map(|_| quote!{
                #utilities_mod_ident::WithSize
            }).collect::<Vec<_>>();

            let (impl_generics, type_generics, where_clause) = struct_generics.split_for_impl();

            let fields = stacks.keys().collect::<Vec<_>>();

            let with_inputs_impl = input_instructions.map(|input_instructions_field| {
                let with_inputs = stacks.iter().map(|(field, (StackMarkerFlags{builder_name, instruction_name, ..}, ty))| {
                    let stack_ident = builder_name.as_ref().unwrap_or(field).unraw().to_snake_case().unraw();
                    let instruction_path = instruction_name.clone().unwrap_or_else(|| {
                        let snake_case_field = field.unraw().to_snake_case().unraw();

                        let instruction_fn_name = syn::Ident::new(&format!("push_{snake_case_field}"), proc_macro2::Span::mixed_site());

                        syn::parse_quote!(::push::instruction::PushInstruction::#instruction_fn_name)
                    });

                    let fn_ident = syn::Ident::new(&format!("with_{stack_ident}_input"), proc_macro2::Span::mixed_site());

                    quote!{
                        /// Adds a input instruction to the current current state's set
                        /// of instructions. The name for the input must have been included
                        /// in the `Inputs` provided when the `Builder` was initially constructed.
                        /// Here you provide the name and the boolean value for that
                        /// input variable. That will create a new `PushInstruction::push_[type]()`
                        /// instruction that will push the specified value onto the stack
                        /// when performed.
                        ///
                        /// # Panics
                        /// This panics if the `input_name` provided isn't included in the set of
                        /// names in the `Inputs` object used in the construction of the `Builder`.
                        #[must_use]
                        pub fn #fn_ident(mut self, input_name: &str, input_value: <#ty as ::push::push_vm::stack::StackType>::Type) -> Self {
                            self.partial_state.#input_instructions_field.insert(
                                ::push::instruction::VariableName::from(input_name),
                                #instruction_path(input_value),
                            );
                            self
                        }
                    }
                });

                quote!{
                    impl<__Exec: #utilities_mod_ident::StackState, #(#stack_generics_with_state_bounds),*> #builder_name<__Exec, #(#stack_generics),*> {
                        #(#with_inputs)*
                    }
                }
            });

            let with_values_impl = stacks.iter().map(|(field, (StackMarkerFlags { builder_name: builder_methods_name, .. }, ty))| {
                let stack_ident = builder_methods_name.as_ref().unwrap_or(field).unraw().to_snake_case().unraw();

                let fn_ident = syn::Ident::new(&format!("with_{stack_ident}_values"), proc_macro2::Span::mixed_site());

                let where_bounds = stacks.keys().map(|ident| {
                    let generic_name = ident.unraw().to_pascal_case_spanned(proc_macro2::Span::mixed_site()).unraw();
                    if ident == field {
                        quote!{#generic_name: #utilities_mod_ident::SizeSet}
                    } else {
                        quote!{#generic_name: #utilities_mod_ident::StackState}
                    }
                });

                let stack_generics_or_type = stacks.keys().map(|ident| {
                    if ident == field {
                        quote!{#utilities_mod_ident::WithSizeAndData}
                    } else {
                        let generic_name = ident.unraw().to_pascal_case_spanned(proc_macro2::Span::mixed_site()).unraw();
                        quote!{#generic_name}
                    }
                    
                });

                quote!{
                    impl<__Exec: #utilities_mod_ident::StackState, #(#where_bounds),*> #builder_name<__Exec, #(#stack_generics),*> {
                        /// Adds the given sequence of values to the stack for the state you're building.
                        ///
                        /// The first value in `values` will be the new top of the
                        /// stack. If the stack was initially empty, the last value
                        /// in `values` will be the new bottom of the stack.
                        ///
                        /// # Arguments
                        ///
                        /// * `values` - A `Vec` holding the values to add to the stack
                        ///
                        /// # Examples
                        ///
                        /// ```ignore
                        /// use push::push_vm::push_state::{ Stack, HasStack, PushState, Builder };
                        /// let mut state = Builder::new(PushState::default())
                        ///     .with_int_values(vec![5, 8, 9])
                        ///     .build();
                        /// let int_stack: &Stack<PushInteger> = state.stack();
                        /// assert_eq!(int_stack.size(), 3);
                        /// // Now the top of the stack is 5, followed by 8, then 9 at the bottom.
                        /// assert_eq!(int_stack.top().unwrap(), &5);
                        /// ```
                        #[must_use]
                        pub fn #fn_ident(mut self, values: Vec<<#ty as ::push::push_vm::stack::StackType>::Type>) -> #builder_name<__Exec, #(#stack_generics_or_type),*> {
                            self.partial_state.#field.extend(values);

                            #builder_name {
                                partial_state: self.partial_state,
                                _p: ::std::marker::PhantomData,
                            }
                        }
                    }
                }
            }).collect::<proc_macro2::TokenStream>();

            let set_max_size_impl = stacks.iter().map(|(field, (StackMarkerFlags { builder_name: builder_methods_name, .. }, _))| {
                let stack_ident = builder_methods_name.as_ref().unwrap_or(field).unraw().to_snake_case().unraw();

                let fn_ident = syn::Ident::new(&format!("with_{stack_ident}_max_size"), proc_macro2::Span::mixed_site());

                let where_bounds = stacks.keys().map(|ident| {
                    let generic_name = ident.unraw().to_pascal_case_spanned(proc_macro2::Span::mixed_site()).unraw();
                    if ident == field {
                        quote!{#generic_name: #utilities_mod_ident::Dataless}
                    } else {
                        quote!{#generic_name: #utilities_mod_ident::StackState}
                    }
                });

                let stack_generics_or_type = stacks.keys().map(|ident| {
                    if ident == field {
                        quote!{#utilities_mod_ident::WithSize}
                    } else {
                        let generic_name = ident.unraw().to_pascal_case_spanned(proc_macro2::Span::mixed_site()).unraw();
                        quote!{#generic_name}
                    }
                    
                });

                quote!{
                    impl<__Exec: #utilities_mod_ident::StackState, #(#where_bounds),*> #builder_name<__Exec, #(#stack_generics),*> {
                        /// Sets the maximum stack size for the stack in this state.
                        ///
                        /// # Arguments
                        ///
                        /// * `max_stack_size` - A `usize` specifying the maximum stack size
                        #[must_use]
                        pub fn #fn_ident(mut self, max_stack_size: usize) ->#builder_name<__Exec, #(#stack_generics_or_type),*>  {
                            self.partial_state.#field.set_max_stack_size(max_stack_size);

                            #builder_name {
                                partial_state: self.partial_state,
                                _p: ::std::marker::PhantomData,
                            }
                        }
                    }
                }
            }).collect::<proc_macro2::TokenStream>();


            Ok(quote!{
                impl #impl_generics #struct_ident #type_generics #where_clause {
                    #[must_use]
                    #struct_visibility fn builder() -> #builder_name<(),#(#default_states),*>{
                        #builder_name::<(),#(#default_states),*>::default()
                    }
                }
                
                #struct_visibility mod #utilities_mod_ident {
                    mod sealed {
                        pub trait SealedMarker {}
                    }

                    pub trait StackState: sealed::SealedMarker {}
                    pub trait Dataless: StackState {}
                    pub trait SizeSet: StackState {}

                    impl sealed::SealedMarker for () {}
                    impl StackState for () {}
                    impl Dataless for () {}

                    pub struct WithSize;
                    impl sealed::SealedMarker for WithSize {}
                    impl StackState for WithSize {}
                    impl Dataless for WithSize {}
                    impl SizeSet for WithSize {}

                    pub struct WithSizeAndData;
                    impl sealed::SealedMarker for WithSizeAndData {}
                    impl StackState for WithSizeAndData {}
                    impl SizeSet for WithSizeAndData {}
                }

                #struct_visibility struct #builder_name<__Exec: #utilities_mod_ident::StackState, #(#stack_generics_with_state_bounds),*> {
                    partial_state: #struct_ident,
                    _p: std::marker::PhantomData<(__Exec, #(#stack_generics),*)>
                }

                impl ::std::default::Default for #builder_name<(), #(#default_states),*> {
                    fn default() -> Self {
                        #builder_name {
                            partial_state: ::std::default::Default::default(),
                            _p: ::std::marker::PhantomData,
                        }
                    }
                }

                impl<__Exec: #utilities_mod_ident::Dataless, #(#stack_generics_with_dataless_bounds),*> #builder_name<__Exec, #(#stack_generics),*> {
                    /// Sets the maximum stack size for all the stacks in this state.
                    ///
                    /// # Arguments
                    ///
                    /// * `max_stack_size` - A `usize` specifying the maximum stack size
                    ///
                    /// # Examples
                    ///
                    /// ```ignore
                    /// use push::push_vm::HasStack;
                    /// use push::push_vm::push_state::{ Stack, HasStack, PushState, Builder };
                    /// let mut state = Builder::new(PushState::default())
                    ///     .with_max_stack_size(100)
                    ///     .build();
                    /// let bool_stack: &Stack<bool> = state.stack();
                    /// assert_eq!(bool_stack.max_stack_size, 100);
                    /// ```
                    #[must_use]
                    pub fn with_max_stack_size(
                        mut self,
                        max_size: usize,
                    ) -> #builder_name<#utilities_mod_ident::WithSize, #(#with_size_repeated),*> {
                        self.partial_state
                            .exec
                            .reserve(max_size - self.partial_state.exec().len());

                        #(
                            self.partial_state.#fields.set_max_stack_size(max_size);
                        )*

                        #builder_name {
                            partial_state: self.partial_state,
                            _p: ::std::marker::PhantomData,
                        }
                    }
                }

                impl<#(#stack_generics_with_state_bounds),*> #builder_name<#utilities_mod_ident::WithSize, #(#stack_generics),*> {
                    /// Sets the program you wish to execute.
                    /// Note that the program will be executed in ascending order.
                    ///
                    /// # Arguments
                    /// - `program` - The program you wish to execute
                    #[must_use]
                    pub fn with_program<P>(mut self, program: P) -> #builder_name<#utilities_mod_ident::WithSizeAndData, #(#stack_generics),*>
                    where
                        P: ::std::iter::IntoIterator<Item = ::push::instruction::PushInstruction>,
                        <P as ::std::iter::IntoIterator>::IntoIter: ::std::iter::DoubleEndedIterator,
                    {
                        self.partial_state.#exec_stack_ident =program.into_iter().rev().collect();
                        #builder_name {
                            partial_state: self.partial_state,
                            _p: ::std::marker::PhantomData,
                        }
                    }
                }

                impl<#(#stack_generics_with_state_bounds),*> #builder_name<#utilities_mod_ident::WithSizeAndData, #(#stack_generics),*> {
                    /// Finalize the build process, returning the fully constructed `PushState`
                    /// value. For this to successfully build, all the input variables has to
                    /// have been given values. Thus every input variable provided
                    /// in the `Inputs` used when constructing the `Builder` must have had a
                    /// corresponding `with_X_input()` call that specified the value for that
                    /// variable.
                    ///
                    /// # Panics
                    /// Panics if one or more of the variables provided in the `Inputs` wasn't
                    /// then given a value during the build process.
                    /*
                     * Note that the `with_x_input()` functions ensure that the instruction for
                     * that input variable will be in the same position in `self.input_instructions`
                     * as the name is in `self.inputs.input_names`. This allows us to zip together
                     * those two lists and know that we'll be pairing up instructions with the appropriate
                     * names.
                     */
                    #[must_use]
                    pub fn build(self) -> #struct_ident {
                        self.partial_state
                    }
                }

                #with_inputs_impl
                #with_values_impl
                #set_max_size_impl

            })

    
}