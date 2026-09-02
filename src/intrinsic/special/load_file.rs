//! Special-form doc entry for `:wat::load-file!` — arc 255 Stone 1a-δ, shape ② of
//! `DESIGN-STONE-1a-delta-and-epsilon-three-shapes-of-not-really-evaluating.md`'s three: a
//! form that never reaches evaluation at all (no eval arm to refuse it, unlike `def`'s
//! shape ①, and no no-op eval arm either, unlike `use!`'s shape ③).

use wat_macros::wat_special_form;

/// Load `<path>` (resolved relative to the loading file's own directory), parse it, and
/// splice its forms into the surrounding program in place of this form — no integrity check
/// on the fetched bytes. `match_load_form` (`src/load/loader.rs:653`) routes the FQDN to
/// `parse_unverified_load` (`loader.rs:680`), which parses the single string-literal arg into
/// a `LoadSpec { source: FilePath(path), verification: None }`; `process_single_load`
/// (`loader.rs:489`) then fetches the bytes (`fetch_source` → `loader.fetch_source_file`,
/// `loader.rs:568`), parses them, and recursively resolves any loads they themselves contain,
/// before appending the result into `out` — the caller's flat form list.
///
/// **Category ground —** the axis is the DOING, and the DOING is measured at the language
/// level, not the Rust call graph: `process_forms` (`loader.rs:470`) replaces the load-form
/// node with the loaded file's own forms, verbatim, in the surrounding form stream — the
/// spliced declarations (`def`s, `defclause`s, more loads) become part of THIS program,
/// visible to every form after this one, exactly `Declaration`'s own variant prose in
/// `wat/runtime-meta.wat` ("registers a program-level entity … visible to everything after
/// it"). **`Io` is refused**, for two reasons. First, at the language level: `:wat::io::read-
/// file` (`src/intrinsic/io/fs.rs`) is ruled `Io` because its ENTIRE observable effect is
/// "data crosses the process boundary IN and is handed back as a value" — a real `role = eval`
/// call site the caller's expression consumes. `:wat::load-file!` has no such call site (shape
/// ②, verified below): the fetch (`loader.fetch_source_file`, the SAME fn `read-file` calls)
/// is not this form's OWN observable effect, it is the internal mechanism by which the
/// declaration gets populated — exactly the relationship `parse_aggregate`'s internal Rust
/// computation has to `structtype`'s `Declaration` ground, not `read-file`'s to its `Io`.
/// Second, and independent of the prose reading: `every_special_form_carries_check_and_eval_
/// impls` (`src/intrinsic/mod.rs:2497`) DERIVES its required-impl-set from `entry.category` —
/// only a `Category::Declaration` row (`mod.rs:2503`) is permitted a `role = declare`-only
/// impl set (`mod.rs:2504`'s `has_declare` check); any other category demands BOTH `role =
/// check` and `role = eval` (`mod.rs:2508`–`2518`'s `else` branch), which this row does not and
/// — per shape ② — must not carry. Ruling `Io` would make this row fail that gate on
/// registration, before any prose argument even matters.
///
/// **Purity ground —** measured directly: `:wat::load-file!` appears in `src/runtime.rs`
/// exactly ONCE, inside `is_mutation_head` — a hand-list, not a dispatch arm — and nowhere in
/// `dispatch_keyword_head_value`, `eval_tail`, or `step_list`. No `handler`, no eval arm, no
/// tail arm. Shape ②, not shape ① (`def`'s regime, which REFUSES itself at eval with
/// `DeclarationInExpressionPosition`) and not shape ③ (`use!`'s regime, which evaluates to
/// `Ok(Value::Unit)` as a no-op) — there is no eval arm here to refuse OR return from. All four
/// consumers of `@Purity` ask a RUNTIME question, and `:wat::load-file!` has no runtime to ask
/// it about — `Pure` would demand a runnable `@example` of a verb that cannot be run,
/// `Effectful` would claim an effect there is no call to have, `Preserving` would claim
/// sub-forms that are never evaluated (the path is a string literal, read once, at load-
/// resolution time). `Unevaluated`.
///
/// **Determinism ground —** unlike the type-declaration family (`structtype`/`newtype`/…,
/// whose declare-time processing is closed entirely over already-declared program state, no
/// external read), `:wat::load-file!`'s processing performs a real filesystem read
/// (`loader.fetch_source_file`) with NO integrity check pinning the result — the identical
/// ambient-state dependency `:wat::io::read-file`'s own row measures for its `Nondeterministic`
/// ruling ("the same path argument can return different content across two calls if the file
/// changed on disk between them"). Unpinned: two loads of the same form, same preceding
/// declarations, can splice DIFFERENT content if `target.wat` changed on disk in between, with
/// no verification to catch it. `Nondeterministic`.
///
/// **Totality ground —** `parse_unverified_load` alone is a narrow shape-check (arity —
/// exactly one arg — and string-literal type), but declare-time processing of a MATCHED form
/// is not: `process_single_load` can raise `LoadErrorKind::Fetch` (file missing / unreadable),
/// `CycleDetected` (a load chain closes on itself), `SetterInLoadedFile` (a loaded file
/// contains a `:wat::config::set-*!` form), or `Parse` (the fetched bytes don't parse) — each a
/// raise the freeze pipeline propagates as a hard failure, never a value a caller matches on.
/// `:wat::load-file!` carries no `VerificationFailed` arm (`verify_pre_parse`/`verify_post_
/// parse` both short-circuit `Ok(())` on `verification: None`) — the one failure mode its two
/// siblings below add. Same reasoning `:wat::i64::/`'s own `@Totality Partial` was ruled on
/// (`RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`). `Partial`.
///
/// **Expand-time ground —** `:wat::load-file!` has no runtime call site at all (`role =
/// declare` emits no shim) — `resolve_loads` is step 3 of the startup pipeline
/// (`src/freeze.rs`'s `startup_from_source` doc), and `register_defmacros` → `expand_all` is
/// step 4, strictly AFTER — so by the time any `defmacro` body is being expanded, load
/// resolution has already run to completion; there is no in-progress load-resolution state for
/// an expand-time call to reach, even in principle. Also absent from `macros/eval.rs`'s
/// expand-time pure-total allow-list (measured — no `:wat::load-file!` arm there). `RuntimeOnly`.
///
/// @added 1.0.0
/// @Category Declaration
/// @Purity Unevaluated
/// @Determinism Nondeterministic
/// @Totality Partial
/// @ExpandTime RuntimeOnly
/// @syntax (:wat::load-file! <path>)
/// @ret :wat::core::nil no runtime value — the form is consumed entirely at load-resolution time and never reaches evaluation; its effect is the loaded file's forms, spliced into the surrounding program
/// @example-norun (:wat::load-file! "lib/util.wat") #=> splices lib/util.wat's forms into the program at this point; no runtime value
#[wat_special_form(":wat::load-file!")]
pub(crate) struct LoadFile;
