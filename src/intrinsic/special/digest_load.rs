//! Special-form doc entry for `:wat::digest-load!` — arc 255 Stone 1a-δ, sibling of
//! `:wat::load-file!` (`load_file.rs`) and `:wat::signed-load!` (`signed_load.rs`): shape ②
//! of `DESIGN-STONE-1a-delta-and-epsilon-three-shapes-of-not-really-evaluating.md`'s three.

use wat_macros::wat_special_form;

/// Load `<path>`, verify the fetched bytes hash to the declared SHA-256 digest BEFORE
/// parsing, then splice the parsed forms into the surrounding program in place of this form.
/// `match_load_form` (`src/load/loader.rs:653`) routes the FQDN to `parse_digest_load_file`
/// (`loader.rs:719`, delegating to `parse_digest_load_shared`), which parses the four args
/// (path, `:wat::verify::digest-<algo>`, payload-interface keyword, payload locator) into a
/// `LoadSpec { verification: Some(VerificationSpec::Digest { .. }), .. }`; `process_single_load`
/// (`loader.rs:489`) fetches the bytes, then `verify_pre_parse` (`loader.rs:596`) hashes them
/// and compares to the declared hex digest — PRE-parse, against raw bytes — before the forms
/// are ever spliced in.
///
/// **Category ground —** same as `:wat::load-file!`'s (`load_file.rs`) — same splicing
/// mechanism, an added integrity gate in front of it: `process_forms` replaces this form's node
/// with the verified file's own forms, one node becoming N, and the node itself does not
/// survive. That is `:Splice`, not `:Declaration`: this form registers nothing itself — the
/// hash check and fetch are the internal mechanism by which the splice is populated, not an
/// entry in any program-level table, and not this form's own observable effect at a `role =
/// eval` call site the way `Io` requires (there is no eval arm — shape ②, verified below).
/// `Splice`.
///
/// **Purity ground —** measured directly: `:wat::digest-load!` appears in `src/runtime.rs`
/// exactly ONCE, inside `is_mutation_head` — a hand-list, not a dispatch arm — and nowhere in
/// `dispatch_keyword_head_value`, `eval_tail`, or `step_list`. No `handler`, no eval arm, no
/// tail arm. Shape ②, the same as `load-file!`: no eval arm exists to refuse (`def`'s shape ①)
/// or return a no-op from (`use!`'s shape ③). `Unevaluated`.
///
/// **Determinism ground —** differs from `load-file!`'s in WHAT is unpinned, not IN whether
/// anything is: on a SUCCESSFUL load, the fetched bytes are cryptographically pinned to the
/// declared digest (a SHA-256 collision aside), so the spliced content itself cannot silently
/// vary across two matching loads. But whether a given call succeeds AT ALL still depends on
/// ambient disk state at fetch time — the same `loader.fetch_source_file` read `load-file!`'s
/// row measures, gated rather than removed: a `target.wat` that changes on disk between two
/// otherwise-identical loads can flip a call from succeeding to raising `VerificationFailed`
/// (or back), with no argument to this form changing. `@Determinism` and `@Totality` are
/// orthogonal axes (`wat/runtime-meta.wat`'s own header) — this row is Nondeterministic AND
/// Partial for two DIFFERENT reasons, not one implying the other. `Nondeterministic`.
///
/// **Totality ground —** `parse_digest_load_shared` (`loader.rs:730`) alone can raise on a
/// wrong arg count (must be exactly 4), a non-string source, a verify-algo keyword missing the
/// `:wat::verify::digest-` prefix or naming an unsupported algorithm (`parse_verify_algo`,
/// `loader.rs:894`), or a malformed payload-interface keyword/locator (`parse_payload_
/// interface`, `loader.rs:834`) — ALL of `load-file!`'s own arg-shape failure surface plus this
/// form's own three additional keyword-shape checks. Declare-time processing of a MATCHED form
/// adds `load-file!`'s full downstream failure surface (`Fetch`, `CycleDetected`,
/// `SetterInLoadedFile`, `Parse`) PLUS `VerificationFailed` (`verify_pre_parse`, `loader.rs:604`–
/// `616`) when the fetched bytes' SHA-256 does not match the declared hex digest — a raise the
/// freeze pipeline propagates as a hard failure, never a value a caller matches on. Same
/// reasoning `:wat::i64::/`'s own `@Totality Partial` was ruled on
/// (`RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`). `Partial`.
///
/// **Expand-time ground —** identical to `load-file!`'s: no runtime call site at all (`role =
/// declare` emits no shim); `resolve_loads` is step 3 of the startup pipeline, strictly BEFORE
/// `register_defmacros` → `expand_all` (step 4) — load resolution (digest verification
/// included) has already run to completion before any `defmacro` body begins expanding. Also
/// absent from `macros/eval.rs`'s expand-time pure-total allow-list. `RuntimeOnly`.
///
/// @added 1.0.0
/// @Category Splice
/// @Purity Unevaluated
/// @Determinism Nondeterministic
/// @Totality Partial
/// @ExpandTime RuntimeOnly
/// @syntax (:wat::digest-load! <path> :wat::verify::digest-<algo> :wat::verify::<iface> <payload>)
/// @ret :wat::core::nil no runtime value — the form is consumed entirely at load-resolution time and never reaches evaluation; its effect is the digest-verified file's forms, spliced into the surrounding program
/// @example-norun (:wat::digest-load! "lib/util.wat" :wat::verify::digest-sha256 :wat::verify::string "<64-hex-char sha256 of lib/util.wat's bytes>") #=> verifies lib/util.wat's bytes against the declared sha256 digest, then splices its forms in; no runtime value
#[wat_special_form(":wat::digest-load!")]
pub(crate) struct DigestLoad;
