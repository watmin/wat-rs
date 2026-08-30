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

/// The `@Total` value -> token match (arc 255 Stone total-T2), exhaustive, no wildcard —
/// mirrors `purity_token`/`determinism_token`/`category_token` in [`emit`], and is the
/// same shape as `wat_intrinsic::totality_token`. A standalone `pub(crate) fn` so it is
/// directly unit-testable. See the call site in [`emit`] for why its result is not yet
/// spliced into `SpecialFormSubmission`.
pub(crate) fn totality_token(t: wat_doc::Totality) -> TokenStream2 {
    match t {
        wat_doc::Totality::Total => quote! { ::wat_doc::Totality::Total },
        wat_doc::Totality::Partial => quote! { ::wat_doc::Totality::Partial },
        wat_doc::Totality::Preserving => quote! { ::wat_doc::Totality::Preserving },
        wat_doc::Totality::Unreviewed => quote! { ::wat_doc::Totality::Unreviewed },
    }
}

/// The `@ExpandTime` value -> token match (arc 255 Stone expand-T2), exhaustive, no
/// wildcard — the special-form twin of `wat_intrinsic::expand_time_token`. Unlike
/// `totality_token` just above, its result IS spliced into `SpecialFormSubmission`
/// in THIS stone (expand-T2's blast radius includes the entry from the start).
pub(crate) fn expand_time_token(t: wat_doc::ExpandTime) -> TokenStream2 {
    match t {
        wat_doc::ExpandTime::Legal => quote! { ::wat_doc::ExpandTime::Legal },
        wat_doc::ExpandTime::RuntimeOnly => quote! { ::wat_doc::ExpandTime::RuntimeOnly },
        wat_doc::ExpandTime::Preserving => quote! { ::wat_doc::ExpandTime::Preserving },
        wat_doc::ExpandTime::Unreviewed => quote! { ::wat_doc::ExpandTime::Unreviewed },
    }
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
    // arc 255 Stone total-T2 — see the identical comment + `totality_token` fn in
    // `wat_intrinsic.rs`: computed for every real special form, NOT spliced into
    // `SpecialFormSubmission` below (that struct, `src/intrinsic/mod.rs`, has no
    // `totality` field; adding one is a `src/` edit this stone's blast radius forbids).
    let totality_token = totality_token(doc.totality);
    // arc 255 Stone expand-T2 — computed for every real special form AND spliced into
    // `SpecialFormSubmission` below: this stone's blast radius includes `src/intrinsic/mod.rs`
    // from the start.
    let expand_time_token = expand_time_token(doc.expand_time);
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
        wat_doc::Category::Projection => quote! { ::wat_doc::Category::Projection },
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
                totality: #totality_token,
                expand_time: #expand_time_token,
                category: #category_token,
                deprecated: #deprecated_lit,
            }
        }
    };

    Ok(expanded)
}

#[cfg(test)]
mod totality_axis_tests {
    //! arc 255 Stones total-T2 / total-T3 — the special-form sibling of
    //! `wat_intrinsic`'s `totality_axis_tests`. T2's Row 4 proved `@Total` reaches
    //! `totality_token` (the same fn [`emit`] calls for every real special form).
    //! T3 adds this module's own Row 1: absence of `@Total` must make [`emit`]
    //! refuse to expand, with `MissingTotality`.
    use super::*;

    fn fixture_item(total_line: &str) -> syn::ItemStruct {
        let src = format!(
            "/// A probe form.\n\
             ///\n\
             /// @added 1.0.0\n\
             /// @Purity Pure\n\
             /// @Determinism Deterministic\n\
             {total_line}\
             /// @Category ControlFlow\n\
             /// @syntax (probe-form a b)\n\
             /// @ret :wat::core::i64 the ret\n\
             /// @example (probe-form 1 2) #=> 3\n\
             struct ProbeForm;"
        );
        syn::parse_str(&src).expect("fixture special-form struct must be syntactically valid Rust")
    }

    fn fqdn() -> LitStr {
        LitStr::new(":probe::probe-form", proc_macro2::Span::call_site())
    }

    /// ★ ROW 4 — a fixture special form declaring `@Total Partial` reads back
    /// `Partial` (not `Unreviewed`) through the SAME `totality_token` [`emit`] calls,
    /// and [`emit`] itself accepts the fixture end-to-end.
    #[test]
    fn row4_total_partial_survives_into_the_generated_token() {
        let item = fixture_item("/// @Total Partial\n");
        let raw_doc = sniff_doc_from_struct(&item).expect("fixture doc must be sniffed");
        let doc = wat_doc::parse_special_form(&raw_doc).expect("fixture doc must parse under the full contract");
        assert_eq!(doc.totality, wat_doc::Totality::Partial, "declared @Total Partial must read back as Partial");

        let token = totality_token(doc.totality);
        assert_eq!(token.to_string(), quote! { ::wat_doc::Totality::Partial }.to_string());
        assert_ne!(token.to_string(), quote! { ::wat_doc::Totality::Unreviewed }.to_string());

        emit(&fqdn(), &item).expect("emit() must accept a fixture declaring @Total Partial");
    }

    /// ★ Arc 255 Stone total-T3, ROW 1 (special-form sibling) — a fixture with NO
    /// `@Total` line must FAIL TO COMPILE with `MissingTotality`, through the same
    /// `emit()` every real `#[wat_special_form]` call site expands through.
    #[test]
    fn absent_total_fails_to_compile_with_missing_totality() {
        let item = fixture_item("");
        let raw_doc = sniff_doc_from_struct(&item).expect("fixture doc must be sniffed");
        assert_eq!(
            wat_doc::parse_special_form(&raw_doc),
            Err(wat_doc::DocError::MissingTotality),
            "a doc block with no @Total must fail to parse with MissingTotality"
        );

        let err = emit(&fqdn(), &item)
            .expect_err("emit() must refuse to expand a fixture with no @Total");
        // Exact, not `contains` — this file's error path is `format!("{:?}", e)`, so the
        // whole rendering IS the variant name and an exact compare costs nothing while
        // catching drift. (tests/lint/no_loose_string_assert.rs went RED on the
        // `contains` form this replaces.)
        assert_eq!(
            err.to_string(),
            "#[wat_special_form] :probe::probe-form: MissingTotality",
            "emit()'s refusal must name the form and MissingTotality"
        );
    }

    /// `totality_token` is exhaustive, no wildcard.
    #[test]
    fn totality_token_matches_every_variant() {
        let cases = [
            (wat_doc::Totality::Total, quote! { ::wat_doc::Totality::Total }),
            (wat_doc::Totality::Partial, quote! { ::wat_doc::Totality::Partial }),
            (wat_doc::Totality::Preserving, quote! { ::wat_doc::Totality::Preserving }),
            (wat_doc::Totality::Unreviewed, quote! { ::wat_doc::Totality::Unreviewed }),
        ];
        for (variant, expected) in cases {
            assert_eq!(totality_token(variant).to_string(), expected.to_string());
        }
    }
}

#[cfg(test)]
mod expand_time_axis_tests {
    //! arc 255 Stone expand-T2 — the special-form sibling of `wat_intrinsic`'s
    //! `expand_time_axis_tests`. `@ExpandTime` is OPTIONAL here too; absence
    //! DEFAULTS to `Unreviewed` through the SAME `emit()` every real
    //! `#[wat_special_form]` call site expands through.
    use super::*;

    fn fixture_item(expand_time_line: &str) -> syn::ItemStruct {
        let src = format!(
            "/// A probe form.\n\
             ///\n\
             /// @added 1.0.0\n\
             /// @Purity Pure\n\
             /// @Determinism Deterministic\n\
             /// @Total Unreviewed\n\
             {expand_time_line}\
             /// @Category ControlFlow\n\
             /// @syntax (probe-form a b)\n\
             /// @ret :wat::core::i64 the ret\n\
             /// @example (probe-form 1 2) #=> 3\n\
             struct ProbeForm;"
        );
        syn::parse_str(&src).expect("fixture special-form struct must be syntactically valid Rust")
    }

    fn fqdn() -> LitStr {
        LitStr::new(":probe::probe-form", proc_macro2::Span::call_site())
    }

    /// ★ ROW 3/4 — a fixture special form declaring `@ExpandTime Legal` reads back
    /// `Legal` (not `Unreviewed`) through the SAME `expand_time_token` [`emit`]
    /// calls, and [`emit`] itself accepts the fixture end-to-end.
    #[test]
    fn expand_time_legal_survives_into_the_generated_token() {
        let item = fixture_item("/// @ExpandTime Legal\n");
        let raw_doc = sniff_doc_from_struct(&item).expect("fixture doc must be sniffed");
        let doc = wat_doc::parse_special_form(&raw_doc).expect("fixture doc must parse under the full contract");
        assert_eq!(doc.expand_time, wat_doc::ExpandTime::Legal, "declared @ExpandTime Legal must read back as Legal");

        let token = expand_time_token(doc.expand_time);
        assert_eq!(token.to_string(), quote! { ::wat_doc::ExpandTime::Legal }.to_string());
        assert_ne!(token.to_string(), quote! { ::wat_doc::ExpandTime::Unreviewed }.to_string());

        emit(&fqdn(), &item).expect("emit() must accept a fixture declaring @ExpandTime Legal");
    }

    /// ★ Row 1's default-when-absent, through the FULL `emit()` pipeline — a
    /// fixture with NO `@ExpandTime` line must still compile (optional in T2) and
    /// its generated token must be `Unreviewed`.
    #[test]
    fn absent_expand_time_defaults_to_unreviewed_in_the_generated_token() {
        let item = fixture_item("");
        let raw_doc = sniff_doc_from_struct(&item).expect("fixture doc must be sniffed");
        let doc = wat_doc::parse_special_form(&raw_doc).expect("absent @ExpandTime must still parse — optional in T2");
        assert_eq!(doc.expand_time, wat_doc::ExpandTime::Unreviewed);

        let token = expand_time_token(doc.expand_time);
        assert_eq!(token.to_string(), quote! { ::wat_doc::ExpandTime::Unreviewed }.to_string());

        emit(&fqdn(), &item).expect("emit() must accept a fixture declaring no @ExpandTime");
    }

    /// `expand_time_token` is exhaustive, no wildcard.
    #[test]
    fn expand_time_token_matches_every_variant() {
        let cases = [
            (wat_doc::ExpandTime::Legal, quote! { ::wat_doc::ExpandTime::Legal }),
            (wat_doc::ExpandTime::RuntimeOnly, quote! { ::wat_doc::ExpandTime::RuntimeOnly }),
            (wat_doc::ExpandTime::Preserving, quote! { ::wat_doc::ExpandTime::Preserving }),
            (wat_doc::ExpandTime::Unreviewed, quote! { ::wat_doc::ExpandTime::Unreviewed }),
        ];
        for (variant, expected) in cases {
            assert_eq!(expand_time_token(variant).to_string(), expected.to_string());
        }
    }
}
