# BRIEF — STONE 255.1c-time: carve `:wat::time::` into home #2

Read `DESIGN-STONE-255.1c-time-home.md` first, then this. **Do not read `DESIGN.md` or
`BRIEF-STONE-255.1b-i-*` — the first is the arc's derivation and the second is WITHDRAWN.**

## THE WORK, in one paragraph

`src/intrinsic/bytes.rs` is home #1 and the exact template. Build `src/intrinsic/time.rs` the same
way: move each `:wat::time::*` handler out of `runtime.rs`'s dispatch block into an
`#[wat_intrinsic("<fqdn>")]`-annotated fn with the full doc-contract comment, and **delete the arm it
came from**. The registry guard already sitting above the block picks each one up the moment it
registers. **Handler bodies MOVE — they are not rewritten.** Floor held.

## ROOMS — read in this order

1. **`src/intrinsic/bytes.rs`** (159 lines, whole file) — **the template.** Note the shape exactly:
   the module doc-comment, the `@added`/`@Purity`/`@Determinism`/`@Category`/`@arg`/`@ret`/`@example`/
   `@see` block above each fn, the `#[wat_intrinsic(":wat::core::Bytes::to-hex")]` attribute, and
   the positional `&WatAST` params followed by the `env/sym/span` context tail. Copy this shape.
2. **`src/intrinsic/mod.rs:45–198`** — the enums your doc tags lower into: `Kind`, `DefinedIn`,
   `Layer`, `RuntimeCategory`, `RuntimePurity`, `RuntimeDeterminism`, `Arity`. **These exist. Do not
   mint new ones.** `mod.rs` is also where `time.rs` must be `mod`-declared, beside `bytes.rs` —
   without that declaration the `inventory::submit!`s never link and the registry stays empty.
3. **`src/runtime.rs:5939–6016`** — the 41 contiguous `:wat::time::*` arms. This is the block you
   empty. Each arm names the handler fn it calls; those fns are the bodies you move.
4. **`src/runtime.rs:5607–5611`** — the registry guard arm. **Read it to understand why you need do
   nothing here:** it already intercepts any registered name, and it sits *above* the `time` block.
   Do not edit it.
5. **`crates/wat-macros/src/wat_intrinsic.rs:9–21` and `:240–260`** — the arity sniffing rules and
   the `@Purity`/`@Determinism` validation. Read these when a handler is not `Exact(1)`: N leading
   `&WatAST` params ⇒ `Exact(N)`; one leading `&[WatAST]` ⇒ `Variadic`.

## THE DOC CONTRACT IS THE POINT — every row answers both axes honestly

The macro **requires** `@Purity` and `@Determinism` on every entry and rejects an unknown variant at
compile time. Known variants: `Pure | Effectful | Preserving` and
`Deterministic | Nondeterministic | Preserving`.

**This family splits, and getting the split right is the load-bearing work of the stone:**

- **Clock readers** — `now`, `epoch-millis`, `epoch-nanos`, `epoch-seconds`, and every `*-ago` /
  `*-from-now` — read ambient state. They are **`Nondeterministic`**.
- **Value arithmetic** — `+`, `-`, `at`, `at-millis`, `at-nanos`, `to-iso8601`, `from-iso8601`, the
  unit constructors (`Day`, `Hour`, `Minute`, `Second`, `Millisecond`, `Microsecond`, `Nanosecond`)
  and the unit-count verbs (`days`, `hours`, `minutes`, `seconds`, `milliseconds`, `microseconds`,
  `nanoseconds`) — are functions of their arguments. **`Pure` + `Deterministic`.**

Classify each of the 41 **from its actual moved body**, not from this list — the list is orientation,
the body is truth. Where the two disagree, the body wins and **say so in your report.**

`@Category` — reuse an existing `RuntimeCategory` variant if one fits; if none does, that is
**STOP-2** (see below), not a licence to invent one silently.

## IMPLEMENTATION SKETCH

```rust
//! :wat::time:: intrinsics — arc 255 home #2, carved to the
//! `#[wat_intrinsic]` form (255.1c-time). [.. module doc, mirroring bytes.rs ..]

/// Current wall-clock time as a `:wat::time::Instant`.
///
/// @added         1.0.0
/// @Purity        Pure          // ← or Effectful — decide from the body
/// @Determinism   Nondeterministic
/// @Category      <existing variant>
/// @ret     :wat::time::Instant the instant sampled at call time
/// @example-norun (:wat::time::now) #=> #inst "..."
#[wat_intrinsic(":wat::time::now")]
pub(crate) fn eval_time_now(
    env: &Environment, sym: &SymbolTable, span: &Span,
) -> Result<Value, EvalBreak> { /* the body, MOVED from runtime.rs */ }
```

Then in `src/intrinsic/mod.rs`, beside the `bytes` declaration: `mod time;`.
Then in `src/runtime.rs`, **delete the 41 arms** at `5939–6016`.

## BLAST RADIUS

`src/intrinsic/time.rs` (new) · `src/intrinsic/mod.rs` (one `mod` line) · `src/runtime.rs` (the
41-arm block deleted, plus the now-unused handler fns moved out). **No behaviour change. No new
enums. No edit to the registry guard, the resolver, the checker, or any `.wat`.**

## STOP TRIGGERS — each means ship nothing, report the gap

**STOP-1 — a handler body cannot move unchanged.** If a `time` handler reaches into something only
`dispatch_keyword_head_value`'s local scope has, the carve is not a move and the seam is wrong.
Report which handler and what it reaches for.

**STOP-2 — no existing `@Category` variant fits.** Report the handler and what it does. Do not add a
variant; the category set is a closed domain and widening it is a ruling, not an implementation
detail.

**STOP-3 — a registered `Nondeterministic` row contradicts an existing hand-list.** `src/rete/purity.rs`
is a hand-maintained `{pure, deterministic}` map (the design says it becomes a projection of the
registry in 255.3). If carving `time` makes the registry and that hand-list disagree about any name,
**report the disagreement** — do not edit `purity.rs` to match. That divergence is exactly what the
registry exists to expose, and it is the first time this arc could observe it.

**STOP-4 — the floor moves.** Any test that goes red is a finding, captured whole and verbatim per
the red protocol. Do not re-run to see if it clears.

## THE GATE

1. `cargo build --release` — exit 0.
2. `cargo clippy --release --all-targets` — **zero warnings**, and **no `#[allow(dead_code)]`**.
3. **The registry answers.** For at least one row from EACH side of the split, run the built binary
   and paste the actual output in your report:
   `(:wat::runtime::metadata-of :wat::time::now)` → `:determinism` must read `Nondeterministic`;
   `(:wat::runtime::metadata-of :wat::time::to-iso8601)` → `:determinism` must read `Deterministic`.
   **A stone where both sides report the same value has not done the thing it exists to do.**
4. `git diff --stat` — the `runtime.rs` deletion and the `time.rs` addition, and nothing else.
5. Floor: **not yours.** The orchestrator runs `scripts/floor.sh` centrally and weighs by its own
   re-run.

Run everything **foreground** and block on it. You are a rider, not the orchestrator: **ending your
turn ENDS you** — nothing wakes you, no notification is coming. Your turn ends when the numbers are
in your hands, not when a command is launched.

## A PRIOR RESULT TO COPY FOR SHAPE

`7b99d123` (255.1b-iii, home #1) — the same carve, one family smaller. And `0a32d5f8` / `851c0d37`
(251.8a / 8a-ii) for the reporting register: small diffs, honest deltas reported rather than
smoothed, and a STOP-adjacent judgment flagged at its real confidence instead of shipped quietly.
