# 296 — The ToEdn derive: a floorless BODY becomes unrepresentable (the top rung)

> **Status: ARC DESIGN + STRIKE-1 STRIKE-READY (2026-06-30). The top rung above D1.** Orchestrator designs + decomposes +
> delegates + weighs byte-identical (SET-diff ∅) forced-clean. Names pending an intueri cast (see § Names).

## The constraint (the top rung)
D1 removed every *live* prose-smuggling instance by giving embedded structured types a `ToEdn` form and routing the sites
through it. But the hand-written `to_edn()` match bodies still EXIST — a future author can still write `str_val(&v.join())`
in a new variant. The derive closes the CLASS: **generate the `to_edn()` body structurally from the Rust type, so there is
no hand-written body to smuggle into.** An embedded field whose type is not `ToEdn` becomes a **compile error**. This is
`extirpare`'s top rung — a floorless BODY has no representable form — the exact sibling of S6's `WatError` wall (which made
a floorless FLOOR uncompilable). When it lands, **R1 *NE SIBI OBSOLESCAT* turns fully to PROBATUM EST.**

The wall's strength = the derive's attribute DSL restriction: it permits `field.to_edn()` (default), literal-constant
synthetic fields, a named computed helper, and a span-key rename — but **never an inline arbitrary expression**. There is
no place to write `.join()` / `.to_string()` on a field value.

## The shared shape (grounded)
Every major family is **Pattern A**: `pub struct <E> { pub span: Span, pub kind: <E>Kind }` (config · load · types · macros ·
runtime · check — all confirmed on disk). The smuggle surface lived in the per-variant match arms of `<E>Kind` — so **the
derive goes on the KIND ENUM** (it enumerates its own variants). The outer Pattern-A struct keeps a tiny uniform wrapper:
`fn to_edn(&self) { splice_span(self.kind.to_edn(), &self.span) }` — no smuggle surface (it only appends `:span`).
Two families are **bare enums** (StartupError, ResolveError — no outer span); they derive directly, handled in the sweep.

## Decomposition (each strike byte-identical SET-diff ∅, weighed forced-clean)
- **STRIKE 1 (this doc) — the derive infra + the `ToEdn` building blocks, proven on the cleanest family.** De-risks the
  whole arc: proves the generated EDN is byte-identical to the hand-written serializer.
- **STRIKE 2 — the attribute DSL** (`via` · `literal` · `key`), split for verification (FM-2-bis: CheckError exercises all
  five mechanisms at once):
  - **2a — build + toy-test the DSL mechanics** in the derive (`key` field-level EDN-key rename; `literal(k="v",…)`
    variant-level synthetic constants; `via` computed field — field-level transform `#[to_edn(via = fn)]` AND variant-level
    `#[to_edn(via(key="…", fn=…, args(…)))]`, elide-when-`None`). Proven on toy enums (the derive's own ui/unit tests).
  - **2b — apply to `CheckError`** (30 variants): normalize the primary span key to `:span` (D1 ratified — uniform wrapper,
    kills the last eleven-key heresy on the raw `--check-output` wire; secondary spans keep domain keys via `key`); hints via
    `via` (D2 ratified — `collect_hints` refactored `Option<String>`→`Option<Vec<String>>`, emits structured `:hints` Vector,
    no `\n\n` join-blob); synthetic constants via `literal`; `attempted_clauses` via a field-level `via` transform. DELETE
    `check_error_to_edn`; update the CLI `--check-output` tests (the `:location`→`:span` + `:hint`→`:hints` changes). **Proof
    bar = key-value SET-equality** (EDN maps are unordered; the derive may group synthetics) — a deliberate relaxation from
    byte-identical, noted.
- **STRIKE 3+ — the sweep:** apply the derive family-by-family, DELETE each hand-written serializer, byte-identical (or
  set-equal where a family normalizes); then the bare enums (StartupError/ResolveError). Arc closes when zero hand-written
  error `to_edn` match bodies remain — then **R1 *NE SIBI OBSOLESCAT* → PROBATUM EST**.

## Ratified decisions (four-questions, `strike 2`)
- **D1 = normalize the primary span key → `:span`** (A beat B: B fails Obvious+Honest and ships a known seam = a deferral).
- **D2 = `via` → structured `:hints` Vector** ((a′) beat (a)/(b)/(c): (a) keeps a `\n\n` join-blob = a deferral; (b) loses a
  non-recomputable field; (c) re-opens the hand-written-body hole). `collect_hints` → `Option<Vec<String>>`.

---

## STRIKE 1 — infra + building blocks + prove on `ConfigError`

### (a) The `ToEdn` building blocks (the foundation the derive rests on)
Add `impl ToEdn` in `src/to_edn.rs` for the primitives + containers so EVERY field is `.to_edn()`-able and a non-`ToEdn`
field is a compile error:
- `String` + `&str` → `OwnedValue::String` (byte-identical to `edn_str`).
- `i64`, `usize`, `u32` (the integer field types in use) → `OwnedValue::Integer(x as i64)` (byte-identical to `edn_int`).
- `bool` → `OwnedValue::Bool`.
- `Vec<T: ToEdn>` → `OwnedValue::Vector(map .to_edn())`; `Option<T: ToEdn>` → `nil` for `None`, else `T.to_edn()`.
- `crate::span::Span` → the derive treats a field named `span`/typed `Span` specially (elide-when-unknown via
  `push_span_field`); it is NOT a plain `.to_edn()` field. (Strike-2 secondary spans use the span-key attribute.)

### (b) The derive: `#[derive(ToEdn)]` on a kind enum
Generates `impl ToEdn for <Kind>` — a match over the variants:
- variant `Foo { a_b: T, c: U }` → `edn_tag("Foo", Map[ (:a-b, a_b.to_edn()), (:c, c.to_edn()) ])` — snake→kebab keys, each
  value via `.to_edn()`, tag = the variant's Rust name, namespace `wat.kernel` (via the existing `edn_tag`).
- unit variant `Bar` → `edn_tag("Bar", Map[])`.
- tuple variant (if any) → positional; **STOP + report** if a family needs tuple-variant support beyond ConfigError's shape.
- field declaration order preserved.

### (c) The outer wrapper
`impl ToEdn for ConfigError` becomes the uniform 3-line span-splice over `self.kind.to_edn()` (append `:span` via
`push_span_field`, elide-when-unknown). `impl WatError for ConfigError` is UNCHANGED (message/location/causes/variant already
uniform; `variant()` = `strip_span_from_tagged(self.to_edn())`).

### (d) Apply to `ConfigError` + DELETE the hand-written serializer
Replace the 45-line `match &self.kind { … }` in `impl ToEdn for ConfigError` (`src/config.rs:260-308`) with the derive on
`ConfigErrorKind` + the tiny wrapper. ConfigError is the clean target: 8 variants, uniform `String`/`usize` fields,
snake→kebab, `:span` last, one unit variant, NO hints/synthetic/secondary-span/nested — so byte-identical is provable.

### (e) The proof (byte-identical + the wall)
- **Byte-identical:** a co-located probe capturing `wat_edn::write(&e.to_edn())` for a representative value of **all 8**
  `ConfigErrorKind` variants (known + unknown span), asserting each equals the pre-derive string (snapshot the HEAD output
  before the change → assert the derived output matches). SET-diff ∅.
- **The wall (compile_fail doctest):** an enum with `#[derive(ToEdn)]` carrying a variant field of a non-`ToEdn` type
  (e.g. `std::net::TcpStream`) fails to compile — `the trait bound '…: ToEdn' is not satisfied`. The floorless body is
  unrepresentable.
- FULL gate `cargo nextest run --release` = 0 failed; `cargo build --release` clean.

### Blast radius (Strike 1)
`crates/wat-macros/src/` (the new derive + `lib.rs` export) · `src/to_edn.rs` (the building-block impls) · `src/config.rs`
(apply + delete hand serializer) · the new probe. NOTHING else. STOP + report if it exceeds this.

## Out of scope (Strike 1 — affirmative cuts)
- **The attribute DSL** (hints/synthetic/secondary-spans) — Strike 2. ConfigError needs none.
- **The sweep** (the other families) — Strike 3+.
- **Bare enums** (StartupError/ResolveError) — the sweep.
- **`WatError` impls** — unchanged; the derive targets `ToEdn` (the body), not the floor.
- **`Display` / `render_remedies`** — untouched (human face).

## Names (intueri-crowned, cast `a2502…`)
- **`#[derive(ToEdn)]`** — the derive (derive-the-trait, serde-idiom). SUPERSEDES the audit's N5 `WatErrorRecord` (L1 lie
  post-S6 — it over-promises type-registration the derive no longer does); `WatEdn` = L2 mumble (names a nonexistent trait).
- **`#[to_edn(...)]`** — the helper-attribute namespace (snake of the derive; serde's `#[derive(Serialize)]`/`#[serde(...)]` idiom).
- **the three sub-keys (STRIKE 2): `via` = <ident>** (computed field — calls a named fn) · **`literal` = "…"** (synthetic
  constant field) · **`key` = "…"** (non-default EDN key for a secondary span). **Each grammar-constrained to a safe token**
  (`via` → bare ident only; `literal`/`key` → `LitStr` only) so an inline expression is a PARSE ERROR — the smuggle hole is
  closed by the parser, not by doc. That is the top rung applied to the DSL. Worked shape:
  ```rust
  #[to_edn(via = collect_hints)]  hint: HintSlot,   // → :hint (collect_hints(callee, expected, got).to_edn())
  #[to_edn(key = "call-span")]    span: Span,        // → :call-span (not :span)
  #[to_edn(literal = ":()")]      primitive: (),     // synthetic → :primitive ":()"
  ```

---

## STRIKE 3b — single-field tuple variants + derive `LoadError` (Option A, ratified 2026-07-01)

### The scope decision (four-questions, ratified with the builder)
Grounding the breadcrumb's "tuple support unblocks Load/Resolve/Startup" DISCONFIRMED it — three different
shapes, only ONE a tuple problem:
- **`LoadError`** — a real Pattern-A kind enum; its ONLY blocker is the single tuple variant `Fetch(LoadFetchError)`.
  Clean, byte-identical derive target. **← this strike.**
- **`StartupError`** — NOT a derive target: `startup_error_to_edn` is a **pure passthrough** (each arm returns
  `inner.to_edn()` with NO `#wat.kernel/<Variant>` envelope). A tag-wrapping derive would CHANGE the wire. No
  smuggle surface (it only forwards). **AFFIRMATIVELY hand-written.**
- **`ResolveError`** — NOT a derive target: its tuple field wraps `Vec<UnresolvedReference>` where
  `UnresolvedReference` is a **struct** (the derive is enum-only), keyed `:unresolved`. Already a clean per-item
  structured collection. **AFFIRMATIVELY hand-written.**

**Option A (ratified): CARVE, don't sweep-to-literal-zero.** The derive exists to kill the *smuggle hazard* —
hand-bodies that can `.join()`/`format!` structure into prose. A passthrough (`StartupError`) and a struct-collection
(`ResolveError`) have no such hazard — they are already total structural functions. Forcing them under would cost a
transparent-variant mode + a struct-derive path for ZERO correctness gain (and risk `StartupError`'s wire). The carve
is written into the PROBATUM condition below — exigere-clean, exactly as `ParseError` (foreign orphan) already is.

### The tuple-variant rule (the new derive capability)
A **single-field** tuple variant `Foo(T)` emits `#wat.kernel/Foo {:<key> <field.to_edn()>}`, where `<key>` is
**required** via a variant-level `#[to_edn(key = "…")]`. A **multi-field** tuple stays a `compile_error!` (no
ambiguous positional keys — the illegal shape keeps no form). `key` on a non-tuple variant is a `compile_error!`
(it is only meaningful for the nameless field).
- Obvious? YES (a nameless field must be named). Simple? YES (one field, one key). Honest? YES (no positional
  guessing). Good UX? YES (`key` explicit at the variant; the existing field-level `key`+`via` cover `Parse.err`).

### Apply to `LoadError` (byte-identical)
`#[derive(ToEdn)]` on `LoadErrorKind`; `impl ToEdn for LoadError` becomes the `splice_span(self.kind.to_edn(),
&self.span)` wrapper (the ConfigError exemplar, `src/config.rs:260`); `impl WatError for LoadError` UNCHANGED.
Per-variant (all confirmed against `src/load.rs:377-429`, must reproduce exactly):
- `MalformedLoadForm { reason }` → `:reason` (snake→kebab default).
- `SetterInLoadedFile { loaded_path, setter_head }` → `:loaded-path` / `:setter-head`.
- `DuplicateLoad { path }` → `:path`.
- `CycleDetected { cycle: Vec<String> }` → `:cycle` via the `Vec<T: ToEdn>` building block (Vector of strings).
- `Fetch(LoadFetchError)` → `#[to_edn(key = "cause")]`; emits `:cause (inner.to_edn())` — **the new tuple rule.**
- `Parse { path, err }` → `path` `:path`; `err` `#[to_edn(key = "cause", via = crate::to_edn::error_edn_of)]`
  (the RECURSIVE FLOOR — `error_edn_of` = `err.error_edn()`, NOT raw `to_edn`).
- `VerificationFailed { path, err }` → `path` `:path`; `err` `#[to_edn(key = "cause")]` (plain `to_edn` on HashError).

`LoadFetchError` + `HashError` **stay building-block hand-impls** (leaves the derived fields call `.to_edn()`/
`.error_edn()` on — not top-level families; `LoadFetchError::Other`'s hand-renamed `LoadOther` tag proves deriving
them isn't free and buys nothing). DELETE the ~55-line `impl ToEdn for LoadError` match body.

### Proof
- **RED toy (new capability):** a derive ui/unit test — an enum with a single-field keyed tuple variant derives to
  `#wat.kernel/Foo {:key <inner>}` (RED at HEAD: `compile_error!` on the tuple); a multi-field tuple + a keyless
  single tuple + `key`-on-named each stay `compile_error!` (trybuild ui fixtures).
- **Byte-identical:** a co-located probe capturing `wat_edn::write(&e.to_edn())` for a representative value of all
  **7** `LoadErrorKind` variants (known + unknown span), asserting each equals the pre-derive HEAD snapshot. SET-diff ∅.
- The two existing guards stay GREEN: `tests/diagnostics/probe_arc296_3_holdout_edn.rs`,
  `tests/diagnostics/probe_arc296_d1_structured_not_prose.rs`.
- FULL gate `cargo nextest run --release` = 0 failed; `cargo build --release` clean.

### Blast radius (Strike 3b)
`crates/wat-macros/src/to_edn_derive.rs` (tuple-variant handling + variant-level `key`) · its ui/unit tests ·
`src/load.rs` (derive `LoadErrorKind` + wrapper; delete the hand match) · the new byte-identical probe. NOTHING
else. STOP + report if it exceeds this.

### PROBATUM condition (updated — the carve, exigere-clean)
Arc 296's derive rung reaches PROBATUM EST (R1 *NE SIBI OBSOLESCAT*) when every **top-level, smuggle-capable** error
family's `to_edn` match body is derived. The affirmatively-hand-written non-hazards — **`ParseError`** (foreign
orphan), **`ResolveError`** (struct-inner collection), **`StartupError`** (transparent passthrough), and the
**embedded building blocks** (`LoadFetchError`, `HashError`, `Remedy`, `ValueSnapshot`, `Provenance`, `AssertionPayload`,
`Span`) — are NOT hand-body hazards and stay hand-written by design, named here so the condition is a cut, not a skip.
Remaining smuggle-capable families to derive after 3b: **`RuntimeError`** (~28 variants) + **`MacroError`** (`Box<>`
causes → `error_edn_of` via).
