# 296.D1 — Embedded structured data travels as EDN, never prose (the down-payment the derive subsumes)

> **Status: STRIKE-READY (2026-06-30). Orchestrator designed + grounded; delegate the build to a sonnet; weigh
> forced-clean by `cargo nextest run --release` AND the captured wire EDN.**

## The constraint being applied
Every embedded structured datum in an error's EDN is a **`ToEdn` value or a native structured EDN** (a `Vector`, a
tagged map, an integer, a keyword) — **never** a `.join(…)` / `render_*() -> String` / `.to_string()` prose blob. The
`WatError` wall (S6, `ed5721ea`) forced the *floor* present; this forces the *embedded body* structural, one rung below
the full `#[derive(WatEdn)]` (which removes the hand-written serializer entirely — a SEPARATE later strike, see § The
rung above).

## Why now / what falls out
The audit (`296/AUDIT-prose-in-errors.md`, 10 findings / 9 L1) traced the class to ONE absent constraint: the error EDN
is not a structural function of the type, so a `Vec`/enum/count/coordinate can be flattened to prose field-by-field. The
enabler is that the embedded structured types (`Remedy`, `LoadFetchError`, `HashError`, the check-side clause-attempt
vec) **have no `ToEdn` form**, so the serializers reach for `render_remedies()` / `.to_string()` / `.join()`. Give those
types their structured form and route the sites through it — the missing impls are exactly "what falls out."

The runtime twin already proves the target shape: `runtime_error_edn.rs` emits `NoMatchingClause`'s `:called-args` as a
`Vector<snap_val>` and `:attempted-clauses` as `Vector<clause_attempt_to_edn>`. The **check twin drifted** — it drops
`attempted_clauses: _` and `.join(", ")`s `called_arg_types`. This strike makes check match runtime.

## The work (grounded sites — all read this session)
**New `ToEdn` impls (the "what falls out"):**
1. `impl ToEdn for Remedy` (`src/remedy/mod.rs`) → `#wat.kernel/Remedy {:form "…" :kind :typo|:retirement :score N
   :note "…"|nil}`. `:kind` is a keyword from `RemedyKind` (`Typo(_)`→`:typo`, `Retirement`→`:retirement`); `:score`
   is `self.score() as i64`; `:note` elides to `nil` when `None`. Add a `pub(crate) fn remedies_to_edn(&[Remedy]) ->
   OwnedValue` returning `OwnedValue::Vector` (`[]` when empty). `RemedyKind` is `pub(crate)` — the impl lives in the
   `remedy` module so it has access; expose `remedies_to_edn` for the serializer call sites.
2. `impl ToEdn for LoadFetchError` (`src/load.rs`) → `#wat.kernel/NotFound {:path "…"}` · `#wat.kernel/LoadOther
   {:path "…" :reason "…"}` · `#wat.kernel/OutOfScope {:path "…" :scope "…"}`.
3. `impl ToEdn for HashError` (`src/hash.rs`) → `#wat.kernel/<Variant> {…}` structurally per its 8 variants
   (all `String`/`usize`/`&'static str` fields → `str_val` / `int_val`; the tag is the variant name).

**Route the 10 smuggling sites through structure (kill the prose):**
| # | site | now | → |
|---|---|---|---|
| 1 | `check/error_edn.rs:324` `DefRestrictedCallerNotAllowed.:prefixes` | `prefixes.join(" ")` | `Vector<str_val>` |
| 2 | `check/error_edn.rs:339` `NoMatchingClauseAtCallSite.:called-arg-types` | `.join(", ")` | `Vector<str_val>` |
| 3 | `check/error_edn.rs:334` `NoMatchingClauseAtCallSite.attempted_clauses: _` | DROPPED | `:attempted-clauses Vector<…>` (mirror runtime `clause_attempt_to_edn`; read `check/error.rs` for the exact field type — audit says `Vec<(usize, Vec<String>)>` → `{:arity N :param-types [str …]}`) |
| 4 | `check/error_edn.rs:79-83` `ReturnTypeMismatch.:remedies` | `render_remedies()` blob | `:remedies (remedies_to_edn remedies)` |
| 5 | `check/error_edn.rs:98-102` `MalformedForm.:remedies` | same | same |
| 6 | `types/error.rs:370-375` `MalformedVariant.:remedies` | `render_remedies()` in inline `to_edn` | same |
| 7 | `runtime_error_edn.rs:138-139` `EvalVerificationFailed.:error` (HashError) | `format!` | `HashError.to_edn()` |
| 8 | `load.rs:305-360` `Fetch.:cause` (LoadFetchError) | `.to_string()` | `LoadFetchError.to_edn()` |
| 9 | `load.rs` `VerificationFailed.:cause` (HashError) | `.to_string()` | `HashError.to_edn()` |
| 10 (L2) | `runtime_error_edn.rs:214-221` `EdnCoerceMismatch.:path` | dot-notation String | `Vector` of segments (split on `.`, drop empties) — if the split is clean; else leave + note |

## Out of scope (affirmative cuts)
- **`Display` / `render_remedies` STAY** — they are the human face (`render_remedies` still renders the "did you mean"
  section for `Display`). This strike changes the **EDN wire face** only; do NOT delete `render_remedies`.
- **The full `#[derive(WatEdn)]`** — the top rung (removes the hand-written serializer so `.to_string()` has NO site).
  Needs its own attribute-vocabulary design (computed `:hint` fields via `collect_hints`; synthetic constant fields on
  unit variants like `BareLegacyUnitType`'s `:primitive ":()"`). NOT this strike; named as the next rung.
- **The `:kind` discriminant / span-key work (N3 per-phase namespaces)** — separate.

## The rung above (named, not built)
`#[proc_macro_derive(WatEdn)]` in `crates/wat-macros` — the serializer becomes derived; a field type lacking `ToEdn` is
a COMPILE error (not a silent `.to_string()`). Requires `#[wat_edn(hint_fn=…)]` + `#[wat_edn(field(k=v))]` attributes to
preserve the hand-written richness this strike keeps. That is where "a floorless-BODY error is unrepresentable" lands.

## Acceptance (the sonnet writes; the orchestrator re-runs + weighs the wire EDN)
- A co-located probe (`tests/diagnostics/`) asserting: `ReturnTypeMismatch` with remedies → `:remedies` is a
  `Vector` of `#wat.kernel/Remedy` tagged maps (NOT a `String`); `LoadError::Fetch` → `:cause` is `#wat.kernel/NotFound`
  (NOT a `String`); `NoMatchingClauseAtCallSite` → `:called-arg-types` is a `Vector` and `:attempted-clauses` is present.
  RED before the impls, GREEN after.
- **Round-trip + CLI tag tests stay green** (`crates/wat-cli/tests/wat_cli.rs` asserts `#wat.kernel/ReturnTypeMismatch`
  shape — the `:remedies` field changes String→Vector; update those assertions structurally).
- FULL gate `cargo nextest run --release` = 0 failed. `cargo build --release` clean.
- Orchestrator weigh: capture the emitted wire EDN for a `ReturnTypeMismatch`-with-remedies + a `LoadError::Fetch` and
  confirm ZERO smuggled prose (no `.join`/`did you mean`/`load: file not found` blob inside a `:String` field).
