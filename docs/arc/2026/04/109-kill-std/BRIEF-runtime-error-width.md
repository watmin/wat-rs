# BRIEF — shrink `RuntimeError` under 128 bytes (kills 1640 `result_large_err` warnings)

> **Status: DRAWN, NOT STARTED.** Drawn 2026-07-29 at the builder's direction, immediately after
> arc 170 merged to `main`, as the first stone of the clippy-to-zero campaign
> (`clippy sweep 1/N`, `d33010c9`, took the non-`result_large_err` count 626 → 226).
>
> **Home:** arc 109 (kill-std) owns the accumulated-cruft cleanup. This is not an arc-170 residual —
> 170's INSCRIPTION records the clippy precondition as rescoped by the builder with this work named
> as the immediate next unit.

---

## The work, in one paragraph

`RuntimeError` is **160 bytes**. Clippy's `result_large_err` threshold is 128, so every function in
the tree-walking interpreter that returns `Result<_, RuntimeError>` earns a warning — **1640 of
them, 72% of the entire clippy floor.** They are not 1640 sites of debt: they are one fat type
reported once per signature. Box the three fat *fields* inside `RuntimeErrorKind` and the type drops
under the threshold, and all 1640 warnings go with it, **without touching a single function
signature.** Twenty construction sites.

## Why it is real, not lint appeasement — MEASURED, not argued

```
Value                             =  48   ← the SUCCESS payload on the hot path
RuntimeError                      = 160
Result<Value, RuntimeError>       = 160   ← entirely the error's width
Result<Value, Box<RuntimeError>>  =  48   ← i.e. the error would cost NOTHING
```

Every eval step returns a 160-byte `Result` to carry a 48-byte value: **112 bytes of dead stack
width on every return, including every success.** An enum is as wide as its widest variant, so
`DivisionByZero` — which carries no data at all — is also 160 bytes. Three variants out of 33 are
taxing the other thirty.

**NOT MEASURED, and must not be claimed:** whether this produces a visible throughput win. The
stack-width argument is sound; sound is not measured. If a number is wanted, the `deep-cascade`
bench is the instrument. **Do not put a perf claim in the commit message.** The justification that
IS grounded is the warning floor plus the type telling the truth about itself.

## Where the 160 comes from — measured per variant

`RuntimeError { span: Span, kind: RuntimeErrorKind }` = 48 + 112.

```
PostconditionFailed   112   ← sets the whole kind width  (signal.rs:371)
EdnCoerceMismatch      96                                (signal.rs:320)
NoMatchingClause       80                                (signal.rs:353)
UnknownField           72                                (signal.rs:335)
SandboxScopeLeak       72                                (signal.rs:285)
Span                   48
```

`PostconditionFailed` is 112 because it carries a **second `Span` by value** (48) alongside two
`String`s. `EdnCoerceMismatch` is 96 because it carries **four inline `String`s** (24 each).

**The span is not the whole trigger.** Measured: boxing `ensure_span` alone takes the variant
112 → 72, which promotes `EdnCoerceMismatch` and leaves `RuntimeError` at **144 — still over.** The
top three must all come down:

```
box ensure_span                     160 → 144   still over
+ shrink EdnCoerceMismatch          → 128       AT the line, fragile — one future field breaks it
+ shrink NoMatchingClause           → 120       under, with headroom   ← THE TARGET
```

## THE ONE CONTRACT DECISION — box FIELDS, never payloads

**Box individual fat fields. Every variant stays a named-field variant.**

This is pinned because the obvious alternative is **already disproven**: wrapping a payload as
`PostconditionFailed(Box<Payload>)` cannot work. The `ToEdn` derive has **no flatten/transparent
option** (grounded: `crates/wat-to-edn-derive/src/lib.rs:347-409` — a single-field tuple variant
*requires* `#[to_edn(key = "…")]` and emits `{:key {…}}`; multi-field tuple variants are a
`compile_error!`). A tuple variant would silently change the structured error EDN from flat keys to
a nested blob and red `probe_arc298_3_runtime_derive_identical`.

Field-level boxing keeps the derive emitting the same keys **by construction**, and the pattern
already exists in this very enum — `MacroExpansionFailed { cause: Box<MacroError> }` with
`#[to_edn(via = crate::to_edn::error_edn_of_boxed)]` at `signal.rs:228-232`. Copy that shape.

**Do NOT delete `ensure_span` to save the 48 bytes.** It looks dead — the Display arm discards it
(`ensure_span: _`, `signal.rs:598`) — but it is a live **machine coordinate**: the derive emits it as
`:ensure-span` and `probe_arc298_3_runtime_derive_identical` asserts it, per the documented
CONFORMARE multi-span convention (`signal.rs:107-114`: outer span = the actionable site, secondary =
a domain-named kind field). Deleting it loses a navigable coordinate and reds a test. Same for
`outer_define_span`, which the Display arm *does* read (`signal.rs:535-538`).

## Read in order — the rooms

1. **`src/value/signal.rs:102-105`** — `RuntimeError { span, kind }`. The 48 + 112.
2. **`src/value/signal.rs:107-114`** — the multi-span convention. Read this before touching a span
   field; it is why they exist.
3. **`src/value/signal.rs:228-232`** — `MacroExpansionFailed`'s `Box<MacroError>` + `via`. **This is
   the exemplar.** Every field you box follows it.
4. **`src/value/signal.rs:371-378`** — `PostconditionFailed`. Box `ensure_span: Span` → `Box<Span>`.
5. **`src/value/signal.rs:320-334`** — `EdnCoerceMismatch`, four `String`s. Bring it to ≤ 72.
6. **`src/value/signal.rs:353-362`** — `NoMatchingClause`, `String + usize + Vec + Vec`. Bring it to
   ≤ 72 (`Vec<T>` is 24; `Box<[T]>` is 16).
7. **`src/value/signal.rs:535, 550, 571, 598`** — the Display arms. Boxed fields deref transparently
   in `format!`, but the destructuring patterns may need `&**`.
8. **The 20 construction sites** — `tests/diagnostics/probe_arc237_stone4_rich_errors.rs` (9),
   `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` (4), `src/value/signal.rs` (3),
   `src/runtime.rs` (2), `tests/rete/probe_arc278_open_surface_dispatch.rs` (2).

## Implementation sketch

```rust
// signal.rs — the shape, per field. Mirror MacroExpansionFailed:228.
PostconditionFailed {
    defclause_name: String,
    clause_index: usize,
    ensure_expr_snapshot: String,
    returned_value: Box<ValueSnapshot>,
    ensure_span: Box<Span>,              // was Span — 48 -> 8
},
```

Construction sites gain `Box::new(...)` on that field and nothing else. Shrink
`EdnCoerceMismatch` and `NoMatchingClause` the same way, choosing per field (`Box<str>` is 16 vs
`String` 24; `Box<[T]>` is 16 vs `Vec<T>` 24; a static op name could be `&'static str` at 16 —
`TypeMismatch:107` already does this for `expected`).

## The RED gate — write it FIRST, and it is a WALL, not a lint

```rust
// tests/value/probe_runtime_error_width.rs
#[test]
fn runtime_error_stays_narrow() {
    // 1640 clippy::result_large_err warnings were ONE fat type. This is the wall that
    // keeps it honest: clippy's threshold is 128, and a future fat field reds this test
    // instead of quietly re-inflating the hot path. MEASURED at 160 before the fix.
    assert!(
        size_of::<RuntimeError>() <= 128,
        "RuntimeError is {} bytes; the eval hot path returns Result<Value(48), RuntimeError>, \
         so every byte over is dead stack width on every success",
        size_of::<RuntimeError>()
    );
}
```

**It is RED today at 160** — verify that before fixing anything. It is a genuine wall rather than a
gate that merely happens to pass: it measures the exact quantity, so it cannot go vacuously green
(R59 `NISI FRANGAS, NIHIL PROBAS`), and it catches the *next* fat field with no audit.

## Blast radius — bounded

`src/value/signal.rs` (the definitions + Display arms), `src/runtime.rs` (2 sites), and 4 test files
(15 sites). **No function signature changes. No new types. No `#[allow]` anywhere — if the answer
needs an allow, the answer is wrong.** Do not touch `clippy.toml`; raising
`large-error-threshold` reaches zero by moving the goalpost and is explicitly rejected.

## STOP triggers — REJECTION criteria. Ship nothing and report.

1. **STOP-1 (load-bearing): the structured EDN must be byte-identical.** If a boxed field changes
   what the `ToEdn` derive emits — a different key, a nesting level, a different rendering — **STOP.**
   `probe_arc298_3_runtime_derive_identical` and `probe_arc237_stone4_rich_errors` are the arbiters,
   and they are the contract. `error_edn_of_boxed` exists for exactly this; if it does not cover your
   case, report what is missing rather than changing the goldens to match new output.
2. **STOP-2: 128 is not reached.** If the top three come down and `RuntimeError` is still over,
   **STOP and report the measured widths** — do not start boxing a fourth, fifth, sixth variant on
   your own judgement. The target is ≤ 128 with headroom; landing exactly at 128 is also a STOP,
   because one future field breaks it.
3. **STOP-3: a field looks dead.** Do not delete any field to save bytes. `ensure_span` reads as
   dead in Display and is a live EDN coordinate. If you believe a field is genuinely unread, report
   it with its writer AND its readers; deletion is a separate decision.
4. **STOP-4: the floor moves for any reason you cannot name.** The floor is **4183/4183**
   (`cargo nextest run --release`). Any change to that count — up or down — is a STOP, not something
   to reconcile.

## Gates the rider runs

- `cargo build --release --all-targets` → 0 warnings (it is at 0 today; keep it).
- The new RED gate: red before, green after.
- `cargo nextest run --release` → **4183/4183**. Read the ANSI-stripped **Summary** line by hand;
  never a piped exit code.
- `cargo clippy --release --workspace --all-targets` → **`result_large_err` count is 0**, and the
  remaining total is ~226 (the other 17 lints, out of this stone's scope).

## Expectations — written before the strike, so the result cannot move the goalposts

| what | how it is checked | expected |
|---|---|---|
| `RuntimeError` width | the new RED gate | ≤ 128, ideally 120 |
| `result_large_err` | clippy JSON histogram | **1640 → 0** |
| other clippy lints | same histogram | unchanged at ~226 |
| structured error EDN | `probe_arc298_3` + `probe_arc237_stone4` | pass, goldens **untouched** |
| floor | `nextest --release` Summary | 4183/4183 |
| rustc warnings | `build --all-targets` | 0 |
| signatures changed | `git diff` | **zero** |

**Runtime prediction:** 20–40 min. **Trap-door risk:** STOP-1 — the derive's treatment of a boxed
field is the only genuinely unknown mechanism here; everything else is arithmetic.

## Out of this stone's scope

The remaining **226** warnings across 17 lints — 143 doc-comment formatting
(`doc_lazy_continuation`, `doc_overindented_list_items`; not machine-applicable because clippy will
not rewrite doc comments), 18 `mutable_key_type` (real judgement: a map keyed by a type with
interior mutability), 8 `ptr_arg`, 6 `only_used_in_recursion`, and the tail. Those are the next
stones of the same campaign, tracked on arc 109's board. Arc 109 also owns arming the wall once the
floor is zero: `.github/workflows/ci.yml:41-44` already says *"clippy is informational for now …
tighten to `-- -D warnings` once the warning floor is driven to zero."* **Arming that is what makes
the zero permanent instead of a moment** — and it is the point of the whole campaign.
