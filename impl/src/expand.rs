use crate::ast::{Enum, Field, Input, Struct};
use crate::attr::Trait;
use crate::fallback;
use crate::generics::InferredBounds;
use crate::unraw::MemberUnraw;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use std::collections::BTreeSet as Set;
use syn::{DeriveInput, GenericArgument, PathArguments, Result, Token, Type};

pub fn derive(input: &DeriveInput) -> TokenStream {
    match try_expand(input) {
        Ok(expanded) => expanded,
        // If there are invalid attributes in the input, expand to an Error impl
        // anyway to minimize spurious secondary errors in other code that uses
        // this type as an Error.
        Err(error) => fallback::expand(input, error),
    }
}

fn try_expand(input: &DeriveInput) -> Result<TokenStream> {
    let input = Input::from_syn(input)?;
    input.validate()?;
    Ok(match input {
        Input::Struct(input) => impl_struct(input),
        Input::Enum(input) => impl_enum(input),
    })
}

fn impl_struct(input: Struct) -> TokenStream {
    let ty = call_site_ident(&input.ident);
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut error_inferred_bounds = InferredBounds::new();

    let source_body = if let Some(transparent_attr) = &input.attrs.transparent {
        let only_field = &input.fields[0];
        if only_field.contains_generic {
            error_inferred_bounds.insert(only_field.ty, quote!(::wherror::__private::Error));
        }
        let member = &only_field.member;
        Some(quote_spanned! {transparent_attr.span=>
            ::wherror::__private::Error::source(self.#member.as_dyn_error())
        })
    } else if let Some(source_field) = input.source_field() {
        let source = &source_field.member;
        if source_field.contains_generic {
            let ty = unoptional_type(source_field.ty);
            error_inferred_bounds.insert(ty, quote!(::wherror::__private::Error + 'static));
        }
        let asref = if type_is_option(source_field.ty) {
            Some(quote_spanned!(source.span()=> .as_ref()?))
        } else {
            None
        };
        let dyn_error = quote_spanned! {source_field.source_span()=>
            self.#source #asref.as_dyn_error()
        };
        Some(quote! {
            ::core::option::Option::Some(#dyn_error)
        })
    } else {
        None
    };
    let source_method = source_body.map(|body| {
        quote! {
            fn source(&self) -> ::core::option::Option<&(dyn ::wherror::__private::Error + 'static)> {
                use ::wherror::__private::AsDynError as _;
                #body
            }
        }
    });

    let provide_method = input.backtrace_field().map(|backtrace_field| {
        let request = quote!(request);
        let backtrace = &backtrace_field.member;
        let body = if let Some(source_field) = input.source_field() {
            let source = &source_field.member;
            let source_provide = if type_is_option(source_field.ty) {
                quote_spanned! {source.span()=>
                    if let ::core::option::Option::Some(source) = &self.#source {
                        source.thiserror_provide(#request);
                    }
                }
            } else {
                quote_spanned! {source.span()=>
                    self.#source.thiserror_provide(#request);
                }
            };
            let self_provide = if source == backtrace {
                None
            } else if type_is_option(backtrace_field.ty) {
                Some(quote! {
                    if let ::core::option::Option::Some(backtrace) = &self.#backtrace {
                        #request.provide_ref::<::wherror::__private::Backtrace>(backtrace);
                    }
                })
            } else {
                Some(quote! {
                    #request.provide_ref::<::wherror::__private::Backtrace>(&self.#backtrace);
                })
            };
            quote! {
                use ::wherror::__private::ThiserrorProvide as _;
                #source_provide
                #self_provide
            }
        } else if type_is_option(backtrace_field.ty) {
            quote! {
                if let ::core::option::Option::Some(backtrace) = &self.#backtrace {
                    #request.provide_ref::<::wherror::__private::Backtrace>(backtrace);
                }
            }
        } else {
            quote! {
                #request.provide_ref::<::wherror::__private::Backtrace>(&self.#backtrace);
            }
        };
        quote! {
            fn provide<'_request>(&'_request self, #request: &mut ::core::error::Request<'_request>) {
                #body
            }
        }
    });

    let mut display_implied_bounds = Set::new();
    let display_body = if input.attrs.transparent.is_some() {
        let only_field = &input.fields[0].member;
        display_implied_bounds.insert((0, Trait::Display));
        Some(quote! {
            ::core::fmt::Display::fmt(&self.#only_field, __formatter)
        })
    } else if let Some(display) = &input.attrs.display {
        display_implied_bounds.clone_from(&display.implied_bounds);
        let use_as_display = use_as_display(display.has_bonus_display);
        let pat = fields_pat(&input.fields);
        Some(quote! {
            #use_as_display
            #[allow(unused_variables, deprecated)]
            let Self #pat = self;
            #display
        })
    } else if let Some(debug_attr) = &input.attrs.debug {
        // Fall back to Debug representation when #[error(debug)] is specified
        Some(quote_spanned! {debug_attr.span=>
            ::core::fmt::Debug::fmt(self, __formatter)
        })
    } else {
        None
    };
    let display_impl = display_body.map(|body| {
        let mut display_inferred_bounds = InferredBounds::new();
        for (field, bound) in display_implied_bounds {
            let field = &input.fields[field];
            if field.contains_generic {
                display_inferred_bounds.insert(field.ty, bound);
            }
        }
        let display_where_clause = display_inferred_bounds.augment_where_clause(input.generics);
        quote! {
            #[allow(unused_qualifications)]
            #[automatically_derived]
            impl #impl_generics ::core::fmt::Display for #ty #ty_generics #display_where_clause {
                #[allow(clippy::used_underscore_binding)]
                fn fmt(&self, __formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    #body
                }
            }
        }
    });

    let from_impl = input.from_field().map(|from_field| {
        let span = from_field.attrs.from.unwrap().span;
        let backtrace_field = input.distinct_backtrace_field();
        let from = unoptional_type(from_field.ty);
        let track_caller = input.location_field().map(|_| quote!(#[track_caller]));
        let source_var = Ident::new("source", span);
        let body = from_initializer(
            from_field,
            backtrace_field,
            &source_var,
            input.location_field(),
        );

        // Check if the field type (after unwrapping Option) is Box<T>
        let field_type = type_parameter_of_option(from_field.ty).unwrap_or(from_field.ty);
        let box_implementations = type_parameter_of_box(field_type).map(|inner_type| {
            // Generate From<T> implementation that boxes the value
            let inner_source_var = Ident::new("source", span);
            let boxed_body = from_initializer_for_box(
                from_field,
                backtrace_field,
                &inner_source_var,
                input.location_field(),
            );
            let inner_from_function = quote! {
                #track_caller
                fn from(#inner_source_var: #inner_type) -> Self {
                    #ty #boxed_body
                }
            };
            let inner_from_impl = quote_spanned! {span=>
                #[automatically_derived]
                impl #impl_generics ::core::convert::From<#inner_type> for #ty #ty_generics #where_clause {
                    #inner_from_function
                }
            };
            quote! {
                #[allow(
                    deprecated,
                    unused_qualifications,
                    clippy::elidable_lifetime_names,
                    clippy::needless_lifetimes,
                )]
                #inner_from_impl
            }
        });

        let from_function = quote! {
            #track_caller
            fn from(#source_var: #from) -> Self {
                #ty #body
            }
        };
        let from_impl = quote_spanned! {span=>
            #[automatically_derived]
            impl #impl_generics ::core::convert::From<#from> for #ty #ty_generics #where_clause {
                #from_function
            }
        };
        Some(quote! {
            #[allow(
                deprecated,
                unused_qualifications,
                clippy::elidable_lifetime_names,
                clippy::needless_lifetimes,
            )]
            #from_impl
            #box_implementations
        })
    });

    let location_impl = input.location_field().map(|location_field| {
        let location = &location_field.member;
        let body = if type_is_option(location_field.ty) {
            quote! {
                self.#location
            }
        } else {
            quote! {
                Some(self.#location)
            }
        };
        quote! {
            #[allow(unused_qualifications)]
            #[automatically_derived]
            impl #impl_generics #ty #ty_generics #where_clause {
                pub fn location(&self) -> Option<&'static ::core::panic::Location<'static>> {
                    #body
                }
            }
        }
    });

    if input.generics.type_params().next().is_some() {
        let self_token = <Token![Self]>::default();
        error_inferred_bounds.insert(self_token, Trait::Debug);
        error_inferred_bounds.insert(self_token, Trait::Display);
    }
    let error_where_clause = error_inferred_bounds.augment_where_clause(input.generics);

    quote! {
        #[allow(unused_qualifications)]
        #[automatically_derived]
        impl #impl_generics ::wherror::__private::Error for #ty #ty_generics #error_where_clause {
            #source_method
            #provide_method
        }
        #display_impl
        #from_impl
        #location_impl
    }
}

fn impl_enum(input: Enum) -> TokenStream {
    let ty = call_site_ident(&input.ident);
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut error_inferred_bounds = InferredBounds::new();

    let source_method = if input.has_source() {
        let arms = input.variants.iter().map(|variant| {
            let ident = &variant.ident;
            if let Some(transparent_attr) = &variant.attrs.transparent {
                let only_field = &variant.fields[0];
                if only_field.contains_generic {
                    error_inferred_bounds.insert(only_field.ty, quote!(::wherror::__private::Error));
                }
                let member = &only_field.member;
                let source = quote_spanned! {transparent_attr.span=>
                    ::wherror::__private::Error::source(transparent.as_dyn_error())
                };
                quote! {
                    #ty::#ident {#member: transparent} => #source,
                }
            } else if let Some(source_field) = variant.source_field() {
                let source = &source_field.member;
                if source_field.contains_generic {
                    let ty = unoptional_type(source_field.ty);
                    error_inferred_bounds.insert(ty, quote!(::wherror::__private::Error + 'static));
                }
                let asref = if type_is_option(source_field.ty) {
                    Some(quote_spanned!(source.span()=> .as_ref()?))
                } else {
                    None
                };
                let varsource = quote!(source);
                let dyn_error = quote_spanned! {source_field.source_span()=>
                    #varsource #asref.as_dyn_error()
                };
                quote! {
                    #ty::#ident {#source: #varsource, ..} => ::core::option::Option::Some(#dyn_error),
                }
            } else {
                quote! {
                    #ty::#ident {..} => ::core::option::Option::None,
                }
            }
        });
        Some(quote! {
            fn source(&self) -> ::core::option::Option<&(dyn ::wherror::__private::Error + 'static)> {
                use ::wherror::__private::AsDynError as _;
                #[allow(deprecated)]
                match self {
                    #(#arms)*
                }
            }
        })
    } else {
        None
    };

    let provide_method = if input.has_backtrace() {
        let request = quote!(request);
        let arms = input.variants.iter().map(|variant| {
            let ident = &variant.ident;
            match (variant.backtrace_field(), variant.source_field()) {
                (Some(backtrace_field), Some(source_field))
                    if backtrace_field.attrs.backtrace.is_none() =>
                {
                    let backtrace = &backtrace_field.member;
                    let source = &source_field.member;
                    let varsource = quote!(source);
                    let source_provide = if type_is_option(source_field.ty) {
                        quote_spanned! {source.span()=>
                            if let ::core::option::Option::Some(source) = #varsource {
                                source.thiserror_provide(#request);
                            }
                        }
                    } else {
                        quote_spanned! {source.span()=>
                            #varsource.thiserror_provide(#request);
                        }
                    };
                    let self_provide = if type_is_option(backtrace_field.ty) {
                        quote! {
                            if let ::core::option::Option::Some(backtrace) = backtrace {
                                #request.provide_ref::<::wherror::__private::Backtrace>(backtrace);
                            }
                        }
                    } else {
                        quote! {
                            #request.provide_ref::<::wherror::__private::Backtrace>(backtrace);
                        }
                    };
                    quote! {
                        #ty::#ident {
                            #backtrace: backtrace,
                            #source: #varsource,
                            ..
                        } => {
                            use ::wherror::__private::ThiserrorProvide as _;
                            #source_provide
                            #self_provide
                        }
                    }
                }
                (Some(backtrace_field), Some(source_field))
                    if backtrace_field.member == source_field.member =>
                {
                    let backtrace = &backtrace_field.member;
                    let varsource = quote!(source);
                    let source_provide = if type_is_option(source_field.ty) {
                        quote_spanned! {backtrace.span()=>
                            if let ::core::option::Option::Some(source) = #varsource {
                                source.thiserror_provide(#request);
                            }
                        }
                    } else {
                        quote_spanned! {backtrace.span()=>
                            #varsource.thiserror_provide(#request);
                        }
                    };
                    quote! {
                        #ty::#ident {#backtrace: #varsource, ..} => {
                            use ::wherror::__private::ThiserrorProvide as _;
                            #source_provide
                        }
                    }
                }
                (Some(backtrace_field), _) => {
                    let backtrace = &backtrace_field.member;
                    let body = if type_is_option(backtrace_field.ty) {
                        quote! {
                            if let ::core::option::Option::Some(backtrace) = backtrace {
                                #request.provide_ref::<::wherror::__private::Backtrace>(backtrace);
                            }
                        }
                    } else {
                        quote! {
                            #request.provide_ref::<::wherror::__private::Backtrace>(backtrace);
                        }
                    };
                    quote! {
                        #ty::#ident {#backtrace: backtrace, ..} => {
                            #body
                        }
                    }
                }
                (None, _) => quote! {
                    #ty::#ident {..} => {}
                },
            }
        });
        Some(quote! {
            fn provide<'_request>(&'_request self, #request: &mut ::core::error::Request<'_request>) {
                #[allow(deprecated)]
                match self {
                    #(#arms)*
                }
            }
        })
    } else {
        None
    };

    let display_impl = if input.has_display() {
        let mut display_inferred_bounds = InferredBounds::new();
        let has_bonus_display = input.variants.iter().any(|v| {
            v.attrs
                .display
                .as_ref()
                .map_or(false, |display| display.has_bonus_display)
        });
        let use_as_display = use_as_display(has_bonus_display);
        let void_deref = if input.variants.is_empty() {
            Some(quote!(*))
        } else {
            None
        };
        let arms = input.variants.iter().map(|variant| {
            let mut display_implied_bounds = Set::new();
            let display = if let Some(display) = &variant.attrs.display {
                display_implied_bounds.clone_from(&display.implied_bounds);
                display.to_token_stream()
            } else if let Some(fmt) = &variant.attrs.fmt {
                let fmt_path = &fmt.path;
                let vars = variant.fields.iter().map(|field| match &field.member {
                    MemberUnraw::Named(ident) => ident.to_local(),
                    MemberUnraw::Unnamed(index) => format_ident!("_{}", index),
                });
                quote!(#fmt_path(#(#vars,)* __formatter))
            } else if let Some(_transparent_attr) = &variant.attrs.transparent {
                // Transparent: forward to the single field's Display implementation
                let only_field = match &variant.fields[0].member {
                    MemberUnraw::Named(ident) => ident.to_local(),
                    MemberUnraw::Unnamed(index) => format_ident!("_{}", index),
                };
                display_implied_bounds.insert((0, Trait::Display));
                quote!(::core::fmt::Display::fmt(#only_field, __formatter))
            } else if let Some(debug_attr) = &variant.attrs.debug {
                // Variant-level #[error(debug)]: debug the variant fields directly
                let ident = &variant.ident;
                let debug_span = debug_attr.span;
                if variant.fields.is_empty() {
                    // Unit variant: just display the variant name
                    let variant_name = ident.to_string();
                    quote_spanned! {debug_span=>
                        ::core::write!(__formatter, "{}", #variant_name)
                    }
                } else if variant.fields.len() == 1 && matches!(variant.fields[0].member, MemberUnraw::Unnamed(_)) {
                    // Tuple variant with single field: Format as Variant(field)
                    let mut field_var = format_ident!("_0");
                    field_var.set_span(debug_span);
                    quote_spanned! {debug_span=>
                        ::core::write!(__formatter, "{}({:?})", stringify!(#ident), #field_var)
                    }
                } else if variant.fields.iter().all(|f| matches!(f.member, MemberUnraw::Unnamed(_))) {
                    // Tuple variant with multiple fields: Format as Variant(field1, field2, ...)
                    let field_vars: Vec<_> = variant.fields.iter().enumerate().map(|(i, _)| {
                        let var = format_ident!("_{}", i);
                        quote_spanned! {debug_span=> #var}
                    }).collect();
                    quote_spanned! {debug_span=>
                        ::core::write!(__formatter, "{}({:?})", stringify!(#ident), (#(#field_vars,)*))
                    }
                } else {
                    // Struct variant: generate proper debug formatting showing all field values
                    let field_writes: Vec<_> = variant.fields.iter().enumerate().map(|(i, field)| {
                        let comma = if i < variant.fields.len() - 1 {
                            quote_spanned! {debug_span=> ::core::write!(__formatter, ", ")?; }
                        } else {
                            quote_spanned! {debug_span=> }
                        };
                        match &field.member {
                            MemberUnraw::Named(ident) => {
                                let field_name = ident.to_string();
                                let var = ident.to_local();
                                quote_spanned! {debug_span=>
                                    ::core::write!(__formatter, "{}: {:?}", #field_name, #var)?;
                                    #comma
                                }
                            }
                            MemberUnraw::Unnamed(index) => {
                                let var = format_ident!("_{}", index);
                                quote_spanned! {debug_span=>
                                    ::core::write!(__formatter, "{:?}", #var)?;
                                    #comma
                                }
                            }
                        }
                    }).collect();

                    quote_spanned! {debug_span=>
                        {
                            ::core::write!(__formatter, "{} {{ ", stringify!(#ident))?;
                            #(#field_writes)*
                            ::core::write!(__formatter, " }}")
                        }
                    }
                }
            } else if let Some(debug_attr) = &input.attrs.debug {
                // Top-level #[error(debug)]: fall back to Debug representation of the variant
                // This implements the feature request from issue #172
                let ident = &variant.ident;
                if variant.fields.is_empty() {
                    // Unit variant: just display the variant name
                    let variant_name = ident.to_string();
                    quote_spanned! {debug_attr.span=>
                        ::core::write!(__formatter, "{}", #variant_name)
                    }
                } else if variant.fields.len() == 1 && matches!(variant.fields[0].member, MemberUnraw::Unnamed(_)) {
                    // Tuple variant with single field: Format as Variant(field)
                    let mut field_var = format_ident!("_0");
                    field_var.set_span(debug_attr.span);
                    quote_spanned! {debug_attr.span=>
                        ::core::write!(__formatter, "{}({:?})", stringify!(#ident), #field_var)
                    }
                } else if variant.fields.iter().all(|f| matches!(f.member, MemberUnraw::Unnamed(_))) {
                    // Tuple variant with multiple fields: Format as Variant(field1, field2, ...)
                    let field_vars: Vec<_> = variant.fields.iter().enumerate().map(|(i, _)| {
                        let mut var = format_ident!("_{}", i);
                        var.set_span(debug_attr.span);
                        var
                    }).collect();
                    quote_spanned! {debug_attr.span=>
                        ::core::write!(__formatter, "{}({:?})", stringify!(#ident), (#(#field_vars,)*))
                    }
                } else {
                    // Struct variant: generate proper debug formatting showing all field values
                    let field_writes: Vec<_> = variant.fields.iter().enumerate().map(|(i, field)| {
                        let comma = if i < variant.fields.len() - 1 {
                            quote_spanned! {debug_attr.span=> ::core::write!(__formatter, ", ")?; }
                        } else {
                            quote_spanned! {debug_attr.span=> }
                        };
                        match &field.member {
                            MemberUnraw::Named(ident) => {
                                let field_name = ident.to_string();
                                let var = ident.to_local();
                                quote_spanned! {debug_attr.span=>
                                    ::core::write!(__formatter, "{}: {:?}", #field_name, #var)?;
                                    #comma
                                }
                            }
                            MemberUnraw::Unnamed(index) => {
                                let var = format_ident!("_{}", index);
                                quote_spanned! {debug_attr.span=>
                                    ::core::write!(__formatter, "{:?}", #var)?;
                                    #comma
                                }
                            }
                        }
                    }).collect();

                    quote_spanned! {debug_attr.span=>
                        {
                            ::core::write!(__formatter, "{} {{ ", stringify!(#ident))?;
                            #(#field_writes)*
                            ::core::write!(__formatter, " }}")
                        }
                    }
                }
            } else {
                // This should be caught by validation - no display attribute and no fallback
                quote!(unreachable!("missing display attribute should have been caught by validation"))
            };
            for (field, bound) in display_implied_bounds {
                let field = &variant.fields[field];
                if field.contains_generic {
                    display_inferred_bounds.insert(field.ty, bound);
                }
            }
            let ident = &variant.ident;
            let pat = fields_pat(&variant.fields);
            quote! {
                #ty::#ident #pat => #display
            }
        });
        let arms = arms.collect::<Vec<_>>();
        let display_where_clause = display_inferred_bounds.augment_where_clause(input.generics);
        Some(quote! {
            #[allow(unused_qualifications)]
            #[automatically_derived]
            impl #impl_generics ::core::fmt::Display for #ty #ty_generics #display_where_clause {
                fn fmt(&self, __formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    #use_as_display
                    #[allow(unused_variables, deprecated, clippy::used_underscore_binding)]
                    match #void_deref self {
                        #(#arms,)*
                    }
                }
            }
        })
    } else {
        None
    };

    let from_impls = input.variants.iter().flat_map(|variant| {
        let from_field = variant.from_field()?;
        let span = from_field.attrs.from.unwrap().span;
        let backtrace_field = variant.distinct_backtrace_field();
        let location_field = variant.location_field();
        let variant_ident = &variant.ident;
        let from = unoptional_type(from_field.ty);
        let source_var = Ident::new("source", span);
        let body = from_initializer(from_field, backtrace_field, &source_var, location_field);
        let track_caller = location_field.map(|_| quote!(#[track_caller]));

        let mut implementations = Vec::new();

        // Main From implementation (always generated)
        let from_function = quote! {
            #track_caller
            fn from(#source_var: #from) -> Self {
                #ty::#variant_ident #body
            }
        };
        let from_impl = quote_spanned! {span=>
            #[automatically_derived]
            impl #impl_generics ::core::convert::From<#from> for #ty #ty_generics #where_clause {
                #from_function
            }
        };
        implementations.push(quote! {
            #[allow(
                deprecated,
                unused_qualifications,
                clippy::elidable_lifetime_names,
                clippy::needless_lifetimes,
            )]
            #from_impl
        });

        // Check if the field type (after unwrapping Option) is Box<T>
        let field_type = type_parameter_of_option(from_field.ty).unwrap_or(from_field.ty);
        if let Some(inner_type) = type_parameter_of_box(field_type) {
            // Generate additional From<T> implementation that boxes the value
            let inner_source_var = Ident::new("source", span);
            let boxed_body = from_initializer_for_box(
                from_field,
                backtrace_field,
                &inner_source_var,
                location_field,
            );
            let inner_from_function = quote! {
                #track_caller
                fn from(#inner_source_var: #inner_type) -> Self {
                    #ty::#variant_ident #boxed_body
                }
            };
            let inner_from_impl = quote_spanned! {span=>
                #[automatically_derived]
                impl #impl_generics ::core::convert::From<#inner_type> for #ty #ty_generics #where_clause {
                    #inner_from_function
                }
            };
            implementations.push(quote! {
                #[allow(
                    deprecated,
                    unused_qualifications,
                    clippy::elidable_lifetime_names,
                    clippy::needless_lifetimes,
                )]
                #inner_from_impl
            });
        }

        Some(implementations)
    }).flatten();

    let location_impl = if input.has_location() {
        let arms = input.variants.iter().map(|variant| {
            let ident = &variant.ident;
            if let Some(location_field) = variant.location_field() {
                let location = &location_field.member;
                let var_location = quote!(location);
                let body = if type_is_option(location_field.ty) {
                    quote! {
                        #var_location
                    }
                } else {
                    quote! {
                        Some(#var_location)
                    }
                };
                quote! {
                    #ty::#ident {#location: #var_location, ..} => #body,
                }
            } else {
                quote! {
                    #ty::#ident {..} => None,
                }
            }
        });
        Some(quote! {
            #[allow(unused_qualifications)]
            #[automatically_derived]
            impl #impl_generics #ty #ty_generics #where_clause {
                pub fn location(&self) -> Option<&'static ::core::panic::Location<'static>> {
                    #[allow(deprecated)]
                    match self {
                        #(#arms)*
                    }
                }
            }
        })
    } else {
        None
    };

    if input.generics.type_params().next().is_some() {
        let self_token = <Token![Self]>::default();
        error_inferred_bounds.insert(self_token, Trait::Debug);
        error_inferred_bounds.insert(self_token, Trait::Display);
    }
    let error_where_clause = error_inferred_bounds.augment_where_clause(input.generics);

    quote! {
        #[allow(unused_qualifications)]
        #[automatically_derived]
        impl #impl_generics ::wherror::__private::Error for #ty #ty_generics #error_where_clause {
            #source_method
            #provide_method
        }
        #display_impl
        #(#from_impls)*
        #location_impl
    }
}

// Create an ident with which we can expand `impl Trait for #ident {}` on a
// deprecated type without triggering deprecation warning on the generated impl.
pub(crate) fn call_site_ident(ident: &Ident) -> Ident {
    let mut ident = ident.clone();
    ident.set_span(ident.span().resolved_at(Span::call_site()));
    ident
}

fn fields_pat(fields: &[Field]) -> TokenStream {
    let mut members = fields.iter().map(|field| &field.member).peekable();
    match members.peek() {
        Some(MemberUnraw::Named(_)) => quote!({ #(#members),* }),
        Some(MemberUnraw::Unnamed(_)) => {
            let vars = members.map(|member| match member {
                MemberUnraw::Unnamed(index) => format_ident!("_{}", index),
                MemberUnraw::Named(_) => unreachable!(),
            });
            quote!((#(#vars),*))
        }
        None => quote!({}),
    }
}

fn use_as_display(needs_as_display: bool) -> Option<TokenStream> {
    if needs_as_display {
        Some(quote! {
            use ::wherror::__private::AsDisplay as _;
        })
    } else {
        None
    }
}

fn from_initializer(
    from_field: &Field,
    backtrace_field: Option<&Field>,
    source_var: &Ident,
    location_field: Option<&Field>,
) -> TokenStream {
    let from_member = &from_field.member;
    let some_source = if type_is_option(from_field.ty) {
        quote!(::core::option::Option::Some(#source_var))
    } else {
        quote!(#source_var)
    };
    let backtrace = backtrace_field.map(|backtrace_field| {
        let backtrace_member = &backtrace_field.member;
        if type_is_option(backtrace_field.ty) {
            quote! {
                #backtrace_member: ::core::option::Option::Some(::wherror::__private::Backtrace::capture()),
            }
        } else {
            quote! {
                #backtrace_member: ::core::convert::From::from(::wherror::__private::Backtrace::capture()),
            }
        }
    });
    let location = location_field.map(|location_field| {
        let location_member = &location_field.member;

        if type_is_option(location_field.ty) {
            quote! {
                #location_member: ::core::option::Option::Some(::core::panic::Location::caller()),
            }
        } else {
            quote! {
                #location_member: ::core::convert::From::from(::core::panic::Location::caller()),
            }
        }
    });
    quote!({
        #from_member: #some_source,
        #backtrace
        #location
    })
}

fn from_initializer_for_box(
    from_field: &Field,
    backtrace_field: Option<&Field>,
    source_var: &Ident,
    location_field: Option<&Field>,
) -> TokenStream {
    let from_member = &from_field.member;
    // For Box<T> fields, we need to Box::new the source when receiving T
    let some_source = if type_is_option(from_field.ty) {
        quote!(::core::option::Option::Some(::std::boxed::Box::new(#source_var)))
    } else {
        quote!(::std::boxed::Box::new(#source_var))
    };
    let backtrace = backtrace_field.map(|backtrace_field| {
        let backtrace_member = &backtrace_field.member;
        if type_is_option(backtrace_field.ty) {
            quote! {
                #backtrace_member: ::core::option::Option::Some(::wherror::__private::Backtrace::capture()),
            }
        } else {
            quote! {
                #backtrace_member: ::core::convert::From::from(::wherror::__private::Backtrace::capture()),
            }
        }
    });
    let location = location_field.map(|location_field| {
        let location_member = &location_field.member;

        if type_is_option(location_field.ty) {
            quote! {
                #location_member: ::core::option::Option::Some(::core::panic::Location::caller()),
            }
        } else {
            quote! {
                #location_member: ::core::convert::From::from(::core::panic::Location::caller()),
            }
        }
    });
    quote!({
        #from_member: #some_source,
        #backtrace
        #location
    })
}

fn type_is_option(ty: &Type) -> bool {
    type_parameter_of_option(ty).is_some()
}

fn unoptional_type(ty: &Type) -> TokenStream {
    let unoptional = type_parameter_of_option(ty).unwrap_or(ty);
    quote!(#unoptional)
}

fn type_parameter_of_option(ty: &Type) -> Option<&Type> {
    let path = match ty {
        Type::Path(ty) => &ty.path,
        _ => return None,
    };

    let last = path.segments.last().unwrap();
    if last.ident != "Option" {
        return None;
    }

    let bracketed = match &last.arguments {
        PathArguments::AngleBracketed(bracketed) => bracketed,
        _ => return None,
    };

    if bracketed.args.len() != 1 {
        return None;
    }

    match &bracketed.args[0] {
        GenericArgument::Type(arg) => Some(arg),
        _ => None,
    }
}

fn type_parameter_of_box(ty: &Type) -> Option<&Type> {
    let path = match ty {
        Type::Path(ty) => &ty.path,
        _ => return None,
    };

    let last = path.segments.last().unwrap();
    if last.ident != "Box" {
        return None;
    }

    let bracketed = match &last.arguments {
        PathArguments::AngleBracketed(bracketed) => bracketed,
        _ => return None,
    };

    if bracketed.args.len() != 1 {
        return None;
    }

    match &bracketed.args[0] {
        GenericArgument::Type(arg) => Some(arg),
        _ => None,
    }
}
