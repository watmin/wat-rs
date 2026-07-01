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
