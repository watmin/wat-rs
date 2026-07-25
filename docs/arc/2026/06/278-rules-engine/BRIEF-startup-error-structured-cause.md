# BRIEF — annihilate the string-wrapped creation-time error carrier (EDN all the way down)

> ## ⛔ SUPERSEDED (2026-07-25) — EXECUTED, and its central PREMISE was FALSIFIED. Do not brief from this.
>
> **The work landed** (stone-1 rider, floor weighed `4163/4163` by the orchestrator's own `--release`): the `_ =>`
> String fallback in `emit_startup_error_structured_exit` is collapsed, `LociDiedError::StartupError`'s carrier is
> retyped `String` → `:wat::core::Error`, `message` is a derived accessor, and the acceptance gate
> (`cache_probe_startup_error_is_navigable_edn_not_string`) proves the cache-probe error decodes as a navigable
> typed tree — zero escaped-EDN-in-a-String.
>
> **§ The mask, bullet 3 (lines ~18-21) is WRONG** — *"This String typing is the coupling that forces the parent-side
> `edn_to_value` decode to expect a String — retyping it structured is what unblocks the whole thing."* The rider
> STOP'd on it and DISPROVED it: `reconstruct_enum_tagged` decodes variant fields **generically**, so the field's
> declared type was never the blocker. The REAL blockers were (a) STRICT `edn_to_value` hitting `UnknownTag` because
> **the Rust error types have no registered wat type**, and (b) **`:wat::core::Span` had no decode schema at all**
> (its `wat-reader` derive is write-only `ToEdn`) — so *no* error's `:location` could ever STRICT-decode. Kept visible,
> not deleted: this brief was written before the home arc was read, and the orchestrator asserted the coupling instead
> of grounding it. The rider's STOP is what corrected it.
>
> **The home arc is 296, not 278.** This is arc **296 stone D** (`docs/arc/2026/06/296-diagnostics-fully-edn/`
> `DESIGN-296-stone-D.md`) — D3 ("light them ablaze": error-chain fields `String` → nested round-trippable value) and
> its closing condition (a death envelope returns as nested EDN, zero strings-that-are-EDN). Read stone D +
> `DESIGN-296-derive.md` before continuing the campaign; brief from THOSE.
>
> **The one contested residue:** RuntimeError's 25/32 variants are registered by a HAND table
> (`register_runtime_error_variants`, `src/types.rs`) because `#[derive(Edn)]`'s `rust_type_to_wat_path`
> (`crates/wat-to-edn-derive/src/lib.rs:179-196`) maps only 6 scalars and hard-rejects every generic — a real,
> designed **STOP-2**. The hand table re-declares field structure `signal.rs`'s `#[to_edn]` attributes already
> declare (equality written twice → drift), which is the exact rot 296's `AUDIT-prose-in-errors.md` names as the root
> disease. **The derive enhancement is its own stone, owed BEFORE the remaining 9 enums.**

> Builder-ruled 2026-07-25: *"annihilate this — we are meant to be edn all the way down — masking it in a string
> is unacceptable."* The no-hidden-failures LAW (R41/R55/R57) reaching the **creation/startup** error path — the
> last place a structured error is `edn::write`'d into a `String` and hidden. Mirror the R57 destruction-side move
> (`251b43b3` — `Failure` carries a structured `:wat::core::Error`; `message`/`location` are DERIVED accessors).

## The mask (grounded)

- **`emit_startup_error_structured_exit`** (`src/process/verbs.rs:62`) forks: `StartupError::Macro` → structured
  EDN (`e.to_edn()` → `#wat.kernel.LociDiedError/StartupError [<tagged cause>]`); **every other variant** →
  `_ =>` fallback → `process_died_error_startup_value(e)` → **String carrier**. Collapse the fork: ALL variants go
  the structured route.
- **`process_died_error_startup_value`** (`src/runtime.rs:23000`) = `process_died_error_startup(to_wire_edn(e))`
  — `to_wire_edn(e)` is the String serialization; the field is `Value::String`. This is the mask. Siblings
  `process_died_error_main_signature_value` / `process_died_error_bad_return_value` (same file, adjacent) build
  `LociDiedError::{MainSignature,BadReturn}` with String fields the same way — **same class** (see scope note).
- **The field decl** (`src/types.rs`): the `LociDiedError::StartupError` variant field is `:wat::core::String`
  (~types.rs:1012 registration; the auto-gen `StartupError/message` accessor ~types.rs:1491). This String typing is
  the coupling that forces the parent-side `edn_to_value` decode to expect a String — retyping it structured is
  what unblocks the whole thing.
- **Structured path already exists**: `StartupError::to_edn_values()` (`src/freeze.rs:553`) maps every variant
  through `to_edn()` to real records. The Macro arm already proves the emission shape works.
- **`extract_panics`** (`src/runtime.rs:12633`) — clones the chain vec; unchanged by the retype (the items become
  structured, the vec-unwrap is agnostic).

## The fix (the R57 pattern, applied to creation-time carriers)

1. **Retype the carrier field** `String` → the structured cause (mirror `Failure`'s `error <- :wat::core::Error`):
   `LociDiedError::StartupError`'s field becomes the boxed error's structured `to_edn()` value (a tagged record),
   not a String. If a `message` accessor is still wanted, make it **derived** off the structured cause (the
   `eval_failure_message` precedent), never a stored String.
2. **`process_died_error_startup_value`** builds the field via the structured `to_edn()` value, not `to_wire_edn`.
3. **Collapse the emission fork** (`emit_startup_error_structured_exit`): delete the `_ =>` String fallback; ALL
   variants route through the structured path the `Macro` arm uses (`e.to_edn()` → `LociDiedError/StartupError
   [<cause>]`).
4. **Confirm the parent decode** (`edn_to_value` for the retyped field) accepts the tagged payload — the retype is
   what makes it valid (the comment's blocker resolves). Ground the exact decode site; adjust if needed.
5. **The R52 ablaze**: retyping the enum field screams every producer/consumer (the `StartupError/message` String
   accessor + any `.message`-as-String reader). Fix each to the structured/derived form. The checker enumerates
   the worklist — do not grep-guess it.

## Scope note (four-questions call — surface, don't silently decide)

`StartupError`'s cause is unambiguously a **structured error** `to_wire_edn`'d into a String → annihilate (clear).
`MainSignature`/`BadReturn` carry a **flat instructive message** (e.g. "main must be `[] -> :nil`") — a genuine
prose message may legitimately stay a String per R53 (*"if there is no good structured data, then an instructive
string"*), OR become a `FlatMessage` EDN record for uniformity. **Ground what each actually carries** (is it a
serialized structured error, or genuine prose?) and four-questions it — do not blanket-String-kill a legit message
nor leave a masked structured error. Report the call; default to structuring StartupError (the ruled target) and
surfacing the MainSignature/BadReturn decision.

## RED gate (write + run first; the gap is already demonstrated live — the `println` startup failure emitted `["#wat.runtime/UnknownFunction {…}"]`)

A **process-tier** startup failure whose emitted chain is inspected, asserting the `LociDiedError/StartupError`
cause is a **structured tagged record** (`#wat.runtime/UnknownFunction {…}` as EDN, navigable by field), NOT a
String. Assert on the STRUCTURE (`assert_edn_eq!` / field-extraction), never a `.contains()` on a Debug string
(`feedback_wat_stdio_is_edn_assert_structure`). Model on the existing crash-surface probes
(`probe_arc278_dead_child_speaks` / the crash-split measure). Fails RED now (String); green when the cause is
structured.

## Rooms (read in order)
1. `src/process/verbs.rs:39-98` (the emission fork — the `_ =>` fallback to collapse).
2. `src/runtime.rs:23000+` (`process_died_error_startup_value` + the `_main_signature`/`_bad_return` siblings).
3. `src/types.rs` ~:1000-1015 + ~:1484-1495 (the `LociDiedError::StartupError` field decl + `StartupError/message`).
4. `src/freeze.rs:542-560` (`to_edn_values` — the structured path to route through).
5. The `edn_to_value` decode of the `LociDiedError`/`StartupError` chain (grep; the parent-side reconstruction).
6. `251b43b3` / R57's `Failure`-carries-`Error` as the exemplar to mirror (`Failure/message` derived accessor).

## STOP triggers
1. If the parent-side `edn_to_value` decode has a genuine reason it CANNOT take a tagged payload even after the
   field retype (not just the declared String) — STOP, surface it (it would mean a deeper carrier constraint).
2. If `MainSignature`/`BadReturn` carry genuine prose (not a serialized structured error), do NOT blanket-kill —
   surface the four-questions call (§ scope note).
3. If `extract_panics` or a consumer relies on parsing the String back into structure — STOP (it shouldn't; grep first).

## Gate
The RED probe green + `cargo nextest run --release` at the known floor (weighed by the ORCHESTRATOR's OWN re-run,
Summary line, never a piped exit). Run everything FOREGROUND; do not background a command and return. Commit on
green (one atomic unit — the no-hidden-failures creation-carrier de-string-wrap).
