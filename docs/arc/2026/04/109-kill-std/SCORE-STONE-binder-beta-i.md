# SCORE — arc 109, binder strike β-i: the two `defrecord` macros

Rider: one flight, ~9 min, no STOP fired. Every row re-run by the orchestrator's own hand.

| # | what | result |
|---|---|---|
| 1 | `(:wat::core::defrecord :user::Box :- [T] [item <- T])` | ✅ |
| 2 | ★★ **`T` is a VARIABLE** | ✅ `:item 42` → `{:item 42}` AND `:item "hi"` → `{:item "hi"}` |
| 3 | ★ old `<T>` spelling · plain non-parametric record | ✅ both unchanged |
| 4 | ★ `:wat::holon::defrecord`, both spellings | ✅ — the trap nothing else would catch |
| 5 | the 4 parametric records in `wat/` still load | ✅ floor green |
| 6 | a malformed `defrecord` still diagnoses | ⚠ **structured, but the message regressed** — see below |
| 7 | floor | ✅ **4855/4855, 0 FAIL, 71.1s** |
| 8 | clippy `-D warnings` | ✅ 0 |

Predicted 15–25 min; actual ~9. The shape was copied from a sibling that already worked.

## The rider's work

`~@binder` spliced first try. `binder` is `(reverse (rest (reverse (rest args))))` — pure ends-peeling,
**no counting anywhere** (STOP-3 clean; my own grep for `length|count|arity` matched two COMMENTS
describing the absence of counting, which is the third time today a pattern matched prose). The
kwargs companion is untouched, preserving `Record.wat`'s own *"params ride ONLY on the recordtype
decl"* invariant (STOP-2 clean).

## ⛔ THE RIDER CORRECTED MY BRIEF, AND IT WAS RIGHT

My brief told it: *"You MAY run `./target/release/wat --check` … it reflects the CURRENT binary,
which already has α."* **True for α, false for this stone.** `wat/Record.wat` is baked into the
binary by `include_str!` at RUST-compile time (`src/stdlib.rs:131`), so a stdlib `.wat` edit is
invisible to `--check` until the orchestrator rebuilds. The rider discovered this empirically —
edited, ran `--check`, got the OLD error plus `⚠ binary looks STALE` — and then traced correctness
by hand through `defstruct`'s precedent and the Rust source instead of guessing.

★ **A rider cannot test a stdlib `.wat` edit at all.** That belongs in every future macro brief.

## ⛔ AND I TRADED A BAD MESSAGE FOR A BROKEN ERROR CLASS

Trap-door 4 fired: retiring the fixed-arity signature deleted
*"macro :wat::core::defrecord expects 2 arguments; got 1"* and replaced it with
*"malformed :wat::core::rest form: cannot take rest of empty Vec"* — which names an internal
primitive at `wat/Record.wat`. EXPECTATIONS row 6 says the diagnostic must be REPLACED, not deleted,
so I "fixed" it with an `Option/expect` guard carrying a proper message.

**That broke it worse.** `Option/expect` in a macro body raises a `#wat.kernel/AssertionFailure` that
**panics the thread** rather than producing an error VALUE — so `expect_startup_err` had nothing to
inspect and the test failed differently. Reverted.

★ **I optimized the STRING and destroyed the SHAPE.** A structured error with poor prose can be
improved in place; a panic cannot be caught by the consumer at all. The test that caught it was not
asserting on the message — it was asserting that an error value comes back.

⚠ And the precedent explains itself: `defstruct`'s friendly `"defstruct: missing field-vector"` is
**dead code** — `(first args)` throws on empty before the `None` branch `Option/expect` guards can
ever be reached. The rider proved this empirically and mirrored it in good faith. Both facts, and
the two candidate real fixes, are filed at
`NOTE-a-macro-cannot-diagnose-with-option-expect.md`. **Row 6 is a recorded loss, not a papered one.**

## The golden that changed, and why it is not a blessing

`probe_two_arg_form_only_one_arg_errors` pinned `ArityMismatch`. The arity gate is gone BY DESIGN —
that is the stone. The test's intent (a one-arg form fails at expand time) is preserved; the class
moved to `ProgramBodyEvalFailed`. I verified the new value matches the chain predicted from the
first floor arm (`ProgramBodyEvalFailed → MacroEvalRuntimeFailed → MalformedForm`) BEFORE recording
it, and updated the two places still claiming `ArityMismatch` — the test comment and the fixture
header.

⚠ Self-inflicted, worth recording: my first fixture edit added TWO comment lines, which shifted the
form from line 2 to line 4 and invalidated the span I had just written into the golden. Rewritten
as one line. **A negative fixture's line numbers are load-bearing.**

## What β-i did NOT do

- **`defservice`** — β-ii. One macro, ~2400 lines, emitting 12 declarations; its single parametric
  site is `wat/cache.wat:195` (`lru-svc<K,V>`).
- **`defn`** — γ. Its macro is already variadic, so it is the smallest of the three.
- **Migrating the 7 parametric `defrecord` call sites** — the codemod's job; both spellings work.
