//! Special-form doc entry for `:wat::signed-load!` — arc 255 Stone 1a-δ, sibling of
//! `:wat::load-file!` (`load_file.rs`) and `:wat::digest-load!` (`digest_load.rs`): shape ②
//! of `DESIGN-STONE-1a-delta-and-epsilon-three-shapes-of-not-really-evaluating.md`'s three.

use wat_macros::wat_special_form;

/// Load `<path>`, parse it, verify the PARSED AST against a declared ed25519 signature +
/// public key, then splice the verified forms into the surrounding program in place of this
/// form. `match_load_form` (`src/load/loader.rs:653`) routes the FQDN to `parse_signed_load_
/// file` (`loader.rs:762`, delegating to `parse_signed_load_shared`), which parses the six
/// args (path, `:wat::verify::signed-<algo>`, a payload-interface pair for the signature, a
/// second for the public key) into a `LoadSpec { verification: Some(VerificationSpec::Signed {
/// .. }), .. }`; `process_single_load` (`loader.rs:489`) fetches and parses the bytes FIRST,
/// then `verify_post_parse` (`loader.rs:621`) hashes the parsed forms' canonical EDN
/// (`crate::hash::verify_program_signature`) and checks the ed25519 signature against it —
/// POST-parse, against the AST, unlike `digest-load!`'s pre-parse raw-byte hash.
///
/// **Category ground —** same as `:wat::load-file!`'s (`load_file.rs`) and `:wat::digest-
/// load!`'s (`digest_load.rs`) — same splicing mechanism, a different (and later-running)
/// integrity gate in front of it: `process_forms` replaces this form's node with the
/// signature-verified file's own forms, one node becoming N, and the node itself does not
/// survive. That is `:Splice`, not `:Declaration`: this form registers nothing itself — the
/// fetch, parse, and signature check are the internal mechanism by which the splice is
/// populated, not an entry in any program-level table, and not this form's own observable
/// effect at a `role = eval` call site the way `Io` requires (there is no eval arm — shape ②,
/// verified below). `Splice`.
///
/// **Purity ground —** measured directly: `:wat::signed-load!` appears in `src/runtime.rs`
/// exactly ONCE, inside `is_mutation_head` — a hand-list, not a dispatch arm — and nowhere in
/// `dispatch_keyword_head_value`, `eval_tail`, or `step_list`. No `handler`, no eval arm, no
/// tail arm. Shape ②, the same as its two siblings: no eval arm exists to refuse (`def`'s
/// shape ①) or return a no-op from (`use!`'s shape ③). `Unevaluated`.
///
/// **Determinism ground —** the same shape as `digest-load!`'s, one layer later: on a
/// SUCCESSFUL load the parsed forms are cryptographically pinned to the declared signature +
/// public key (a forgery aside), so the spliced content itself cannot silently vary across two
/// matching loads. But — measured directly, `--check`ing a real ed25519 keypair/signature of
/// the RIGHT byte lengths against `target.wat` — whether a call succeeds still depends on
/// ambient disk state at fetch time: the same signature verified cleanly through base64
/// decode, length checks, and public-key validity, then failed at the final cryptographic
/// check (`SignatureMismatch`) because the fetched bytes' canonical-EDN hash did not match
/// what was signed — exactly the failure a `target.wat` edit between signing and loading would
/// also produce. `Nondeterministic`.
///
/// **Totality ground —** `parse_signed_load_shared` (`loader.rs:773`) alone can raise on a
/// wrong arg count (must be exactly 6), a non-string source, a verify-algo keyword missing the
/// `:wat::verify::signed-` prefix or naming an unsupported algorithm, or either of the TWO
/// payload-interface pairs (signature, public key) being malformed — twice `digest-load!`'s
/// own keyword-shape surface, since this form carries two payloads where `digest-load!`
/// carries one. Declare-time processing of a MATCHED form adds `load-file!`'s full downstream
/// failure surface (`Fetch`, `CycleDetected`, `SetterInLoadedFile`, `Parse`) PLUS
/// `VerificationFailed` (`verify_post_parse`, `loader.rs:621`–`648`) when the parsed AST's
/// signature does not verify — measured directly: a well-formed six-arg instantiation with a
/// real ed25519 keypair/signature (correct base64, correct 64/32-byte lengths, a valid
/// compressed Edwards point) still raises `VerificationFailed { cause: SignatureMismatch }`
/// when the signature is not actually over `target.wat`'s canonical-EDN hash — a raise the
/// freeze pipeline propagates as a hard failure, never a value a caller matches on. Same
/// reasoning `:wat::i64::/`'s own `@Totality Partial` was ruled on
/// (`RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`). `Partial`.
///
/// **Expand-time ground —** identical to its two siblings': no runtime call site at all
/// (`role = declare` emits no shim); `resolve_loads` is step 3 of the startup pipeline,
/// strictly BEFORE `register_defmacros` → `expand_all` (step 4) — load resolution (signature
/// verification included) has already run to completion before any `defmacro` body begins
/// expanding. Also absent from `macros/eval.rs`'s expand-time pure-total allow-list.
/// `RuntimeOnly`.
///
/// @added 1.0.0
/// @Category Splice
/// @Purity Unevaluated
/// @Determinism Nondeterministic
/// @Totality Partial
/// @ExpandTime RuntimeOnly
/// @syntax (:wat::signed-load! <path> :wat::verify::signed-<algo> :wat::verify::<iface> <sig> :wat::verify::<iface> <pubkey>)
/// @ret :wat::core::nil no runtime value — the form is consumed entirely at load-resolution time and never reaches evaluation; its effect is the signature-verified file's forms, spliced into the surrounding program
/// @example-norun (:wat::signed-load! "lib/util.wat" :wat::verify::signed-ed25519 :wat::verify::string "<base64 64-byte ed25519 sig>" :wat::verify::string "<base64 32-byte ed25519 pubkey>") #=> verifies lib/util.wat's parsed forms against the declared ed25519 signature, then splices its forms in; no runtime value
#[wat_special_form(":wat::signed-load!")]
pub(crate) struct SignedLoad;
