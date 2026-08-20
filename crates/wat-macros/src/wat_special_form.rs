//! Codegen for `#[wat_special_form("<fqdn>")]` — arc 255.SF.
//!
//! Annotates a unit struct; the `///` doc block is parsed via
//! `wat_doc::parse_special_form` (which requires `@arg` OR `@syntax` — at
//! least one expressing the shape — plus `@Purity`, `@Determinism`,
//! `@Category`, `@added`, `@ret`, and ≥1 `@example`).
//! Emits an `inventory::submit!` of a `SpecialFormSubmission` — no
//! `NativeHandler`, no dispatch shim. The entry lands in the
//! `IntrinsicRegistry` as `Kind::SpecialForm` and is visible to
//! `lookup_entry` / `all_entries` (and therefore `render-doc`).

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Error, Expr, ExprLit, Lit, LitStr, Meta};

// @Category variants are now validated by wat_doc::parse_special_form via Category::from_str.

/// Sniff the struct's doc comment — same strategy as `wat_intrinsic::sniff_doc`
/// but for `ItemStruct` rather than `ItemFn`.
fn sniff_doc_from_struct(item: &syn::ItemStruct) -> Option<String> {
    let lines: Vec<String> = item
        .attrs
        .iter()
        .filter_map(|attr| {
            if let Meta::NameValue(nv) = &attr.meta {
                if nv.path.is_ident("doc") {
                    if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = &nv.value {
                        let raw = s.value();
                        return Some(raw.strip_prefix(' ').map(str::to_owned).unwrap_or(raw));
                    }
                }
            }
            None
        })
        .collect();
    if lines.is_empty() { None } else { Some(lines.join("\n")) }
}

pub(crate) fn emit(fqdn: &LitStr, item: &syn::ItemStruct) -> syn::Result<TokenStream2> {
    let raw_doc = match sniff_doc_from_struct(item) {
        Some(d) => d,
        None => {
            return Err(Error::new_spanned(
                item,
                format!(
                    "#[wat_special_form] {}: missing doc comment (/// is required; \
                     must include @arg or @syntax, @Purity, @Determinism, @Category, @added, @ret, and ≥1 @example)",
                    fqdn.value()
                ),
            ));
        }
    };

    let doc = match wat_doc::parse_special_form(&raw_doc) {
        Ok(d) => d,
        Err(e) => {
            return Err(Error::new_spanned(
                item,
                format!("#[wat_special_form] {}: {:?}", fqdn.value(), e),
            ));
        }
    };

    let prose_lit = &doc.prose;
    let added_lit = &doc.added;
    let syntax_lit = &doc.syntax;
    let ret_type_lit = &doc.ret_type;
    let ret_lit = &doc.ret;
    let purity_token = match doc.purity {
        wat_doc::Purity::Pure => quote! { ::wat_doc::Purity::Pure },
        wat_doc::Purity::Effectful => quote! { ::wat_doc::Purity::Effectful },
        wat_doc::Purity::Preserving => quote! { ::wat_doc::Purity::Preserving },
    };
    let determinism_token = match doc.determinism {
        wat_doc::Determinism::Deterministic => quote! { ::wat_doc::Determinism::Deterministic },
        wat_doc::Determinism::Nondeterministic => quote! { ::wat_doc::Determinism::Nondeterministic },
        wat_doc::Determinism::Preserving => quote! { ::wat_doc::Determinism::Preserving },
    };
    let category_token = match doc.category {
        wat_doc::Category::Transform => quote! { ::wat_doc::Category::Transform },
        wat_doc::Category::Reflection => quote! { ::wat_doc::Category::Reflection },
        wat_doc::Category::ControlFlow => quote! { ::wat_doc::Category::ControlFlow },
        wat_doc::Category::Binding => quote! { ::wat_doc::Category::Binding },
        wat_doc::Category::Entropic => quote! { ::wat_doc::Category::Entropic },
        wat_doc::Category::Arithmetic => quote! { ::wat_doc::Category::Arithmetic },
        wat_doc::Category::Io => quote! { ::wat_doc::Category::Io },
        wat_doc::Category::Probe => quote! { ::wat_doc::Category::Probe },
        wat_doc::Category::Combine => quote! { ::wat_doc::Category::Combine },
        wat_doc::Category::Declaration => quote! { ::wat_doc::Category::Declaration },
        wat_doc::Category::Resource => quote! { ::wat_doc::Category::Resource },
        wat_doc::Category::Message => quote! { ::wat_doc::Category::Message },
        wat_doc::Category::Ambient => quote! { ::wat_doc::Category::Ambient },
        wat_doc::Category::Project => quote! { ::wat_doc::Category::Project },
        wat_doc::Category::CheckGate => quote! { ::wat_doc::Category::CheckGate },
    };

    let args_lit: Vec<TokenStream2> = doc.args.iter().map(|a| {
        let name = &a.name;
        let ty = &a.ty;
        let desc = &a.desc;
        let is_rest = a.is_rest;
        quote! { (#name, #ty, #desc, #is_rest) }
    }).collect();

    let examples_lit: Vec<TokenStream2> = doc.examples.iter().map(|ex| {
        let expr = &ex.expr;
        let run = ex.run;
        let expected = match &ex.expected {
            Some(s) => quote! { ::std::option::Option::Some(#s) },
            None => quote! { ::std::option::Option::None },
        };
        quote! {
            ::wat::intrinsic::ExampleSubmission {
                expr: #expr,
                expected: #expected,
                run: #run,
            }
        }
    }).collect();

    let see_lit: Vec<&str> = doc.see.iter().map(String::as_str).collect();

    let deprecated_lit = match &doc.deprecated {
        Some(d) => {
            let since = &d.since;
            let use_instead = &d.use_instead;
            quote! { ::std::option::Option::Some((#since, #use_instead)) }
        }
        None => quote! { ::std::option::Option::None },
    };

    let expanded = quote! {
        // The annotated struct. A zero-cost attribute anchor — never constructed
        // (it exists only to carry the doc + this attribute). `allow(dead_code)`
        // keeps every special-form marker struct warning-clean by construction.
        #[allow(dead_code)]
        #item

        // Auto-collect: link-time registration of this special form in the
        // IntrinsicRegistry as Kind::SpecialForm (no handler).
        ::inventory::submit! {
            ::wat::intrinsic::SpecialFormSubmission {
                name: #fqdn,
                prose: #prose_lit,
                added: #added_lit,
                syntax: #syntax_lit,
                args: &[#(#args_lit),*],
                ret_type: #ret_type_lit,
                ret: #ret_lit,
                examples: &[#(#examples_lit),*],
                see: &[#(#see_lit),*],
                purity: #purity_token,
                determinism: #determinism_token,
                category: #category_token,
                deprecated: #deprecated_lit,
            }
        }
    };

    Ok(expanded)
}
