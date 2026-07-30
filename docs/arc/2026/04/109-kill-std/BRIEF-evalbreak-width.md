# BRIEF — box `EvalBreak::Diagnostic`'s payload (kills 979 of 1640 `result_large_err`, 60%)

> **Status: DRAWN, NOT STARTED.** Drawn 2026-07-29, second stone of the clippy-to-zero campaign.
>
> **Home:** arc 109 (kill-std) owns the accumulated-cruft cleanup, same as its sibling
> `BRIEF-runtime-error-width.md`.
>
> ⚠ **This brief SUPERSEDES the measured claims of `BRIEF-runtime-error-width.md`.** That brief
> asserted `result_large_err` was "ONE fat type" (`RuntimeError`) and predicted three boxed fields
> would land it at 120. Both were wrong, and the first rider's STOP-2 report plus the orchestrator's
> own re-measurement proved it. The corrected map is below. **Do not use the older brief's
> arithmetic for anything.**

---

## The work, in one paragraph

`EvalBreak` is the eval loop's `Err` type, and it is **128 bytes** — exactly clippy's firing
threshold — because its `Diagnostic` variant holds a `RuntimeError` **inline**. That single inline
payload earns **979** `result_large_err` warnings, 60% of the entire clippy floor and more than any
other cause. Change `Diagnostic(RuntimeError)` to `Diagnostic(Box<RuntimeError>)` and `EvalBreak`
drops to **80 bytes** (measured, via a mirror enum) — under the threshold with 48 bytes of headroom.
`EvalBreak` derives **only `Debug`**, no `ToEdn`, so there is no EDN contract to preserve. Construction
already funnels through `impl From<RuntimeError> for EvalBreak`, so the change is one enum field, one
`From` body, 19 direct constructions and 11 patterns.

## The measured map — why this stone exists, and why it is FIRST

Every number below was measured this session by the orchestrator (clippy JSON histogram +
`size_of` probes), not predicted:

```
result_large_err, by which error type clippy names as too fat:
  979  value::signal::EvalBreak       ← THIS STONE
  482  value::signal::RuntimeError    ← its own stone, separately
  103  types::error::TypeError
   44  load::LoadError
   21  freeze::StartupError
   11  misc
 1640  TOTAL

widths:
  EvalBreak                        = 128   ← AT the threshold
  EvalBreak, if Diagnostic boxed   =  80   ← measured with a mirror enum
  EvalSignal                       =  80   ← the other variant; it sets the floor at 80
  RuntimeError                     = 128
  Span                             =  48
```

**clippy's boundary is `>= 128`, not `> 128`.** Grounded on our own tree, no throwaway crate needed:
`RuntimeError` is *exactly* 128 today and all 1640 warnings still fire. So a gate written `<= 128`
passes while nothing is fixed. **The target is `<= 120`.**

**Why boxing decouples, not just shrinks.** After this change `EvalBreak`'s width is set by
`EvalSignal` (80), *not* by `RuntimeError`. So `EvalBreak` can never re-breach the threshold because
some future `RuntimeErrorKind` variant got fatter. That independence is the point, beyond the 979.

**NOT MEASURED, and must not be claimed:** any throughput win. The stack-width argument is sound;
sound is not measured. **Do not put a perf claim in the commit message.** The grounded justification
is the warning floor plus the type telling the truth about its own size.

## THE ONE CONTRACT DECISION — box the PAYLOAD here, and why that is not a contradiction

**`Diagnostic(RuntimeError)` becomes `Diagnostic(Box<RuntimeError>)`.**

The sibling brief pinned "box FIELDS, never payloads." That rule was grounded in a real constraint:
the `ToEdn` derive has no flatten/transparent option (`crates/wat-to-edn-derive/src/lib.rs:347-409`),
so boxing a *derived* type's tuple payload would nest its structured EDN and break the goldens.

**That constraint does not bind here.** `EvalBreak` is `#[derive(Debug)]` only
(`src/value/signal.rs:68`) — it has no `ToEdn` impl, derived or hand-written, so there is no EDN shape
to preserve. Verify this yourself as STOP-0 below. The rule is not being waived; it does not apply.

## Read in order — the rooms

1. **`src/value/signal.rs:63-77`** — `EvalBreak`'s doc + definition. `Diagnostic(RuntimeError)` at
   `:72`. This is the one type-level edit.
2. **`src/value/signal.rs:68`** — the `#[derive(Debug)]` line. Confirm no `ToEdn` (STOP-0).
3. **`src/value/signal.rs:79`** — `impl From<RuntimeError> for EvalBreak`. **This is the funnel**:
   every `?`-propagation site goes through it, so its body carries most of the migration.
4. **`src/runtime.rs`** — 71 of the 85 `EvalBreak::Diagnostic` mentions. Most are `?`/`From`
   (untouched); the direct constructions and patterns are the work.
5. **`src/freeze.rs:723`** — `impl From<RuntimeError> for StartupError`, a neighbouring funnel. Read it
   for shape only; `StartupError` is out of scope.
6. **The remaining sites** — `src/freeze/env.rs` (2), `src/value/signal.rs` (2), `src/freeze.rs` (1),
   `tests/diagnostics/probe_arc243_stone7b_signal_split.rs` (4),
   `tests/value/probe_rational_C3_i64_overflow.rs` (3), `tests/value/probe_int_modrem.rs` (2).

Measured split across all 85 mentions: **19 direct constructions, 11 patterns**, the balance
`?`/`From`-mediated.

## Implementation sketch

```rust
// signal.rs:72 — the one type-level change
pub enum EvalBreak {
    /// A genuine runtime diagnostic … (keep the existing doc comment)
    ///
    /// Boxed (arc 109, BRIEF-evalbreak-width): an inline RuntimeError made
    /// EvalBreak 128 bytes — exactly clippy's result_large_err threshold —
    /// earning 979 warnings. Boxed, EvalBreak is 80 (set by EvalSignal), so
    /// its width no longer tracks RuntimeErrorKind's widest variant.
    Diagnostic(Box<RuntimeError>),
    Signal(EvalSignal),
}

// signal.rs:79 — the funnel absorbs every `?` site
impl From<RuntimeError> for EvalBreak {
    fn from(e: RuntimeError) -> Self { EvalBreak::Diagnostic(Box::new(e)) }
}
```

Direct constructions gain `Box::new(...)`. Patterns binding the payload (`Diagnostic(e)`) keep
working — `e` is now `Box<RuntimeError>`; field reads auto-deref, and a site that *moves* the
`RuntimeError` out needs `*e`. Prefer `.into()` at a construction site where it reads cleanly, since
the `From` impl is the funnel.

## The RED gate — write it FIRST, and it is a WALL

Add to the existing `tests/value/probe_runtime_error_width.rs` (created by the previous rider;
**leave its existing `RuntimeError` assertion alone** — that is the sibling stone's business):

```rust
#[test]
fn eval_break_stays_narrow() {
    // 979 clippy::result_large_err warnings — 60% of the whole floor — were this
    // one inline payload. clippy fires at >= 128 (grounded: RuntimeError is
    // exactly 128 today and all 1640 still fire), so 120 is the real ceiling.
    // MEASURED at 128 before this stone; 80 after (EvalSignal sets the floor).
    assert!(
        size_of::<EvalBreak>() <= 120,
        "EvalBreak is {} bytes; the eval hot path returns Result<Value(48), EvalBreak>, \
         and clippy::result_large_err fires at >= 128",
        size_of::<EvalBreak>()
    );
}
```

**It is RED today at 128 — verify that before changing any production code.** It measures the exact
quantity, so it cannot go vacuously green (R59 `NISI FRANGAS, NIHIL PROBAS`), and it catches the next
fat variant with no audit. If you find yourself writing `<= 128`, stop: that is the exact vacuous
gate this brief exists to correct.

## Blast radius — bounded

`src/value/signal.rs` (the variant + the `From` body), `src/runtime.rs`, `src/freeze.rs`,
`src/freeze/env.rs`, and 3 test files. **No function signature changes. No new types. No `#[allow]`
anywhere — if the answer needs an allow, the answer is wrong.** Do not touch `clippy.toml`; raising
`large-error-threshold` reaches zero by moving the goalpost and is rejected. Do not touch
`RuntimeErrorKind`'s variants or the uncommitted leaf-field boxes already in the tree — those belong
to the sibling stone.

## STOP triggers — REJECTION criteria. Ship nothing and report.

1. **STOP-0 (run this FIRST): `EvalBreak` must have no `ToEdn`.** Confirm by reading
   `src/value/signal.rs:68` and grepping the tree for a hand-written `impl ToEdn for EvalBreak`. If one
   exists, **STOP** — the payload-boxing contract above rests on its absence, and the whole approach
   needs re-drawing.
2. **STOP-1: `EvalBreak` does not reach `<= 120`.** Report the measured `size_of` for `EvalBreak`,
   `EvalSignal` and `RuntimeError`. Do not start boxing `Signal`'s payload or anything in
   `RuntimeErrorKind` on your own judgement.
3. **STOP-2: the structured error EDN moves.** `RuntimeError`'s EDN must be untouched by this stone.
   `tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` and
   `probe_arc237_stone4_rich_errors.rs` are the arbiters. Note honestly: many of their tests are
   `#[ignore = "296-recapture-pending"]` and already fail at HEAD for an unrelated stale-golden
   reason, so a filtered green run proves nothing. If you need to show the EDN is unmoved, capture the
   raw EDN before and after with `--run-ignored all` and diff it — and report which tests you used.
4. **STOP-3: the floor moves for a reason you cannot name.** The floor is `cargo nextest run --release`
   at **4184 passed / 262 skipped** (orchestrator's own run this session; 4183 plus the previous
   rider's new width probe). Adding your own gate test makes 4185. Any *other* change to the count is
   a STOP, not something to reconcile.

## Gates the rider runs

- `cargo build --release --all-targets` → no new warnings. One pre-existing `unused_comparisons` in
  `tests/value/probe_arc216_stone5a_value_hash.rs:347` is present at HEAD; it is not yours.
- The new RED gate: red before (128), green after (80).
- `cargo nextest run --release` → **4185 passed** (4184 + your gate). Read the ANSI-stripped
  **Summary** line by hand; never a piped exit code (`| tail` returns `tail`'s exit).
- The clippy count, by JSON — a bare `grep -c` on clippy's text output is unreliable because
  incremental caching suppresses re-emission for unchanged crates:
  ```
  touch src/value/signal.rs
  cargo clippy --release --workspace --all-targets --message-format=json \
    | grep -c '"code":"clippy::result_large_err"'
  ```
  Expect **1640 → 661**. Report the number you actually get.

## Expectations — written before the strike, so the result cannot move the goalposts

| what | how it is checked | expected |
|---|---|---|
| `EvalBreak` width | the new RED gate | 128 → **80** |
| `result_large_err` total | clippy JSON count | **1640 → 661** |
| `EvalBreak`-attributed warnings | clippy JSON, grouped by named type | **979 → 0** |
| `RuntimeError`-attributed warnings | same | **482, unchanged** (sibling stone) |
| other clippy lints | same histogram | unchanged at ~229 |
| structured error EDN | STOP-2's before/after raw capture | byte-identical |
| floor | `nextest --release` Summary | 4185 passed |
| signatures changed | `git diff` | **zero** |

**Runtime prediction:** 25–45 min. **Trap-door risk:** a site that *moves* the `RuntimeError` out of
the pattern rather than borrowing it — that needs `*e`, and the compiler will name each one exactly.

## Out of this stone's scope

- **`RuntimeError` itself (482).** Its own stone. The orchestrator's measurement found the real fix is
  structural: there is **no canonical constructor** — 1438 open `RuntimeError { … }` literals — which is
  precisely why a width change there is expensive. The correct sequence is `RuntimeError::new(span,
  kind)` first, then `kind: Box<RuntimeErrorKind>` at one site (measured: takes it 128 → **56**). That is
  a fleet, drawn separately.
- **`TypeError` (103, 152 bytes), `LoadError` (44, 160), `StartupError` (21, 160), misc (11).** Separate
  types, separate analyses, ~179 warnings that survive this stone and its sibling.
- **Arming the wall.** `.github/workflows/ci.yml:41-44` already says *"clippy is informational for now …
  tighten to `-- -D warnings` once the warning floor is driven to zero."* Arming that is what makes the
  zero permanent instead of a moment, and it is the campaign's last act.
