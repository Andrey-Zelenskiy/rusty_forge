// Copyright Andrey Zelenskiy, 2024-2026

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, Data, DataEnum, DataStruct, DeriveInput, Fields,
    GenericArgument, PathArguments, Type,
};

// Derive methods for Builders to set field values without boilerplate
#[proc_macro_derive(BuilderSetters, attributes(setter))]
pub fn derive_setters(input: TokenStream) -> TokenStream {
    // Get information about the type
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) =
        input.generics.split_for_impl();

    let expanded = match &input.data {
        Data::Struct(data) => generate_struct_setters(data),
        Data::Enum(data) => generate_enum_setters(data),
        _ => panic!("BuilderSetter only supports Structs and Enums"),
    };

    TokenStream::from(quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #expanded
        }
    })
}

// Specific derive implementation for Structs
fn generate_struct_setters(data: &DataStruct) -> proc_macro2::TokenStream {
    // Ensure that the fields are named
    let fields = match &data.fields {
        Fields::Named(f) => &f.named,
        _ => panic!("BuilderSetters only supports Structs with named fields"),
    };

    let setters = fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        let field_ty = &f.ty;
        let setter_name = format_ident!("set_{}", field_name);

        // Check if the field is marked with #[setter(nested)]
        let mut is_nested = false;
        for attr in &f.attrs {
            if attr.path().is_ident("setter") {
                let _ = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("nested") {
                        is_nested = true;
                    }
                    Ok(())
                });
            }
        }

        if is_nested {
            // Nested setter
            quote! {
                pub fn #setter_name<F>(&mut self, f: F) -> &mut Self
                where F: FnOnce(&mut #field_ty) -> &mut #field_ty
                {
                    f(&mut self.#field_name);
                    self
                }
            }
        }
        else if let Some(inner_ty) = get_option_inner_type(field_ty) {
            // Option<T> unwrapping setter
            quote! {
                pub fn #setter_name<P:Into<#inner_ty>>(&mut self, value:P) -> & mut Self{
                    self.#field_name = Some(value.into());
                    self
                }
            }
        }
        else {
            // Standard value setter
            quote! {
                pub fn #setter_name<P:Into<#field_ty>>(&mut self, value:P) -> & mut Self{
                    self.#field_name = value.into();
                    self
                }
            }
        }
    });

    quote! {#(#setters)*}
}

// Specific derive implementation for Enums
fn generate_enum_setters(data: &DataEnum) -> proc_macro2::TokenStream {
    let variants = data.variants.iter().map(|v| {
        let var_name = &v.ident;
        let method_name = format_ident!("set_{}", var_name.to_string().to_lowercase());

        // Check enum structure
        match &v.fields {
            Fields::Unit => quote! {
                pub fn #method_name(&mut self) -> &mut Self {
                    self = Self::#var_name;
                    self
                }
            },
            Fields::Unnamed(f) => {
                // Temporarily name the fields 
                let types = f.unnamed.iter().map(|field| &field.ty);
                let args = (0..f.unnamed.len()).map(|i| format_ident!("arg{}",i)).collect::<Vec<_>>();
                quote! {
                    pub fn #method_name<#(#args: Into<#types>),*>(&mut self, #(#args: #args),*) - &mut Self {
                        self = Self::#var_name(#(#args.into()),*)
                        self
                    }
                }
            },
            Fields::Named(f) => {
                let idents = f.named.iter().map(|field| &field.ident).collect::<Vec<_>>();
                let types = f.named.iter().map(|field| &field.ty);
                quote! {
                    pub fn #method_name<#(#idents:Into<#types>),*>(&mut self, #(#idents: #idents),*) -> &mut Self {
                        self = Self::#var_name {#(#idents: #idents.into()),*}
                    }
                }
            }
        }
    });

    quote! {#(#variants)*}
}

// Method to extract the type inside of Option<T>
fn get_option_inner_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(tp) = ty {
        let last_segment = tp.path.segments.last()?;
        if last_segment.ident == "Option" {
            if let PathArguments::AngleBracketed(args) = &last_segment.arguments
            {
                if let Some(GenericArgument::Type(inner)) = args.args.first() {
                    return Some(inner);
                }
            }
        }
    }
    None
}

// Derive for structs consisting of types that implement TargetFromBuilder
#[proc_macro_derive(BuilderFromTargets)]
pub fn derive_builder_from_targets(input: TokenStream) -> TokenStream {
    // Get information about the structure
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Derive the name of the builder
    let builder_name = format_ident!("{}Builder", name);

    // Restrict the derive to structs with named fields
    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => &f.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    let builder_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;

        // Use the TargetFromBuilder to find the right Builder type
        quote! { pub #name: <#ty as TargetFromBuilder>::Builder }
    });

    // Command to iteratively build the fields
    let build_logic = fields.iter().map(|f| {
        let name = &f.ident;

        // Call .build() to return BuildError
        quote! { #name: self.#name.build()? }
    });

    // Command to iteratively return default builders from target
    let from_target_logic = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! {
            #name: <<#ty as TargetFromBuilder>::Builder as BuilderMethods>::from_target(&target.#name)
        }
    });

    let expanded = quote! {
        #[derive(BuilderSetters, Default, serde::Serialize, serde::Deserialize)]
        pub struct #builder_name {
            #(
                #[setter(nested)]
                #builder_fields,
            )*
        }

        impl BuilderMethods for #builder_name {
            type Target = #name;

            fn build(&mut self) -> Result<Self::Target, BuildError> {
                Ok(Self::Target {
                    #(#build_logic,)*
                })
            }

            fn from_target(target: &Self::Target) -> Self {
                Self {
                    #(#from_target_logic,)*
                }
            }
        }

        impl TargetFromBuilder for #name {
            type Builder = #builder_name;
        }
    };

    TokenStream::from(expanded)
}
