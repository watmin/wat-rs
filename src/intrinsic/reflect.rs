//! `:wat::intrinsic::examples` — the iv-b2-a reflection seam (arc 255.1b).
//!
//! Walks the intrinsic registry and returns every registered intrinsic's
//! carried `@example`s to wat as data, so a wat verifier can run them.
//! Mirrors `:wat::stdlib::sources` (io.rs:1454): return plain Vectors, let
//! wat wrap. The wat verifier (`verify-examples`, iv-b2-b) `eval-ast!`s them.
//!
//! Record shape per element: a `:wat::intrinsic::Example` `Value::wat__core__Record`
//! with fields `[fqdn, expr, expected, run, pure, deterministic]` (declaration
//! order) — `fqdn` a keyword, `expr` a quoted `Value::wat__WatAST`, `expected`
//! a `Value::Option<Value::wat__WatAST>` (None for markerless/`@example-norun`),
//! `run`/`pure`/`det` bools.
//!
//! This read satisfies iv-b1's `#[expect(dead_code)]` on
//! `IntrinsicEntry.examples` and `ExampleSubmission` — both removed in
//! `src/intrinsic/mod.rs` this strike.
//!
//! Arc 255.1b-v adds `show-source` and `render-doc` — the reflection surface
//! over the intrinsic registry, proven on the `core::Bytes` pilot.

use std::sync::Arc;

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::parser::parse_one_with_file;
use crate::span::Span;
use crate::value::{EvalBreak, Environment, RuntimeError, RuntimeErrorKind, SymbolTable, Value};
use crate::value::value::AggregateValue;

::wat_source_derive::wat_field_names_from!(EXAMPLE_FIELDS, "wat/doctest.wat", ":wat::intrinsic::Example");
fn example_names() -> Arc<Vec<String>> {
    static N: std::sync::OnceLock<Arc<Vec<String>>> = std::sync::OnceLock::new();
    N.get_or_init(|| crate::value::value::names_arc_from_static(EXAMPLE_FIELDS)).clone()
}

/// Walk the intrinsic registry and return every registered intrinsic's
/// carried `@example`s as a `Vector` of `:wat::intrinsic::Example` records —
/// the iv-b2-a reflection seam. The wat verifier (`verify-examples`, iv-b2-b)
/// iterates this to run or skip each example.
///
/// Each element is a `:wat::intrinsic::Example` `Value::wat__core__Record` with fields
/// (declaration order):
/// - `fqdn`: the intrinsic's FQDN as a keyword
/// - `expr`: the example expression, parsed into a quoted form (`Value::wat__WatAST`)
/// - `expected`: `Value::Option` — `Some(WatAST)` when `#=>` is present + runnable, else `None`
/// - `run`: bool — true for `@example`, false for `@example-norun`
/// - `pure`: bool — read off the entry's declared `purity` (arc 255.1c site 2;
///   `Pure`/`Preserving` both count, since a registered row's declared purity
///   is a fact about its body, not a namespace guess)
/// - `deterministic`: bool — `pure` ∧ declared `determinism` is
///   `Deterministic`/`Preserving`
///
/// A parse failure of an example string is a loud seam error (acceptable —
/// a malformed example is a real defect; the macro enforced the doc SHAPE,
/// not that `expr` parses as wat).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Reflection
/// @ret :wat::core::Vector<wat::intrinsic::Example> a Vector of Example records, one per @example/@example-norun across all registered intrinsics
/// @example-norun (:wat::intrinsic::examples)
#[wat_intrinsic(":wat::intrinsic::examples")]
pub(crate) fn eval_intrinsic_examples(
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    // Suppress unused-param warnings for the context tail — zero wat args,
    // but the macro requires the tail and the NativeHandler ABI needs them.
    let _ = (env, sym);

    let mut tuples: Vec<Value> = Vec::new();

    for entry in crate::intrinsic::registry().all_entries() {
        let fqdn_kw = Value::wat__core__keyword(Arc::new(entry.name.to_string()));
        // Arc 255.1c site 2 — read the entry we already hold instead of a prefix
        // guess: `entry.purity`/`entry.determinism` are declared fields in the
        // same struct, not re-derived from the FQDN.
        let pure = matches!(entry.purity, wat_doc::Purity::Pure | wat_doc::Purity::Preserving);
        let det = pure
            && matches!(
                entry.determinism,
                wat_doc::Determinism::Deterministic | wat_doc::Determinism::Preserving
            );

        for ex in entry.examples {
            // Parse `expr` into a quoted form — loud on failure.
            let expr_ast = parse_one_with_file(ex.expr, "<intrinsic-example>")
                .map_err(|e| RuntimeError::new(span.clone(), RuntimeErrorKind::MalformedForm {
                        head: ":wat::intrinsic::examples".into(),
                        reason: format!(
                            "intrinsic {} @example expr failed to parse: {:?}",
                            entry.name, e
                        ),
                    }))?;
            let expr_q = Value::wat__WatAST(Arc::new(expr_ast));

            // Parse `expected` only when this is a runnable example (`run=true`).
            // For `@example-norun`, the `#=>` text may be pseudo-code (human
            // doc only, not wat syntax) — the verifier skips it, so yield None.
            // None is also yielded for a markerless `@example-norun` (None).
            // Field type is Option<:wat::WatAST> → Value::Option.
            let expected_field = if ex.run {
                match ex.expected {
                    Some(s) => {
                        let expected_ast =
                            parse_one_with_file(s, "<intrinsic-example-expected>").map_err(
                                |e| RuntimeError::new(span.clone(), RuntimeErrorKind::MalformedForm {
                                        head: ":wat::intrinsic::examples".into(),
                                        reason: format!(
                                            "intrinsic {} @example expected failed to parse: {:?}",
                                            entry.name, e
                                        ),
                                    }),
                            )?;
                        Value::Option(Arc::new(Some(Value::wat__WatAST(Arc::new(expected_ast)))))
                    }
                    None => Value::Option(Arc::new(None)),
                }
            } else {
                // @example-norun: expected is human-doc pseudo-code, not wat; yield None.
                Value::Option(Arc::new(None))
            };

            // Builder doctrine (2026-06-21): EDN-representable data → `Value::wat__core__Record`
            // (the `:wat::core::defrecord` representation, so the generated named accessors
            // `:wat::intrinsic::Example/<field>` work); `Value::Struct` is reserved for
            // payloads that are NOT EDN-able. An Example is fully EDN-representable
            // (keyword + WatASTs + bools), so it's a wat__core__Record. `class_fqdn` carries
            // NO leading colon (matches the RecordDef class identity).
            let record = Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::intrinsic::Example".to_string(),
                example_names(),
                Arc::new(vec![
                    fqdn_kw.clone(),
                    expr_q,
                    expected_field,
                    Value::bool(ex.run),
                    Value::bool(pure),
                    Value::bool(det),
                ]),
            )));
            tuples.push(record);
        }
    }

    Ok(Value::Vec(Arc::new(tuples)))
}

// ─── Arc 255.1b-v: @see registry cross-check ─────────────────────────────────

/// Walk the intrinsic registry and collect dangling `@see` references —
/// FQDNs listed in an entry's `@see` doc that do NOT resolve to any registered
/// intrinsic. An empty result means the corpus is internally consistent.
///
/// Lives in `#[cfg(test)]` — only consumed by the @see cross-check test in
/// `intrinsic/mod.rs`. The `see` field is also read by `eval_render_doc`'s
/// "See also:" section (non-test code), so there is no dead-code issue.
#[cfg(test)]
pub(crate) fn check_see_refs() -> Vec<String> {
    let reg = crate::intrinsic::registry();
    let mut dangling: Vec<String> = Vec::new();
    for entry in reg.all_entries() {
        for &see_fqdn in entry.see {
            if reg.lookup_entry(see_fqdn).is_none() {
                dangling.push(format!(
                    "dangling @see `{}` on `{}`",
                    see_fqdn, entry.name
                ));
            }
        }
    }
    dangling
}

// ─── Arc 255.1b-v: show-source + render-doc ──────────────────────────────────

/// Extract the FQDN string from a handler arg: if it's a keyword literal, use
/// its string directly (avoids runtime resolution that may lose the name). If
/// it evaluates to a keyword value, use that. Otherwise return a TypeMismatch.
fn extract_fqdn(
    op: &'static str,
    arg: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<String, EvalBreak> {
    match arg {
        WatAST::Keyword(k, _) => Ok(k.clone()),
        _ => {
            let v = crate::runtime::eval_inner(arg, env, sym)?.value_owned();
            match &v {
                Value::wat__core__keyword(k) => Ok((**k).clone()),
                other => Err(RuntimeError::new(arg.span().clone(), RuntimeErrorKind::TypeMismatch {
                        op: op.into(),
                        expected: ":wat::core::keyword (an FQDN like :wat::core::Bytes::to-hex)",
                        got: Box::new(crate::runtime::ValueSnapshot::of(other)),
                    })
                .into()),
            }
        }
    }
}

/// Return the Rust handler source of a registered intrinsic (or the wat source
/// of a user-defined function).
///
/// For intrinsics: returns the `source` field on the `IntrinsicEntry` — the
/// handler's token-restringified source, captured at compile time by the
/// `#[wat_intrinsic]` macro via `quote!(handler_fn).to_string()`. Faithful-
/// if-reformatted; comments are lost (token restringify), structural source
/// is preserved. This mirrors Pry's `show-source` on a Ruby or C extension:
/// both kinds expose their source uniformly.
///
/// For user forms (defn/defmacro): returns the form's body serialized via
/// `write-forms` (the WAT → EDN write path). Returns the FQDN keyword form
/// string for primitives and special forms (no source available).
///
/// Returns a `:wat::core::String`; the caller prints it.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Reflection
/// @arg fqdn :wat::core::keyword the FQDN keyword of the intrinsic or user form to inspect, e.g. `:wat::core::Bytes::to-hex`
/// @ret :wat::core::String the handler's Rust source (for intrinsics) or the body's wat source (for user forms)
/// @example-norun (:wat::core::show-source :wat::core::Bytes::to-hex) #=> "pub (crate) fn eval_bytes_to_hex ..."
#[wat_intrinsic(":wat::core::show-source")]
pub(crate) fn eval_show_source(
    fqdn: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::show-source";
    let _ = span;
    let name = extract_fqdn(OP, fqdn, env, sym)?;

    // Intrinsic path: registry entry carries the captured Rust source.
    if let Some(entry) = crate::intrinsic::registry().lookup_entry(&name) {
        return Ok(Value::String(Arc::new(entry.source.to_string())));
    }

    // User-form path: look up via the symbol table and write-forms the body.
    match crate::runtime::lookup_form(&name, sym) {
        Some(crate::runtime::Binding::UserFunction { f, .. }) => {
            match &f.body {
                crate::value::FunctionBody::Wat(ast) => {
                    let edn = crate::wat_edn_bridge::watast_to_edn(ast.as_ref());
                    let text = wat_edn::write(&edn);
                    Ok(Value::String(Arc::new(text)))
                }
                crate::value::FunctionBody::Native => {
                    // Native builtin with no wat body — return the fqdn as a hint.
                    Ok(Value::String(Arc::new(format!(
                        ";; {} — native Rust builtin (no wat source available)",
                        name
                    ))))
                }
            }
        }
        Some(crate::runtime::Binding::Macro { def, .. }) => {
            let edn = crate::wat_edn_bridge::watast_to_edn(&def.body);
            let text = wat_edn::write(&edn);
            Ok(Value::String(Arc::new(text)))
        }
        Some(crate::runtime::Binding::Primitive { .. })
        | Some(crate::runtime::Binding::SpecialForm { .. })
        | Some(crate::runtime::Binding::Type { .. }) => {
            Ok(Value::String(Arc::new(format!(
                ";; {} — substrate primitive (no source available in this context)",
                name
            ))))
        }
        None => Err(RuntimeError::new(fqdn.span().clone(), RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!("no intrinsic or user form found for FQDN `{}`", name),
            })
        .into()),
    }
}

/// Render the registry metadata of an intrinsic as a human-readable plain-text
/// string (with `\n` newlines). The caller prints it: `(println (render-doc :wat::core::Bytes::to-hex))`.
///
/// Format (plain-text, stable):
/// ```text
/// :wat::core::Bytes::to-hex
///
/// <prose>
///
/// Examples:
///   (<expr>)          ; or:  (<expr>) #=> <expected>
/// ```
///
/// Pure and deterministic → returns a `:wat::core::String`; assertable in tests.
/// Plain-text only (no ANSI/glow/markdown rendering — flavor is the caller's
/// choice; a renderer drops in later over the SAME `metadata-of` data).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Reflection
/// @arg fqdn :wat::core::keyword the FQDN keyword of the registered intrinsic to render, e.g. `:wat::core::Bytes::to-hex`
/// @ret :wat::core::String a plain-text multi-line String rendering the intrinsic's name, prose, and examples
/// @example-norun (:wat::core::render-doc :wat::core::Bytes::to-hex) #=> ":wat::core::Bytes::to-hex\n\n..."
#[wat_intrinsic(":wat::core::render-doc")]
pub(crate) fn eval_render_doc(
    fqdn: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::render-doc";
    let _ = span;
    let _ = (env, sym);
    let name = extract_fqdn(OP, fqdn, env, sym)?;

    let entry = match crate::intrinsic::registry().lookup_entry(&name) {
        Some(e) => e,
        None => {
            return Err(RuntimeError::new(fqdn.span().clone(), RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!("no registered intrinsic found for FQDN `{}`", name),
                })
            .into());
        }
    };

    // Plain-text block: name line, blank, prose, blank, Examples section.
    let mut out = String::new();

    // Name / signature line.
    out.push_str(entry.name);
    out.push('\n');

    // Blank line separator.
    out.push('\n');

    // Prose (the GFM body, which is plain text here).
    out.push_str(entry.prose.trim());
    out.push('\n');

    // Syntax line (special forms only).
    // When entry.syntax is non-empty, render it verbatim (@syntax was provided).
    // When empty but @arg entries are present, derive: `(head <arg>…)` where
    // head is the last `::` segment of the FQDN (e.g. `:wat::core::if` → `if`).
    {
        let syntax_str: Option<String> = if !entry.syntax.is_empty() {
            Some(entry.syntax.to_string())
        } else if !entry.args.is_empty() {
            // Derive the grammar from the head short-name + arg names.
            let head = entry.name.rsplit("::").next().unwrap_or(entry.name);
            let slots: Vec<String> = entry.args.iter()
                .map(|&(name, _, _, _)| format!("<{}>", name))
                .collect();
            Some(format!("({} {})", head, slots.join(" ")))
        } else {
            None
        };
        if let Some(s) = syntax_str {
            out.push('\n');
            out.push_str("Syntax: ");
            out.push_str(&s);
            out.push('\n');
        }
    }

    // Category line.
    {
        out.push('\n');
        out.push_str("Category: ");
        out.push_str(entry.category.as_str());
        out.push('\n');
    }

    // Purity line.
    {
        out.push('\n');
        out.push_str("Purity: ");
        out.push_str(entry.purity.as_str());
        out.push('\n');
    }

    // Determinism line.
    {
        out.push('\n');
        out.push_str("Determinism: ");
        out.push_str(entry.determinism.as_str());
        out.push('\n');
    }

    // Yields line (optional — only for HOF intrinsics with @yields).
    if let Some(yields_ty) = entry.yields_type {
        out.push('\n');
        out.push_str("Yields: ");
        out.push_str(yields_ty);
        out.push('\n');
    }

    // Examples section (if any).
    if !entry.examples.is_empty() {
        out.push('\n');
        out.push_str("Examples:\n");
        for ex in entry.examples {
            out.push_str("  ");
            out.push_str(ex.expr);
            if let Some(expected) = ex.expected {
                out.push_str("  #=> ");
                out.push_str(expected);
            }
            out.push('\n');
        }
    }

    // See also section (if any).
    if !entry.see.is_empty() {
        out.push('\n');
        out.push_str("See also:\n");
        for &fqdn in entry.see {
            out.push_str("  ");
            out.push_str(fqdn);
            out.push('\n');
        }
    }

    Ok(Value::String(Arc::new(out)))
}
