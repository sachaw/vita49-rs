// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{parse2, Ident, Token, Type};

struct CifBasicArgs {
    cif_name: Ident,
    _comma0: Token![,],
    cif_field: Ident,
    _comma1: Token![,],
    cif_field_w_unit: Ident,
    _comma2: Token![,],
    cif_type: Type,
}

impl Parse for CifBasicArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let cif_name = input.parse()?;
        let _comma0: Token![,] = input.parse()?;
        let cif_field = input.parse()?;
        let _comma1: Token![,] = input.parse()?;
        let cif_field_w_unit = input.parse()?;
        let _comma2: Token![,] = input.parse()?;
        let cif_type = input.parse()?;
        Ok(CifBasicArgs {
            cif_name,
            _comma0,
            cif_field,
            _comma1,
            cif_field_w_unit,
            _comma2,
            cif_type,
        })
    }
}

pub fn cif_basic(input: TokenStream) -> TokenStream {
    let CifBasicArgs {
        cif_name,
        cif_field,
        cif_field_w_unit,
        cif_type: friendly_type,
        ..
    } = parse2(input).expect("failed to parse macro input");

    let cif = cif_name;
    let cif_mut = format_ident!("{}_mut", cif);
    let cif_fields = format_ident!("{}_fields", cif);
    let cif_fields_mut = format_ident!("{}_fields_mut", cif);

    let cif_attr_field = format_ident!("{}_attributes", cif_field);
    let cif_attr_field_w_unit = format_ident!("{}_attributes", cif_field_w_unit);

    let set_cif_field_fn = format_ident!("set_{}", cif_field);
    let unset_cif_field_fn = format_ident!("unset_{}", cif_field);
    let enable_cif_fn = format_ident!("set_{}_enabled", cif);
    let disable_cif_fn = format_ident!("unset_{}_enabled", cif);

    // Friendly function names (exposed to user)
    let get_fn = format_ident!("{}", cif_field_w_unit);
    let set_fn = format_ident!("set_{}", cif_field_w_unit);
    let get_attr_fn = format_ident!("{}", cif_attr_field_w_unit);
    let set_attr_fn = format_ident!("set_{}", cif_attr_field_w_unit);
    let set_cif7_field_fn = format_ident!("set_field_attributes_enabled");

    let cif_name_str = cif.to_string();
    let mut cif_name_chars = cif_name_str.chars();
    let cif_type_name = match cif_name_chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + cif_name_chars.as_str(),
    };
    let cif_type_name = format_ident!("{}", cif_type_name);
    let cif_fields_type_name = format_ident!("{}Fields", cif_type_name);

    let get_fn_doc =
        format!("Get the current {cif_field_w_unit}. If `None` is returned, the field is unset.");
    let set_fn_doc = format!(
        "Set the {cif_field_w_unit}. If `None` is passed, the field will be unset.\n\n\
        [`update_packet_size()`](Vrt::update_packet_size()) should be executed after running this method."
    );
    let get_attr_fn_doc = format!(
        "Get the current {cif_attr_field_w_unit} (CIF7 attributes). If `None` is returned, the field is unset."
    );
    let set_attr_fn_doc = format!(
        "Set the {cif_attr_field_w_unit} (CIF7 attributes). If `None` is passed, the field will be unset.\n\n\
        [`update_packet_size()`](Vrt::update_packet_size()) should be executed after running this method."
    );

    if cif == "cif0" {
        quote! {
            #[doc = #get_fn_doc]
            fn #get_fn(&self) -> Option<&#friendly_type> {
                self.#cif_fields().#cif_field.as_ref()
            }
            #[doc = #get_attr_fn_doc]
            #[cfg(feature = "cif7")]
            fn #get_attr_fn(&self) -> &Vec<#friendly_type> {
                self.#cif_fields().#cif_attr_field.as_ref()
            }
            #[doc = #set_fn_doc]
            fn #set_fn(&mut self, #cif_field_w_unit: Option<#friendly_type>) {
                self.#cif_fields_mut().#cif_field = #cif_field_w_unit;
                if self.#cif_fields().#cif_field.is_some() {
                    self.#cif_mut().#set_cif_field_fn();
                } else {
                    self.#cif_mut().#unset_cif_field_fn();
                }
            }
            #[doc = #set_attr_fn_doc]
            #[cfg(feature = "cif7")]
            fn #set_attr_fn(&mut self, #cif_attr_field_w_unit: Option<Vec<#friendly_type>>) {
                if let Some(vec) = #cif_attr_field_w_unit {
                    self.cif0_mut().#set_cif7_field_fn();
                    self.#cif_fields_mut().#cif_attr_field = vec;
                    self.#cif_mut().#set_cif_field_fn();
                } else {
                    self.#cif_fields_mut().#cif_attr_field.clear();
                }
            }
        }
    } else {
        quote! {
            #[doc = #get_fn_doc]
            fn #get_fn(&self) -> Option<&#friendly_type> {
                self.#cif_fields()?
                    .#cif_field
                    .as_ref()
            }
            #[doc = #get_attr_fn_doc]
            #[cfg(feature = "cif7")]
            fn #get_attr_fn(&self) -> Option<&Vec<#friendly_type>> {
                if let Some(cif_fields) = self.#cif_fields() {
                    Some(cif_fields
                        .#cif_attr_field
                        .as_ref())
                } else {
                    None
                }
            }
            #[doc = #set_fn_doc]
            fn #set_fn(&mut self, #cif_field_w_unit: Option<#friendly_type>) {
                // `.is_some()` rather than `if let Some(v)`: the value is moved into
                // the field below, so it must not be consumed by the pattern here
                // (that only worked for `Copy` field types).
                if #cif_field_w_unit.is_some() {
                    if self.#cif().is_none() {
                        self.cif0_mut().#enable_cif_fn();
                        *self.#cif_mut() = Some(#cif_type_name::default())
                    }
                    self.#cif_mut().as_mut().unwrap().#set_cif_field_fn();

                    if self.#cif_fields().is_none() {
                        *self.#cif_fields_mut() = Some(#cif_fields_type_name::default());
                    }
                    self.#cif_fields_mut().as_mut().unwrap().#cif_field = #cif_field_w_unit;
                } else {
                    let mut clear_cif = false;
                    let mut clear_fields = false;
                    if let Some(c) = self.#cif_mut() {
                        c.#unset_cif_field_fn();
                        clear_cif = c.empty();
                    }
                    if let Some(f) = self.#cif_fields_mut() {
                        f.#cif_field = None;
                        #[cfg(feature = "cif7")]
                        f.#cif_attr_field.clear();
                        clear_fields = f.empty();
                    }
                    if clear_cif {
                        *self.#cif_mut() = None;
                        self.cif0_mut().#disable_cif_fn();
                    }
                    if clear_fields {
                        *self.#cif_fields_mut() = None;
                    }
                }
            }
            #[doc = #set_attr_fn_doc]
            #[cfg(feature = "cif7")]
            fn #set_attr_fn(&mut self, #cif_attr_field_w_unit: Option<Vec<#friendly_type>>) {
                if let Some(vec) = #cif_attr_field_w_unit {
                    self.cif0_mut().#set_cif7_field_fn();
                    if self.#cif().is_none() {
                        self.cif0_mut().#enable_cif_fn();
                        *self.#cif_mut() = Some(#cif_type_name::default())
                    }
                    self.#cif_mut().as_mut().unwrap().#set_cif_field_fn();

                    if self.#cif_fields().is_none() {
                        *self.#cif_fields_mut() = Some(#cif_fields_type_name::default());
                    }
                    self.#cif_fields_mut().as_mut().unwrap().#cif_attr_field = vec;
                } else {
                    if let Some(f) = self.#cif_fields_mut() {
                        f.#cif_attr_field.clear();
                    }
                }
            }
        }
    }
}
