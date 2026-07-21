// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, Ident, ItemStruct};

static PRIMITIVES: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32",
    "f64", "char", "bool",
];

/// Field types whose wire size depends on the packet prologue TSI/TSF and so take
/// a `prologue: PrologueCtx` context argument. Such a field gets `ctx = "prologue"`
/// forwarded from the enclosing `Cif*Fields` ctx; every other field type is
/// prologue-independent and gets no extra ctx.
static PROLOGUE_TYPES: &[&str] = &["StateTime"];

pub fn cif_fields(attr: TokenStream, item: TokenStream) -> TokenStream {
    let cif_name = parse_macro_input!(attr as Ident);
    let input = parse_macro_input!(item as ItemStruct);
    let struct_name = input.ident;
    let mut expanded_fields = Vec::new();
    let mut expanded_size_lines = Vec::new();
    let mut empty_check_lines = Vec::new();

    for field in input.fields {
        let cif_field = field.clone().ident.unwrap();
        let cif_type = field.clone().ty;

        let field_doc = format!("{cif_field} data field");
        let attr_doc = format!("{cif_field} data attributes field (only used if CIF7 is enabled)");

        let attr_field = format_ident!("{}_attributes", cif_field);
        // CIF fields other than cif0 are optional, so we have to add an unwrap()
        let (main_cond, attr_cond) =
            if cif_name == "cif0" && !format!("{struct_name}").contains("Ack") {
                (
                    format!("{cif_name}.{cif_field}() && cif7_opts.current_val"),
                    format!("{cif_name}.{cif_field}() && cif7_opts.num_extra_attrs > 0"),
                )
            } else {
                (
                    format!("{cif_name}.unwrap().{cif_field}() && cif7_opts.current_val"),
                    format!("{cif_name}.unwrap().{cif_field}() && cif7_opts.num_extra_attrs > 0"),
                )
            };

        let cif_type_string = cif_type.to_token_stream().to_string();

        // Prologue-sized field types need the prologue TSI/TSF forwarded as ctx.
        let (main_deku, attr_deku) = if PROLOGUE_TYPES.contains(&cif_type_string.as_str()) {
            (
                quote! { #[deku(cond = #main_cond, ctx = "prologue")] },
                quote! { #[deku(cond = #attr_cond, count = "cif7_opts.num_extra_attrs", ctx = "prologue")] },
            )
        } else {
            (
                quote! { #[deku(cond = #main_cond)] },
                quote! { #[deku(cond = #attr_cond, count = "cif7_opts.num_extra_attrs")] },
            )
        };

        let expanded = quote! {
            #[doc = #field_doc]
            #main_deku
            pub #cif_field: Option<#cif_type>,

            #[doc = #attr_doc]
            #[cfg(feature = "cif7")]
            #attr_deku
            pub #attr_field: Vec<#cif_type>,
        };
        expanded_fields.push(expanded);

        let expanded = if PRIMITIVES.contains(&cif_type_string.as_str()) {
            quote! {
                if let Some(v) = &self.#cif_field {
                    acc += (std::mem::size_of_val(v) / std::mem::size_of::<u32>()) as u16;
                }
                #[cfg(feature = "cif7")]
                if let Some(v) = self.#attr_field.first() {
                    acc += ((std::mem::size_of_val(v) * self.#attr_field.len()) / std::mem::size_of::<u32>()) as u16;
                }
            }
        } else {
            quote! {
                if let Some(v) = &self.#cif_field {
                    acc += v.size_words();
                }
                #[cfg(feature = "cif7")]
                if let Some(v) = self.#attr_field.first() {
                    acc += v.size_words() * (self.#attr_field.len() as u16);
                }
            }
        };

        expanded_size_lines.push(expanded);

        let expanded = quote! {
            #[cfg(feature = "cif7")]
            if self.#cif_field.is_some() || ! self.#attr_field.is_empty() {
                return false;
            }
            #[cfg(not(feature = "cif7"))]
            if self.#cif_field.is_some() {
                return false;
            }
        };
        empty_check_lines.push(expanded);
    }

    let cif_name_str = cif_name.to_string();
    let mut cif_name_chars = cif_name_str.chars();
    let mut cif_type_name = match cif_name_chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + cif_name_chars.as_str(),
    };
    cif_type_name = format!("&{cif_type_name}");
    if cif_name != "cif0" || format!("{struct_name}").contains("Ack") {
        cif_type_name = format!("Option<{cif_type_name}>");
    }
    // `prologue` carries the packet prologue's TSI/TSF so fields whose wire size
    // follows the packet timestamp (Age / Shelf Life, Sector Start-Time) can size
    // themselves; fixed-size fields ignore it. Fully-qualified so no per-module use.
    let deku_ctx = format!(
        "endian: deku::ctx::Endian, {cif_name}: {cif_type_name}, cif7_opts: Cif7Opts, prologue: crate::packet_header::PrologueCtx"
    );
    let struct_doc = format!("Structure for all {cif_name} data fields (not indicators)");
    let size_doc = format!("Gets the size of all {cif_name} data fields in 32-bit words");
    let empty_doc = format!("Returns true if all {cif_name} data fields are empty, false if not");

    let expanded = quote! {
        #[doc = #struct_doc]
        #[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DekuRead, DekuWrite)]
        #[deku(
            endian = "endian",
            ctx = #deku_ctx,
        )]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub struct #struct_name {
            #(#expanded_fields)*
        }

        impl #struct_name {
            #[doc = #size_doc]
            pub fn size_words(&self) -> u16 {
                let mut acc = 0;
                #(#expanded_size_lines)*
                acc
            }

            #[doc = #empty_doc]
            pub fn empty(&self) -> bool {
                #(#empty_check_lines)*
                true
            }
        }
    };

    TokenStream::from(expanded)
}
