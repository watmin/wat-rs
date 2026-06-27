# 118.2 — the faithful HOF family: default lazy, opt into eager via `:wat::seq::*`

> **Status: STRIKE — DESIGN. Contract pinned 2026-06-27.** The deliverable arc 118 was reclaimed for: the full,
> familiar HOF family over the `Stream` foundation, *"exactly as a clojure dev expects them placed and used."*

## The one contract decision (pinned — `118/DESIGN.md`, four-questioned)

**Default LAZY; opt into eager via the `:wat::seq::*` namespace.** Builder: *"clojure's default behavior is lazy — we
assume this behavior — users must opt into eager — we break what we break and we fix what we must."*

- `:wat::core::map` / `filter` / `take` / `drop` / `concat` / … flip to **LAZY** — they return a `Stream`
  (= `clojure.core/map`). The bare `core::` name is the familiar default every Clojure hand reaches for.
- **Eager is opt-in via the namespace** (NOT a `v`-suffix — four-questioned: `mapv`/`filterv` is Clojure-INCOMPLETE,
  no `takev`/`dropv`/`concatv`, so it gives a uniform opt-in for only 2 verbs; `:wat::seq::<anything>` is ONE rule
  for the WHOLE family).
- The three namespaces, settled:
  - **`:wat::core::*`** — the familiar default; **aliases the lazy `:wat::stream::*` forms**.
  - **`:wat::stream::*`** — the lazy family + primitives (`cons`/`lazy`/`empty` + map/filter/take/…). Returns `Stream`.
  - **`:wat::seq::*`** — the eager family (uniform opt-in). Returns `Vector`.
- `first` / `rest` / `empty?` stay **polymorphic** in `:wat::core::*` (Vec | List | Stream).

## Implementation — the lazy HOFs are WAT over the foundation (Option C); the Rust eager intrinsics RETIRE

The current `:wat::core::map`/`filter`/`take`/`drop` are **eager Rust intrinsics** (`src/collection/transform.rs`,
Vec→Vec). They are **replaced** by:
- **Lazy `:wat::stream::*` HOFs in WAT** over the foundation — the clojure-canonical recursive shape, e.g.
  `(defn stream::map [f s] (stream/lazy (if (empty? s) (stream/empty) (stream/cons (f (first s)) (stream::map f (rest s))))))`.
  Pure wat, closures + recursion (Option C). Input is any **seqable** (Vec | List | Stream) via polymorphic `first`/`rest`.
- **`:wat::core::*` = defaliases to the `:wat::stream::*` forms** (the familiar default names).
- **`:wat::seq::*` eager HOFs in WAT** — force to a `Vector` (`(defn seq::map [f s] (into [] (stream::map f s)))` or an
  eager accumulate). The eager opt-in.

This removes Rust HOF intrinsics in favor of wat-over-primitives — the substrate-self-hosting direction.

## The roster

| verb(s) | `:wat::core::*` (default) | `:wat::stream::*` (lazy) | `:wat::seq::*` (eager) |
|---|---|---|---|
| `map` `filter` `take` `drop` `take-while` `drop-while` `concat` | lazy (alias → stream) | **canonical, lazy** → `Stream` | eager → `Vector` |
| `iterate` `repeat` `cycle` `unfold` `range` (∞-capable) | lazy (alias → stream) | **canonical, lazy** → `Stream` | — (∞ has no eager form) |
| `reduce` `foldl` `fold` `into` | eager (terminal — value) | — | `seq::reduce`/`fold` (exist) |
| `for-each` `doseq` | eager (effect, nil) | — | — |
| `doall` `dorun` | eager forcers (Stream → Vector / nil) | — | — |
| `first` `rest` `empty?` `count` `nth` | **polymorphic** (Vec\|List\|Stream) | — | — |

- **`reduce`/`for-each`/`fold` consume head-dropping / tail-recursive** — the only correct way to fold a single-pass
  stream. Consumer discipline is **structural, not policy** (no enforcement check — the rewind footgun isn't shipped).
- **`reduced`** — early-exit marker so a push consumer (`reduce`) can short-circuit (so `take`-then-`reduce` composes).
- **"kill the duplicate `reduce`"** (core vs the old list alias) lands HERE.

## The cascade (the build strategy) — flip, then ride the red

The flip makes `core::map` return a `Stream` where ~50 files expect a `Vector`. **The red-test cascade IS the
progress meter** (examinare); each failure names the next site. Fix each broken site by intent:
- needs the result eager (length / index / `conj` / re-traverse / passed where a Vector is required) → **`seq::map`**
  (or wrap `(into [] …)` / `doall`).
- just consumes once (feeds `reduce`/`for-each`/another lazy stage) → **leave it** (lazy is correct, often free).

Blast radius (grounded 2026-06-27): `core::map` 113 · `filter` 36 · `take` 21 · `drop` 62 · `concat` 76 occurrences,
~50 distinct files (`src/` + `wat/` + `wat-tests/` + `crates/` + `tests/` — the WHOLE workspace; the gate is the
meter, not a scoped grep — lesson banked twice this session).

## Decomposition (sub-strikes — depth-first, the gate stays the judge)

- **118.2a — the lazy `:wat::stream::*` HOF family + `:wat::core::` aliases + the eager `:wat::seq::*` family + forcers
  (`into`/`doall`/`dorun`/`for-each`) + `reduced`.** Retire the Rust eager `map`/`filter`/`take`/`drop` intrinsics.
  This is the FLIP — it turns the tree RED. Land it; do NOT chase the cascade green in the same sub-strike.
- **118.2b … N — the cascade-fix sweeps.** Drive the red set to 0, site by site, by intent (seq:: vs leave-lazy).
  Likely grouped by area (src/ wat/ · wat-tests/ · crates/ · tests/). The red-count is the meter.
- **118.2-Z — completion + close.** The ∞ generators (`iterate`/`unfold`/`repeat`/`cycle`) + `reductions`/`partition`/
  `interleave`/`zip`; intueri the family; annihilate any `:wat::list::`/eager-intrinsic remnant; gate 0; INSCRIPTION.

## Out of scope (rejected — named, not deferred-in-costume)
- **The imperative `stream/generate` yielder** — CEK-era additive follow-on (needs a reified continuation; `118/DESIGN.md`).
- **Memoization / rewind** — single-pass is the law; a rewind buffer is the user's to build, not core's.
- **Re-homing eager Vec ops that aren't HOFs** (`sort`/`reverse`/`nth`) — they're inherently eager, already in `core`, untouched.

## The RED probe (disconfirming — `118.2/probe`)
A laziness observation: `(:wat::core::map BOOM <src>)` where `BOOM` errors when applied, **never consumed**. Today
(eager Rust intrinsic) `core::map` applies `BOOM` to every element at construction → the program errors → RED. After
the flip (lazy) `BOOM` is applied to nothing → the program returns `nil` → GREEN. Commit the probe RED before 118.2a.

## Expectations (scorecard — fixed before the strike)
| # | what | command | expected |
|---|---|---|---|
| 1 | the RED probe flips GREEN | `cargo nextest run --release -E 'test(lazy_core_map_does_not_force_late_elements)'` (RED at HEAD — `DivisionByZero`; `tests/types/probe_arc118_2_lazy_map.rs`) | PASS after 118.2a |
| 2 | `core::map` returns a `Stream` | `(:wat::core::type (:wat::core::map inc [1 2 3]))` via `cargo wat` | `wat::stream::Stream` |
| 3 | eager opt-in materializes | `(:wat::core::type (:wat::seq::map inc [1 2 3]))` | `wat::core::Vector` |
| 4 | whole workspace green | `cargo nextest run --release` | floor 0 (after the full cascade) |

**Pairs:** `118/DESIGN.md` (the contract block + the seq/stream split + CEK-stability) · `src/stream/mod.rs` (the
foundation) · `src/collection/transform.rs` (the eager intrinsics being retired) · `wat/seq.wat` (the eager folds).
