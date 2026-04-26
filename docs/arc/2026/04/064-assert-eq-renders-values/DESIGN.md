# Arc 064 — Self-explanatory assertion failures (rendered values + surfaced location)

**Status:** shipped 2026-04-26. See `INSCRIPTION.md` for the
canonical post-ship record. Implementation matched the DESIGN's
three-piece slice layout; one minor delta — `(:wat::core::show
(:wat::core::quote :outcome))` returns `<WatAST>` (the WatAST
summary form) rather than `:outcome`. Quote evaluates to a
`Value::wat__WatAST(Keyword …)`, not a `Value::wat__core__keyword`,
so the WatAST arm fires before the keyword arm. Rendering quoted
forms back to source text is left to a future arc; the keyword
literal in normal use (`Value::wat__core__keyword`) renders as
`:foo` correctly.

**Predecessor:** arc 060 (join-result) addressed the silent-crash
diagnostic gap by making thread death observable as data. Same
principle applies to assertion failures: the data needed to
diagnose the failure should be IN-BAND in the failure payload.

**Consumer:** experiment 009 T11 (proof-of-computation / PoW
kinship) failed in a deftest with three assertions; the substrate
returned only *"assert-eq failed"* — no information about WHICH
assertion fired or what the actual/expected values were. The proofs
session was left to bisect by splitting the deftest, which is the
wrong direction (smaller deftests don't fix the diagnostic gap;
they work around it).

Builder direction (2026-04-26, mid-T11 debugging):

> "hold on - undo - what diagnostic is missing - infra needs to
> address this - its not obvious to you what the failure is and we
> should make it be obvious"

The substrate's failure payload should carry the data needed to
diagnose the failure. Today `assert-eq` throws away that data at
the call site by passing `:None :None`. This arc closes that gap.

---

## What's already there (no change needed)

| Surface | Status |
|---------|--------|
| `:wat::kernel::assertion-failed!` (message + `Option<String>` actual + `Option<String>` expected) | shipped — already accepts the rendered values, just nobody passes them |
| `:wat::core::i64::to-string`, `f64::to-string`, `bool::to-string` | shipped — per-type renderers exist |
| `:wat::test::assert-contains` (passes haystack + needle to assertion-failed!) | shipped — proves the contract works when the values are renderable |
| `AssertionPayload.location` and `frames` (Span + Vec<FrameInfo>) | shipped (arc 016) — captured at panic time, NOT surfaced in the test runner output |
| `Span { file: Arc<String>, line: i64, col: i64 }` per WatAST node | shipped (arc 016 slice 1) — every form carries source position |
| `snapshot_call_stack()` populates frames from the call stack at panic | shipped (arc 016 slice 2) — newest-first, callee path + call_span per frame |

The infrastructure is in place at multiple layers. Two specific
gaps surface today:

1. The wat-side `assert-eq` (in `wat/std/test.wat:48`) passes
   `:None :None` for actual + expected — never renders.
2. The test runner's failure formatter doesn't reference
   `location` / `frames` — they're captured but not displayed.

## What's missing (this arc)

| Op / change | What it does |
|----|----|
| `:wat::core::show<T>` (new primitive) | `:T → :String` — polymorphic rendering, per-Value-variant dispatch |
| `:wat::test::assert-eq<T>` (reimplementation) | Calls `show` on actual + expected; passes `Some(...)` to `assertion-failed!` |
| `test_runner.rs::format_failure` (extension) | Reads `location` and (optionally) top-N `frames` from the Failure struct; appends to the formatted output |

Three changes. Two add a new primitive + a reimpl on top of it; the
third is a display fix in the test runner that surfaces existing
captured data.

The dispatch shape for `show` mirrors `:wat::core::=` — polymorphic
over T, implemented per-Value-variant in `runtime.rs`.

---

## Decisions to resolve

### Q1 — Naming: `show` vs `to-string<T>` vs `display` vs other

`show` is Haskell's name for the same primitive — render any value
to a debug-friendly String. Short, conventional in the FP tradition.

`to-string<T>` matches the substrate's existing per-type pattern
(`i64::to-string`, `f64::to-string`, `bool::to-string`) but reads
oddly when polymorphic — `(:wat::core::to-string<T> v)` says "this
is the polymorphic version" but doesn't visually distinguish from
the per-type ones.

`display` matches Rust's `Display` trait but is heavier as a verb.

**Recommended: `show`.** Single word, conventional, no overlap with
existing naming.

### Q2 — Renderable types: which Value variants does `show` cover?

V1 should cover the variants assertions commonly compare:
- `Value::bool` → `"true"` / `"false"`
- `Value::i64` → decimal (matching `i64::to-string`)
- `Value::f64` → standard (matching `f64::to-string`)
- `Value::String` → the string itself, optionally with quotes
- `Value::Option(...)` → `"Some(<show inner>)"` / `":None"`
- `Value::Result(...)` → `"Ok(<show inner>)"` / `"Err(<show inner>)"`
- `Value::Vec(xs)` → `"[<show x>, <show x>, ...]"` (some bound on size)
- `Value::Tuple(...)` → `"(<show item>, <show item>, ...)"`

Variants that DON'T render naturally to a short String:
- `Value::holon__HolonAST` → too large to render inline; render as `"<HolonAST cosine=... dim=...>"` or similar summary
- `Value::Vector` → render as `"<Vector dim=N bytes=M>"` summary
- `Value::Struct` → field-by-field render
- Channels, ProgramHandles → render as `"<Sender>"` etc.

**Recommended:** v1 covers all primitive variants + Option/Result/Vec/Tuple
recursively. Compound types (Struct, HolonAST, Vector, channels)
get a "summary" render that names the type and key dimensions but
doesn't dump the full content. If a consumer wants deeper rendering
for a specific type, a per-type `show` override can be added later.

### Q3 — Quoting strings

When `show` renders a String, should it quote it?

- `(:wat::core::show "hello")` → `"\"hello\""` (quoted)
- `(:wat::core::show "hello")` → `"hello"` (unquoted)

Quoted matches `Debug` semantics — distinguishes the empty string
from "actual nothingness" (`(show "")` → `"\"\""` vs `""`). Useful
for assertion failure messages where you want to see the boundary
clearly.

**Recommended: quoted.** Matches Rust's `{:?}` and Haskell's `show`.
The cost is two extra characters per string; the benefit is
unambiguous boundaries.

### Q4 — Where does `assert-eq`'s reimplementation live?

Two options:
- (a) Update `wat/std/test.wat` — change the existing macro to call `show`
- (b) Move `assert-eq` into the runtime as a primitive

**Recommended: (a).** Keep `assert-eq` as a wat-stdlib macro;
just make it call the new `show` primitive. Consistent with how
`assert-contains` and `assert-coincident` are implemented (wat-stdlib
on top of `assertion-failed!`).

### Q5 — Should `show` be exposed as a callable primitive, or only
used internally by `assert-eq`?

If it's a callable primitive:
- Test code can use it for diagnostic prints
- Future assertions (assert-not-eq, assert-true, assert-false) can use it
- Generally useful — Rust's `format!("{:?}", x)` is the equivalent

If it's internal only:
- Smaller surface
- Less to document
- But other assertions can't reuse it without re-exposing

**Recommended: exposed.** Public primitive at
`:wat::core::show`. Useful beyond assert-eq; aligns with how
`i64::to-string` etc. are exposed.

### Q6 — Source location info — IN SCOPE for this arc

The user explicitly directed this is required, not a later arc.
On audit, source-position propagation is ALREADY DONE:

- arc 016 slice 1: every `WatAST` carries a `Span`
- arc 016 slice 2: `snapshot_call_stack()` fills `AssertionPayload.frames`
- arc 016 populated `Failure.location` and `Failure.frames`

What's missing is the **display layer** in `test_runner.rs`. Today
the formatter reads `message`, `actual`, `expected` from the Failure
struct and ignores `location` / `frames` even though they're
populated. The fix:

```
failure: assert-eq failed
  at:       wat-tests-integ/.../explore-directed.wat:142:5
  actual:   false
  expected: true
```

If `RUST_BACKTRACE=1` is set (or the wat equivalent), append the
full frames stack newest-first per Rust's panic-hook convention.
Otherwise the top-frame `location` is enough for diagnostic.

**Recommended: include in this arc as Slice 3 of the same commit.**
Tiny change to `format_failure`; reads existing struct fields.

### Q7 — Frame display under RUST_BACKTRACE

Arc 016 already wires `RUST_BACKTRACE` for the runtime panic hook.
The test runner's failure formatter should parallel: short by
default (top-frame `at:` line); full stack when `RUST_BACKTRACE=1`.

**Recommended:** match arc 016's existing convention. Same env var,
same threshold. No new toggles.

---

## What ships

One slice, three pieces. Single commit.

- **Slice piece A — `:wat::core::show` primitive** in `runtime.rs`
  with per-Value dispatch; scheme registration in `check.rs`
- **Slice piece B — `:wat::test::assert-eq` reimplementation** in
  `wat/std/test.wat` using `show` to populate `assertion-failed!`'s
  actual + expected
- **Slice piece C — `test_runner.rs::format_failure` extension**
  reads `location` from the Failure struct and prepends an
  `at: file:line:col` line; appends `frames` newest-first when
  `RUST_BACKTRACE=1`
- **Tests** inline in `src/runtime.rs::mod tests`:
  - `show` for each primitive variant (bool, i64, f64, String)
  - `show` for Option/Result/Vec/Tuple recursively
  - `show` summary for HolonAST/Vector/Struct
  - `assert-eq` failure includes rendered actual + expected
  - `assert-eq` failure includes `at:` line with file:line
- `docs/USER-GUIDE.md` — add `show` to the surface table; update
  `assert-eq` row to note that failures now carry rendered values
  AND source location

Estimated effort: ~150 lines Rust (polymorphic dispatch) + ~30
lines tests + ~5 lines wat (test.wat update) + ~20 lines test
runner formatter + doc updates. Single commit. Slightly larger
than arcs 058/059/060/061/062/063 because of the per-variant
dispatch; still single-slice scope.

---

## Open questions

- **Pretty-print depth limit**: a deeply nested Vec<Vec<...>> could
  produce an enormous show output. Recommended cap: 1KB or some
  reasonable limit, with truncation indicator (`...`).
- **`show` for user-defined struct/enum variants**: per-type
  override mechanism. Out of scope for v1.
- **Custom `show` impl per consumer**: future arc could add a
  `defshow` form letting users override show for their domain types.
- **Surrounding form text in failure output**: the `Span` carries
  file/line/col but not the source text itself. A future arc could
  read the file at failure time and include the offending line for
  context (Rust does this via `--explain`). Out of scope for v1;
  the file:line is enough to navigate to the source.

## Slices

One slice. Single commit. Pattern matches arcs 058/059/060/061/062/063.

## Consumer follow-up

After this arc lands, experiment 009 T11's failure becomes
self-diagnostic. Instead of:

```
test exp::t11-... ... FAILED
  failure: assert-eq failed
```

The output becomes:

```
test exp::t11-... ... FAILED
  failure: assert-eq failed
    at:       wat-tests-integ/experiment/009-cryptographic-substrate/explore-directed.wat:347:5
    actual:   false
    expected: true
```

With `RUST_BACKTRACE=1`, also a frames stack newest-first.

The proofs session can immediately see (a) WHICH assertion fired
(file:line) and (b) what the actual/expected values were. Bisection
is no longer needed; the failure carries its own context. The
diagnostic story closes completely with this single arc.

## Connection to broader diagnostic story

The substrate's diagnostic story is being built incrementally:

- Arc 016 — failure location + frames captured into AssertionPayload
  (display side never landed for the test-runner output)
- Arc 060 — join-result (thread death as data)
- **Arc 064 (this)** — assert-eq failures render actual + expected
  values AND surface source location in test-runner output (closes
  arc 016's display-side gap as part of the same slice)
- Arc 065+ (future) — runtime errors include surrounding form
  context; per-type `defshow` overrides; pretty-print depth bounds

Each arc closes one specific gap. With this one, the assertion
failure → diagnostic context loop closes completely.
