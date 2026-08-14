# DESIGN — STONE 255.1c-time: HOME #2, the `:wat::time::` carve

Home #1 (`core::Bytes`, `7b99d123`) proved the template. This stone finds the **rhythm** — and it is
chosen to break something Bytes structurally could not.

## Why this family, and not the obvious ones

Measured this session (HEAD `d439393b`), `runtime.rs` dispatch-arm census by family — **561**
line-start string-literal arms total (⚠ a grep, not a census; an earlier count of the same surface
returned 678 by a different pattern — neither is authoritative, and the registry's own cascade is):

| family | arms | verdict |
|---|---|---|
| `core` (grab-bag) | 122 | not a family — no coherent home yet |
| `kernel` | 37 | effectful, entangled with the IO tier |
| `holon` | 36 | VSA surface, its own concerns |
| **`time`** | **41** | ★ **THIS ONE** |
| `core::i64` | 20 | ⛔ **the hot path — carve LAST**, see below |
| `core::f64` | 19 | adjacent to the hot path |
| `std` / `runtime` / `rete` / `edn` / `config` | 14/13/10/8/6 | small, but each entangled |

**`:wat::time::` is the clean cut, on four measured grounds:**

1. **CONTIGUOUS.** All 41 arms occupy `src/runtime.rs:5939–6016` — one unbroken block. The carve is
   a block deletion, not a scatter-gather.
2. **ALREADY DOWNSTREAM OF THE REGISTRY GUARD.** The registry dispatch arm is
   `runtime.rs:5608` (`h if crate::intrinsic::registry().lookup(h).is_some()`). Every `time` arm sits
   *after* it, so a registered `time` name is intercepted by the guard the instant it registers —
   **no dispatch reordering, no risk of shadowing.**
3. **COLD.** `:wat::core::i64::+` dispatches at `runtime.rs:5036` — **before** the guard. The hot
   arithmetic path never touches the registry today, and this stone does not change that. The perf
   gate the design demands is **not** this stone's risk; it becomes live only when `core::i64`
   carves, which is why `i64` goes last.
4. **★ IT STRADDLES THE DETERMINISM AXIS — the reason it is home #2 and not home #12.**

## The load-bearing reason: Bytes could not break the metadata contract

Every `core::Bytes` entry is `@Purity Pure` + `@Determinism Deterministic`. **A home whose every row
takes the same two values cannot falsify the metadata contract** — R59 `NISI FRANGAS, NIHIL PROBAS`:
a pass that nothing could break proves nothing. The registry's whole thesis is *declared* purity and
determinism as queryable truth; so far that thesis has been exercised on one corner of a 3×3 grid.

`:wat::time::` splits cleanly across it:

| rows | purity / determinism |
|---|---|
| `now` · `epoch-millis` · `epoch-nanos` · `epoch-seconds` · `*-ago` · `*-from-now` (≈19) | reads the clock — **`Nondeterministic`** |
| `+` · `-` · `at` · `at-millis` · `at-nanos` · `to-iso8601` · `from-iso8601` · `Day`/`Hour`/`Minute`/`Second`/`Millisecond`/`Microsecond`/`Nanosecond` · `days`/`hours`/`minutes`/… (≈22) | pure arithmetic on values — `Pure` + `Deterministic` |

This is the design's own worked example made real. `NOTE-purity-is-definition-time-queryable-metadata.md`
argues the two axes are orthogonal using `Uuid/v4` (pure ∧ non-deterministic) as the proof; **`time`
is the first home that would actually carry a non-deterministic row.**

## The composition, disconfirmed by reading the macro (not assumed)

Two claims this stone rests on, both checked against `crates/wat-macros/src/wat_intrinsic.rs` this
session:

1. **`@Purity Effectful` / `@Determinism Nondeterministic` are accepted.** Validated at `:248–257`
   (with the known-variant lists in the error text) and lowered at `:366–372`. **The macro will not
   reject a non-deterministic row.**
2. **Arity beyond `Exact(1)` works.** `:9–21` — N leading `&WatAST` params ⇒ `Exact(N)`; a single
   `&[WatAST]` leading param ⇒ `Variadic`, emitted at `:389–394`. Bytes only ever exercised
   `Exact(1)`; `time::+` (binary) and the `Day`-family constructors will exercise `Exact(2)` and
   `Exact(0)`/`Variadic`.

## The ONE contract decision, pinned

**A carved intrinsic's handler body is MOVED, not rewritten.** The arm body that exists in
`runtime.rs` today becomes the `#[wat_intrinsic]`-annotated fn body in `src/intrinsic/time.rs`,
byte-for-byte apart from the signature shim (positional `&WatAST` params replacing
`args[0]`/`args[1]` indexing, which is what the macro's arity check exists to make safe). **Any
behaviour change is out of scope and is a STOP.** The stone is a carve, not a cleanup.

## What is OUT OF SCOPE — affirmatively cut, not deferred

- **The blanket-accept** (`resolve/walk.rs:257`) — that is `255.1b-iv`, and it cannot land until
  enough homes are registered that the corpus survives it. Out of this stone's scope; tracked as the
  arc's own next-after-homes strike.
- **Un-ignoring the nine gates.** They unlock when the thing they gate is built. `time` is one home
  of many; `probe_undefined_builtin_resolves` stays ignored until `1b-iv`. Out of scope.
- **`core::i64` / `core::f64` and the perf gate.** Named above; last, under their own bench.
- **`rete/purity.rs`'s hand-list becoming a projection** — that is `255.3` (consumers collapse). But
  see STOP-3: a non-deterministic registered row may *diverge* from the hand-list, and that
  divergence is a finding worth surfacing even though fixing it is not this stone's job.

## Progress meter

Home #1 registered **6 production names**. This stone takes it to **≈47** and deletes ~78 lines of
`runtime.rs` dispatch block. `runtime.rs` measured 35,066 lines on 2026-08-14. The carve's honest
claim is not "the megafile shrank" — it is **"one more family is nameable, queryable, and reflectable,
and its arms are gone from the central match."**
