# BRIEF-STONE-255.SF — the special-form doc-contract (`if`/`let` exemplar)

**Read first, in full:** `DESIGN-STONE-special-form-doc-contract.md` (this dir) — it
is the frozen contract. This brief is the strike path; the DESIGN is the law.

**The work in one paragraph:** Add the SPECIAL-FORM kind to the intrinsic
doc/reflection registry, proven on `if` and `let`. Special forms have no uniform
handler, so the registry owns their DOC only (dispatch stays inline). Introduce a
bespoke `wat_special_form!` macro that anchors a per-form marker unit-struct,
parses the firm-grammar doc (special-form marker set), and submits a registry
entry with `handler: None`, `kind: SpecialForm`. The north-star probe
(`wat-tests/reflect/special-form-doc-if-let.wat`) goes from RED → GREEN.

## Rooms (read in order, each with why)

1. `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-special-form-doc-contract.md`
   — the contract (markers, witness model, the two exemplars, what dies).
2. `src/intrinsic/bytes.rs` — the VALUE exemplar to mirror (the `///` + `#[wat_intrinsic]` shape). Do NOT change its behavior.
3. `src/intrinsic/mod.rs:40-264` — `Kind` enum (add `SpecialForm`), `IntrinsicEntry`/`IntrinsicSubmission` (add fields), the `registry()` builder (~300), `lookup`/`lookup_entry` (~282).
4. `crates/wat-doc/src/lib.rs` — `parse()` (line 170) + required-directive enforcement (~173, ~452) + `DocComment`/`DocError` + `check_args` (480). You will add a special-form parse path.
5. `crates/wat-macros/src/wat_intrinsic.rs` + `lib.rs` — the value macro to mirror; `KNOWN_CATEGORIES` (wat_intrinsic.rs:272).
6. `src/intrinsic/reflect.rs:302` (`eval_render_doc`), `:230` (`show-source`) — must handle `kind: SpecialForm` + `handler: None`.
7. `src/runtime.rs:10107` (`eval_metadata_of`) — reads entry fields; report `Kind::SpecialForm`.
8. `src/special_forms.rs` — kill `signature: HolonAST` + `sketch()`/`build_registry()` sketch + its test (the crutch dies; `@syntax` replaces it). Keep `lookup_special_form` only if other code needs name-existence; otherwise retire it too (grep first).
9. `wat/runtime-meta.wat:16` — `defenum :wat::runtime::Kind` gains `:SpecialForm`.

## Implementation sketch (the strike path)

- **Entry/Submission (mod.rs):** add `kind: Kind`, `handler: Option<NativeHandler>`, `syntax: &'static str` (empty for value intrinsics). Change `pure: bool`→`purity: Purity` and `deterministic: bool`→`determinism: Determinism` (new tri-state enums: `Purity{Pure,Effectful,Preserving}`, `Determinism{Deterministic,Nondeterministic,Preserving}`). The value macro maps `@pure true`→`Purity::Pure`, `@pure false`→`Effectful`; special macro maps `@purity preserving`→`Preserving`, etc. Update the `registry()` builder, the cross-check tests (`pure_declared_matches_is_effectful_op`, `purity_mandated_examples`), and `derive_pure_deterministic` to read the enums. `lookup` becomes `.and_then(|e| e.handler)` — a `None`-handler entry does NOT dispatch via the registry (special forms stay inline).
- **wat-doc:** add a special-form parse entry (e.g. `parse_special_form(raw)`) that REQUIRES `@syntax`, makes `@arg` OPTIONAL, requires `@purity`/`@determinism` (tri-state) + `@category` + `@ret` + ≥1 `@example`, and REJECTS `@pure`/`@deterministic` (those are value-only). The value `parse()` stays as-is (rejects `@purity`/`@syntax`). Add a `syntax: String` field to `DocComment` (or a special-form-specific struct — your call, keep it clean).
- **wat-macros:** `wat_special_form!` attribute on a unit struct: read the `///` doc, call the special-form parser, validate `@category` against `KNOWN_CATEGORIES += ["ControlFlow","Binding"]`, emit `inventory::submit!` with `handler: None`, `kind: SpecialForm`, `syntax: <grammar>`. No arity sniff (no fn). Mirror `wat_intrinsic.rs`'s doc-error rendering.
- **special/ home:** `src/intrinsic/special/mod.rs` (mod-declared in `intrinsic/mod.rs`), `control_flow.rs` (`If`), `binding.rs` (`Let`) — the exact `///` + `#[wat_special_form(...)]` from the DESIGN §5.
- **reflect/runtime:** `render-doc` renders the `@syntax` line + `Kind: SpecialForm`; `show-source` on a `None`-handler entry returns the `@syntax` grammar + "dispatched inline at <sites>" (not a fn body); `metadata-of` reports `Kind::SpecialForm`.
- **runtime-meta.wat:** `:SpecialForm` added to the `Kind` defenum (iv-c invariant: Rust `Kind` variants MUST match this defenum exactly).

## Blast radius

`src/intrinsic/{mod,reflect}.rs`, `src/intrinsic/special/*` (new), `crates/wat-doc/src/lib.rs`, `crates/wat-macros/src/{wat_intrinsic,lib}.rs` (+ a new `wat_special_form.rs`), `src/runtime.rs` (metadata-of + lookup callers), `src/special_forms.rs` (crutch kill), `wat/runtime-meta.wat`. Do NOT touch the 1192-ref checker type table, the VSA core, or any `-> :T` work.

## STOP triggers (halt + surface; do not improvise)

1. If supporting a special-form mode in wat-doc would require a *structural rewrite* of the value `parse()` (not an additive sibling path) — STOP, surface the shape.
2. If `pure: bool → purity: Purity` cascades into the checker's type table or beyond the registry/reflect/macro surface — STOP, report the cascade; we may scope it separately.
3. If killing `special_forms.rs`'s `signature`/`build_registry` breaks a consumer you can't cleanly retire (grep `lookup_special_form` usages first) — STOP, list the consumers.
4. If `if`/`let` need a uniform `handler` to satisfy any existing dispatch assertion — STOP; they must NOT (dispatch stays inline). Report what demanded it.

## Done = the gate (EXPECTATIONS file). Do NOT report green without running every row yourself.
