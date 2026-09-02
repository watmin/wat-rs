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

use std::borrow::Cow;
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
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Reflection
/// @ret (:wat::core::Vector :- [:wat::intrinsic::Example]) a Vector of Example records, one per @example/@example-norun across all registered intrinsics
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
            // Field type is (Option :- [:wat::WatAST]) → Value::Option.
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

/// Does a single `@see` target resolve? Extracted out of [`check_see_refs`] so the negative
/// case (an *undeclared* wat target must still be flagged dangling — STOP-3) can be exercised
/// directly, on a single constructed FQDN, without needing a real corpus entry that carries a
/// dangling `@see` (there is none — the whole point is that the gate is green).
///
/// Arc 255 STONE "`@see` can cross the boundary" — a target resolves against **both** stores,
/// per the DESIGN's pinned contract
/// (`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-see-can-cross-the-boundary.md`):
///   - a **registered Rust intrinsic** (`registry().lookup_entry`, as before), OR
///   - a **declared wat verb** — present in the frozen stdlib's `binding_metadata`
///     AND carrying an axis-declaration key. `contains_key` alone is NOT the
///     test: a capability-only map (e.g. `{:restricted-to […]}`) is stored in
///     `binding_metadata` and declares nothing, so accepting it on presence
///     alone would point a reader at a verb with no documentation — the exact
///     dead link this rule forbids. `crate::declare::register::meta_has_doc_axis_key` is
///     the SAME predicate the storage door (`record_binding_metadata`) and the
///     `metadata-of` reflection surface already use — reused here, not
///     restated, so the three cannot drift apart on what counts as "declared".
///
/// `reg` and `wat_binding_metadata` are threaded in rather than re-fetched, so a caller walking
/// many targets (`check_see_refs`) builds the `FrozenWorld` once and this fn does no I/O of its
/// own.
#[cfg(test)]
pub(crate) fn see_target_resolves(
    target: &str,
    reg: &crate::intrinsic::IntrinsicRegistry,
    wat_binding_metadata: &crate::value::symbol_table::BindingMetadata,
) -> bool {
    let is_registered_intrinsic = reg.lookup_entry(target).is_some();
    let is_declared_wat_verb =
        wat_binding_metadata.get(target).is_some_and(crate::declare::register::meta_has_doc_axis_key);
    is_registered_intrinsic || is_declared_wat_verb
}

/// Build the ONE bare `FrozenWorld` (`crate::freeze::startup_bare` — stdlib loaded, no user
/// program) `check_see_refs` and its direct-target test siblings check `@see` targets against.
/// The wat store (`binding_metadata`) does not exist until the stdlib has loaded, so this must
/// run once before any `@see` is resolved, never once per target.
#[cfg(test)]
pub(crate) fn bare_stdlib_world() -> crate::freeze::FrozenWorld {
    crate::freeze::startup_bare().unwrap_or_else(|e| {
        panic!("check_see_refs: startup_bare() failed to freeze the stdlib: {e:?}")
    })
}

/// Walk the intrinsic registry and collect dangling `@see` references —
/// FQDNs listed in an entry's `@see` doc that do NOT resolve to either
/// store (see [`see_target_resolves`] for what "resolve" means). An empty
/// result means the corpus is internally consistent.
///
/// The wat store does not exist until the stdlib has loaded (`check_see_refs`
/// runs as a bare `#[test]`, outside any program), so [`bare_stdlib_world`] is
/// called ONCE for the whole walk, not once per `@see`.
///
/// Lives in `#[cfg(test)]` — only consumed by the @see cross-check tests in
/// `intrinsic/mod.rs`. The `see` field is also read by `eval_render_doc`'s
/// "See also:" section (non-test code), so there is no dead-code issue.
#[cfg(test)]
pub(crate) fn check_see_refs() -> Vec<String> {
    let reg = crate::intrinsic::registry();

    // Build the second store ONCE — a bare frozen world carries no user
    // program, only the loaded stdlib, so `world.symbols().binding_metadata`
    // is exactly the wat-declaration store `@see` needs to check against.
    let world = bare_stdlib_world();
    let wat_binding_metadata = &world.symbols().binding_metadata;

    let mut dangling: Vec<String> = Vec::new();
    for entry in reg.all_entries() {
        for &see_fqdn in entry.see {
            if !see_target_resolves(see_fqdn, reg, wat_binding_metadata) {
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
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
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

    // Registry path. Gate on `entry.kind`, NOT on `entry.source.is_empty()` — an empty
    // capture from a genuine intrinsic must stay distinguishable from a special form's
    // structural absence of source (arc 255 Stone P2; see NOTE-an-absence-recorded-as-
    // an-answer). `Kind::SpecialForm` entries carry `source: ""` by construction (special
    // forms are dispatched by the runtime engine, not a registered Rust fn) — route them
    // to the SAME honest fallback text used below for `Binding::SpecialForm` et al., so
    // there is exactly one wording for "no source available in this context", not two.
    if let Some(entry) = crate::intrinsic::registry().lookup_entry(&name) {
        return Ok(match entry.kind {
            // Arc 255 Stone P6-a — a special form's declaration names none of its own
            // implementations (`#[wat_special_form]` sits on a doc-only unit struct; a
            // proc-macro cannot reach into another file to capture `eval_if`'s body). The
            // THIRD inventory stream (`#[wat_special_form_impl]`, gathered into `entry.impls`)
            // is where the real source lives. `impls.is_empty()` must keep EXACTLY ONE meaning
            // — "no source available in this context", P2's wording, unchanged — whether the
            // cause is "nobody has annotated this form yet" (every form P6-c will add) or a
            // genuinely source-less substrate primitive; a second wording here would be the
            // same absence-as-answer defect this NOTE family exists to kill (STOP-4).
            crate::intrinsic::Kind::SpecialForm if entry.impls.is_empty() => Value::String(Arc::new(format!(
                ";; {} — substrate primitive (no source available in this context)",
                name
            ))),
            crate::intrinsic::Kind::SpecialForm => {
                // check → eval → tail order (the NOTE's own regime order — check runs once
                // statically, eval and tail are the mutually-exclusive per-invocation
                // regimes), never inventory's arbitrary submission order.
                let role_order = [
                    crate::intrinsic::SpecialFormRole::Check,
                    crate::intrinsic::SpecialFormRole::Eval,
                    crate::intrinsic::SpecialFormRole::Tail,
                ];
                let mut out = format!(";; {} — special form (check · eval · tail)\n", name);
                for role in role_order {
                    for (r, source) in &entry.impls {
                        if *r == role {
                            out.push_str(&format!("\n;; role: {}\n{}\n", role.label(), source));
                        }
                    }
                }
                Value::String(Arc::new(out))
            }
            _ => Value::String(Arc::new(entry.source.to_string())),
        });
    }

    // User-form path: look up via the symbol table and write-forms the body.
    match crate::reflect::lookup::lookup_form(&name, sym) {
        Some(crate::reflect::lookup::Binding::UserFunction { f, .. }) => {
            match &f.body {
                crate::value::FunctionBody::Wat(ast) => {
                    let edn = crate::edn::bridge::watast_to_edn(ast.as_ref());
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
        Some(crate::reflect::lookup::Binding::Macro { def, .. }) => {
            let edn = crate::edn::bridge::watast_to_edn(&def.body);
            let text = wat_edn::write(&edn);
            Ok(Value::String(Arc::new(text)))
        }
        Some(crate::reflect::lookup::Binding::Primitive { .. })
        | Some(crate::reflect::lookup::Binding::SpecialForm { .. })
        | Some(crate::reflect::lookup::Binding::Type { .. })
        // Arc 255 Stone 3a-i — `lookup_form` now consults `crate::intrinsic::registry()`
        // itself (step 3, ahead of CheckEnv/special_forms), so this arm CANNOT actually fire:
        // any name for which it would fire already matched the `registry().lookup_entry(&name)`
        // check at the top of this function and returned there. Folded into the same honest
        // fallback text for exhaustiveness, not because it is reachable.
        | Some(crate::reflect::lookup::Binding::Registered { .. }) => {
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
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
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
    // When empty but @arg entries are present, derive: `(<fqdn> <arg>…)`.
    //
    // ⛔ THE HEAD IS THE FULL FQDN, and it used to be the last `::` segment
    // (`:wat::core::if` → `if`). Arc 255 Stone 1a-α, builder's ruling: **wat is FQDN,
    // always — anything that is not a binder is illegal; even bound symbols are
    // shadow-FQDN, in `$bound`.** A `Syntax:` line is a grammar a reader is meant to be
    // able to TYPE, so a short head does not render the form more readably, it renders
    // something that is not wat. `signature_of_defn`'s `@arg` path
    // (`src/reflect/verbs.rs`) has always emitted the full keyword here; this renderer
    // was the one out of step, and a doc surface teaching an illegal spelling is the
    // same defect class as `match`'s six-week `-> <T>` fossil, one layer down.
    {
        let syntax_str: Option<String> = if !entry.syntax.is_empty() {
            Some(entry.syntax.to_string())
        } else if !entry.args.is_empty() {
            let head = entry.name;
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

    // Yields section (optional — only for HOF intrinsics with ≥1 `@yields`). Arc 255
    // Stone P5-b: N lines, one per subject, `<argname> <type>` — the TYPE is DERIVED from
    // the matching `@arg`'s own canonical bracket-form type (`fn_arg_param_type`), never
    // re-typed by hand, so it cannot drift from what the `@arg` itself declares.
    if !entry.yields.is_empty() {
        out.push('\n');
        out.push_str("Yields:\n");
        for &(arg_name, _desc) in entry.yields {
            let arg_ty = entry.args.iter().find(|&&(name, ..)| name == arg_name).map(|&(_, ty, ..)| ty);
            let param_ty = arg_ty.and_then(super::fn_arg_param_type);
            out.push_str("  ");
            out.push_str(arg_name);
            out.push(' ');
            out.push_str(param_ty.unwrap_or("?"));
            out.push('\n');
        }
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

// ─── Arc 109 β-ii-c: type-params-used-in ─────────────────────────────────────

/// Does `param_name` appear as a whole TOKEN inside `content` (a Keyword's text with any
/// leading `:` already stripped, or a bare Symbol's text)? Splits on the type-parameter
/// boundary characters `<`, `,`, `>` — never a bare substring test — so `content.split(...)`
/// yields the run of characters between two boundaries (or a boundary and the string's own
/// start/end), and `param_name` must equal one of those runs exactly. This is what keeps `K`
/// from matching inside `Key` or `KV`: `"wat::cache::Lru<K,V>"` splits (on `<`/`,`/`>`) into
/// `["wat::cache::Lru", "K", "V", ""]`; a bare `"V"` (no brackets, the whole stripped token)
/// splits into the single run `["V"]`.
fn token_names_param(content: &str, param_name: &str) -> bool {
    content
        .split(['<', ',', '>'])
        .any(|segment| segment == param_name)
}

/// Does `param_name` appear anywhere within `node` — walking every List/Vector/Set item and
/// every Map key+value, and at each Keyword or Symbol LEAF matching the param as a bounded
/// token (never a bare substring)? A type parameter can live INSIDE a token: with today's
/// keyword type spelling, `:wat::cache::Lru<K,V>` is ONE `Keyword` node whose text carries `K`
/// and `V` as substrings, so a children-only walk would silently find nothing.
fn param_appears_in(param_name: &str, node: &WatAST) -> bool {
    match node {
        WatAST::Keyword(s, _) => {
            let content = s.strip_prefix(':').unwrap_or(s.as_str());
            token_names_param(content, param_name)
        }
        WatAST::Symbol(ident, _) => ident.as_str() == param_name,
        WatAST::List(items, _) | WatAST::Vector(items, _) | WatAST::Set(items, _) => {
            items.iter().any(|c| param_appears_in(param_name, c))
        }
        WatAST::Map(pairs, _) => pairs
            .iter()
            .any(|(k, v)| param_appears_in(param_name, k) || param_appears_in(param_name, v)),
        _ => false,
    }
}

/// Extract a type-param's bare name text from its `params`-element node: a `Symbol` (the usual
/// shape — `fqdn-tp-syms` mints these via `symbol-node`) or a `Keyword` (its text with any
/// leading `:` stripped), so a caller may pass either spelling.
fn param_name_of<'a>(node: &'a WatAST, op: &'static str) -> Result<Cow<'a, str>, RuntimeError> {
    match node {
        WatAST::Symbol(ident, _) => Ok(Cow::Borrowed(ident.as_str())),
        WatAST::Keyword(s, _) => Ok(Cow::Borrowed(s.strip_prefix(':').unwrap_or(s.as_str()))),
        other => Err(RuntimeError::new(other.span().clone(), RuntimeErrorKind::TypeMismatch {
            op: op.into(),
            expected: ":wat::WatAST (a Symbol or Keyword naming a type param)",
            got: Box::new(crate::runtime::ValueSnapshot::unavailable(
                "non-name AST node in `params`",
            )),
        })),
    }
}

/// Return the subset of `params` — in the order given — that appear anywhere within `node`.
///
/// A pure, total, structural AST search: no allocation-order dependence, no evaluation of
/// `node`'s content beyond walking its shape. Built for `defservice` (arc 109 β-ii-c) so a
/// generated companion type can carry only the type params its own field/member vector
/// actually mentions, instead of the service's FULL declared param list stamped onto every
/// companion regardless of use — but the question ("which of these params does this AST
/// mention?") is general; any future declarator macro needing the same answer can reuse it.
///
/// A program-body macro cannot express this search itself: no recursion (`foldl` walks one
/// level only), no helper `defn` (the F5 pure-combinator allow-list refuses a user-defined head
/// AT DEFINITION), no `mapv` over a bare primitive keyword. See
/// `NOTE-the-F5-allow-list-and-what-a-macro-body-may-call.md`.
///
/// ⚠ A type parameter can live INSIDE a single token. With today's keyword type spelling,
/// `:wat::cache::Lru<K,V>` is ONE `Keyword` node whose *name* carries `K` and `V` as text — a
/// walk that only descends into children (never inspecting a leaf's own text) finds nothing.
/// This walks every List/Vector/Set item and every Map key+value, and at each Keyword/Symbol
/// leaf matches a param name against a token bounded by `<`, `,`, `>`, and the text's own
/// start/end — never a bare substring, so `K` does not match inside `Key` or `KV`.
///
/// **Expand-time ground —** same category as its `ast-*` siblings (pure, total, structural
/// node walk; no IO): asks which of a set of type-param name nodes appear anywhere in an AST;
/// `defservice` calls it at expand time to compute each generated companion type's own param
/// subset. Ruling relocated from `macros/eval.rs`'s expand-time allow-list (arc 255 expand-T4a;
/// arc 109 β-ii-c); the verdict is that list's.
///
/// Arc 255 Stone the-registry-answers-first-wave-2 — OVERTURNS its own guard's `total: true`.
/// Re-derived by reading `param_name_of` (this file, immediately above): `params`'s declared
/// element type is the BROAD `:wat::WatAST` (any AST node), but `param_name_of` requires each
/// element to be specifically a `Symbol` or `Keyword` — anything else (a `List`, `Map`, literal,
/// …) raises `TypeMismatch` (*"non-name AST node in `params`"*), and nothing in the declared
/// signature rules that shape out (this verb is also on `intrinsic/mod.rs`'s
/// `FROZEN_CHECKER_DEBT_LEDGER` — it carries no `env.register()` TypeScheme at all, so the
/// checker does not even enforce the outer `:wat::WatAST` wrapping, let alone the inner
/// Symbol/Keyword shape). Empirically confirmed against the pre-stone binary: a call passing a
/// quoted `(1 2)` list as a `params` element passes `--check` (exit 0) and raises `TypeMismatch`
/// at run. Same class as `:wat::core::with-children`'s already-`Partial` ruling
/// (`src/rete/purity.rs`'s `intrinsic_meta`, an unregistered head this stone leaves untouched),
/// and the same RULING this campaign already applied to `:wat::string::concat`
/// (`RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Partial
/// @ExpandTime    Legal
/// @Category      Reflection
/// @arg params (:wat::core::Vector :- [:wat::WatAST]) the candidate type-param name nodes (Symbols, or Keywords), in declaration order
/// @arg node :wat::WatAST the AST subtree to search — typically a field/member vector
/// @ret (:wat::core::Vector :- [:wat::WatAST]) the subset of `params`, in the order given, that appear anywhere in `node`
/// @example (:wat::core::type-params-used-in (:wat::core::Vector :- [:wat::WatAST] (:wat::core::symbol-node "K") (:wat::core::symbol-node "V")) (:wat::core::keyword-node ":wat::core::Vector<K>")) #=> [K]
/// @example (:wat::core::type-params-used-in (:wat::core::Vector :- [:wat::WatAST] (:wat::core::symbol-node "K") (:wat::core::symbol-node "V")) (:wat::core::keyword-node ":wat::core::HashMap<Key,KV>")) #=> []
/// @example-norun (:wat::core::type-params-used-in (:wat::core::Vector (:wat::core::symbol-node "K") (:wat::core::symbol-node "V")) (:wat::core::read-string "[cache <- (:wat::cache::Lru :- [K V])]")) #=> Vector[Symbol(K), Symbol(V)]
/// @example-norun (:wat::core::type-params-used-in (:wat::core::Vector (:wat::core::symbol-node "K") (:wat::core::symbol-node "V")) (:wat::core::read-string "[capacity <- :wat::core::i64]")) #=> Vector[]
#[wat_intrinsic(":wat::core::type-params-used-in")]
pub(crate) fn eval_type_params_used_in(
    params: &WatAST,
    node: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::type-params-used-in";
    let _ = span;

    let params_v = crate::runtime::eval_inner(params, env, sym)?.value_owned();
    let param_vals: &Vec<Value> = match &params_v {
        Value::Vec(v) => v.as_ref(),
        other => {
            return Err(RuntimeError::new(params.span().clone(), RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "(:wat::core::Vector :- [:wat::WatAST])",
                    got: Box::new(crate::runtime::ValueSnapshot::of(other)),
                })
            .into());
        }
    };

    let node_v = crate::runtime::eval_inner(node, env, sym)?.value_owned();
    let node_ast: &WatAST = match &node_v {
        Value::wat__WatAST(a) => a.as_ref(),
        other => {
            return Err(RuntimeError::new(node.span().clone(), RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::WatAST",
                    got: Box::new(crate::runtime::ValueSnapshot::of(other)),
                })
            .into());
        }
    };

    let mut out: Vec<Value> = Vec::with_capacity(param_vals.len());
    for pv in param_vals.iter() {
        let pv_ast: &WatAST = match pv {
            Value::wat__WatAST(a) => a.as_ref(),
            other => {
                return Err(RuntimeError::new(params.span().clone(), RuntimeErrorKind::TypeMismatch {
                        op: OP.into(),
                        expected: "(:wat::core::Vector :- [:wat::WatAST]) of param name nodes",
                        got: Box::new(crate::runtime::ValueSnapshot::of(other)),
                    })
                .into());
            }
        };
        let name = param_name_of(pv_ast, OP)?;
        if param_appears_in(&name, node_ast) {
            out.push(pv.clone());
        }
    }

    Ok(Value::Vec(Arc::new(out)))
}

/// Do `a` and `b` denote the SAME type, whatever spelling each wears?
///
/// Arc 109 stone (`BRIEF-STONE-type-equal-the-missing-door.md`) — the missing door: `TypeExpr`
/// already derives `PartialEq, Eq`, so structural type equality exists in Rust at check time, but
/// a macro body (this crate's sole caller-of-record: `defservice`) had no way to reach it and was
/// reduced to rendering both sides to strings and comparing text — a defect that breaks the moment
/// either side's DECLARED spelling changes shape (`Peer<A,B>` vs `(Peer :- [A B])`) even though
/// both denote the identical type. This verb parses each side once via [`crate::types::
/// parse_type_node`] — the one door that reads all four surfaces (keyword, `wat.type/` symbol,
/// parametric form, `[arg… :-> ret]` bracket) — and compares the resulting `TypeExpr`s directly.
///
/// Deliberately NOT: subtyping (`is_subtype` answers that, over the edge table); AST equality
/// (`Peer<A,B>` and `(Peer :- [A B])` are different ASTs denoting the same type — that gap is the
/// entire point); a checker-time answer (this compares DECLARED spellings at macro-expansion time,
/// before inference; a fresh-unification-var mismatch like `<T>` vs `<?454>` is a different
/// question `assignable`'s `else` branch already owns).
///
/// ★ Contract: given a node that does not parse as a type at all, this RAISES rather than
/// returning `false`. *"These are not both types"* is a different fact from *"these are different
/// types"* — collapsing them into `false` would make a malformed input indistinguishable from a
/// legitimate mismatch, a silent pass at exactly the site meant to catch a mistake. Mirrors the
/// `TypeMismatch` shape [`eval_type_params_used_in`] already uses for a bad argument, immediately
/// above.
///
/// **Expand-time ground —** the missing door: `defservice` and its siblings live entirely in
/// macro bodies and had no way to ask whether two declared type spellings are the same type;
/// pure ∧ deterministic ∧ total structural comparison via `parse_type_node` (raises on a
/// non-type node rather than returning a silently-wrong `false`), same category as
/// `type-params-used-in`. Ruling relocated from `macros/eval.rs`'s expand-time allow-list (arc
/// 255 expand-T4a; arc 109, `BRIEF-STONE-type-equal-the-missing-door.md`); the verdict is that
/// list's.
///
/// Arc 255 Stone the-registry-answers-first-wave-2 — OVERTURNS its own guard's `total: true`.
/// Re-derived by reading this fn's own doc two paragraphs up ("★ Contract: given a node that
/// does not parse as a type at all, this RAISES rather than returning `false`") and its body
/// (`parse_type_node` raising `TypeMismatch` for a non-type node): the declared arg type is the
/// BROAD `:wat::WatAST`, and this verb is on `intrinsic/mod.rs`'s `FROZEN_CHECKER_DEBT_LEDGER`
/// (no `env.register()` TypeScheme — the checker enforces nothing about `a`/`b` beyond parsing).
/// Empirically confirmed against the pre-stone binary: a call passing a quoted `(1 2 3)` list as
/// `a` passes `--check` (exit 0) and raises `TypeMismatch` ("not a type") at run. This is a
/// documented, deliberate raise-on-malformed-input — not a bug — but per
/// `RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md` a raise is not a matchable
/// outcome, so the verb is `Partial`, not `Total`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Partial
/// @ExpandTime    Legal
/// @Category      Reflection
/// @arg a :wat::WatAST a type-expression node — keyword, `wat.type/` symbol, parametric form `(Head :- [args])`, or fn-type bracket `[arg… :-> ret]`
/// @arg b :wat::WatAST the other type-expression node, same surface set
/// @ret :wat::core::bool true iff `a` and `b` denote the same type
/// @example (:wat::core::type-equal? (:wat::core::keyword-node ":wat::kernel::Peer<A,B>") '(:wat::kernel::Peer :- [A B])) #=> true
/// @example (:wat::core::type-equal? (:wat::core::keyword-node ":wat::core::Vector<wat::core::HashMap<K,V>>") '(:wat::core::Vector :- [(:wat::core::HashMap :- [K V])])) #=> true
/// @example (:wat::core::type-equal? (:wat::core::keyword-node ":wat::kernel::Peer<A,B>") (:wat::core::keyword-node ":wat::kernel::Peer<B,A>")) #=> false
/// @example (:wat::core::type-equal? (:wat::core::keyword-node ":wat::core::i64") (:wat::core::keyword-node ":wat::core::i64")) #=> true
/// @example (:wat::core::type-equal? (:wat::core::keyword-node ":wat::core::i64") (:wat::core::keyword-node ":wat::core::String")) #=> false
#[wat_intrinsic(":wat::core::type-equal?")]
pub(crate) fn eval_type_equal(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::type-equal?";
    let _ = span;

    let a_v = crate::runtime::eval_inner(a, env, sym)?.value_owned();
    let a_ast: &WatAST = match &a_v {
        Value::wat__WatAST(ast) => ast.as_ref(),
        other => {
            return Err(RuntimeError::new(a.span().clone(), RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::WatAST",
                    got: Box::new(crate::runtime::ValueSnapshot::of(other)),
                })
            .into());
        }
    };

    let b_v = crate::runtime::eval_inner(b, env, sym)?.value_owned();
    let b_ast: &WatAST = match &b_v {
        Value::wat__WatAST(ast) => ast.as_ref(),
        other => {
            return Err(RuntimeError::new(b.span().clone(), RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::WatAST",
                    got: Box::new(crate::runtime::ValueSnapshot::of(other)),
                })
            .into());
        }
    };

    let a_ty = crate::types::parse_type_node(a_ast).map_err(|e| {
        RuntimeError::new(a_ast.span().clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: ":wat::WatAST (a type expression: keyword, wat.type/ symbol, parametric form, or fn-type bracket)",
            got: Box::new(crate::runtime::ValueSnapshot::described(
                ":wat::WatAST",
                format!("not a type: {e}"),
            )),
        })
    })?;

    let b_ty = crate::types::parse_type_node(b_ast).map_err(|e| {
        RuntimeError::new(b_ast.span().clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: ":wat::WatAST (a type expression: keyword, wat.type/ symbol, parametric form, or fn-type bracket)",
            got: Box::new(crate::runtime::ValueSnapshot::described(
                ":wat::WatAST",
                format!("not a type: {e}"),
            )),
        })
    })?;

    Ok(Value::bool(a_ty == b_ty))
}

/// `(:wat::core::type v) -> :wat::core::String` — arc 234 Stone 234.0, homed arc 255 Stone
/// A-2-ii-b-0 alongside its `type-params-used-in`/`type-equal?` siblings.
///
/// Polymorphic runtime primitive that extracts a Value's record-type FQDN as a String.
/// Thin `#[wat_intrinsic]` delegate over the pre-existing `eval_type` (`src/runtime.rs`) — the
/// body does not move. Homed here with its real (1) arity declared; the hand-rolled
/// `args.len() != 1` guard in `eval_type` retires.
///
/// **Purity ground:** the sole arg is evaluated by ordinary call-by-value (not itself an
/// effect). Past that, the body only reads the already-evaluated value's declared type name off
/// it (`Value::declared_type_name`) — no `eval_inner`/`apply_function` on caller-supplied code
/// beyond the one argument evaluation. Pure ∧ Deterministic.
///
/// **Totality ground — measured, not transcribed (this stone's own determination):**
/// `eval_type`'s body has exactly ONE `return Err` — the arity-mismatch check above, which
/// retires on homing (the macro-generated shim now enforces arity before this body runs). Past
/// that single arity check, `eval_type` unconditionally calls `arg_val.declared_type_name()`
/// (`src/runtime.rs:16372`-area) and returns `Ok`; nothing else in the body can fail.
/// `declared_type_name` (`src/value/value.rs:1705`) returns a bare `String`, not a `Result` —
/// there is no `Err` arm for it to produce — and its own match is exhaustive over every `Value`
/// variant with no bare `_ =>`/`other =>` catch-all (its doc: "the Rust compiler rejects a
/// future variant that forgets to supply its declared-type arm"), so no domain hole is even
/// structurally possible: every variant, nominal or primitive, has an explicit arm that returns
/// a name. With the arity failure retired and zero remaining domain-failure paths, this verb is
/// `Total`.
///
/// **Expand-time ground —** Pure ∧ Deterministic and safe to evaluate during expansion; also
/// `Total`, so no partiality question even arises for expand-time legality here. Legal.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Total
/// @ExpandTime    Legal
/// @Category      Reflection
/// @arg     v :T any value — every `Value` variant has a declared-type arm
/// @ret     :wat::core::String the value's record-type FQDN
/// @example (:wat::core::type 3) #=> "wat::core::i64"
/// @see     :wat::core::type-equal?
#[wat_intrinsic(":wat::core::type")]
pub(crate) fn eval_type(
    v: &WatAST,
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_type(std::slice::from_ref(v), list_span, env, sym)
}
