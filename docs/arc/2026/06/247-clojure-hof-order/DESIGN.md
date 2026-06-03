# Arc 247 — Clojure-honest seq-HOF order (fn-first)

**Status:** OPEN 2026-06-03. Spawned from arc 237's wind-down — child of the generative-macro need (237.8c's equality wants a macro-generated defclause; the generator is a comprehension; the generative layer must stand on a `map` that tells the truth about the dialect). Spawn chain: **237 ⇠ (generative macros) ⇠ 247 (this — the Clojure-honest foundation).** Deepest dependency; built first.

**Implements arc 109 INVENTORY § N.1** — *"`:wat::core::map` arg order is backwards"* (`docs/arc/2026/04/109-kill-std/INVENTORY.md:1389`). Banked during 109's wind-down; the divergence surfaced at the **arc 232.0** research probe (sonnet wrote `(map xs fn)` and needed a separate note to remember the order — the natural reach is fn-first). N.1's target is exactly this arc's: `(map f xs)`, *"audit should sweep the family, not just map"* (map/filter/reduce/for-each). **Closing 247 resolves N.1** — mark it RESOLVED in 109's INVENTORY when this arc lands.

## Why — dialect honesty, not preference

The substrate **claims** clojure-on-rust (CLAUDE.md: "Clojure-faithful data literals; same family as Ruby-on-C / Clojure-on-Java"). Clojure's seq higher-order functions are **fn-first** — `(map f coll)`, `(filter pred coll)`, `(reduce f init coll)`, `(sort-by keyfn coll)` — used with thread-last `->>`. The substrate's are **coll-first** (`(map xs f)` …). That is the substrate **lying about what it is** — the same conformare ethos that drove arc 243 (be what you claim) and the nil/equality reckonings this session: an identity the code contradicts is a defect, full stop. The user's call: *"we are clojure on rust … we make map dialect compliant."*

This is NOT the coll-first-vs-threading *preference* debate (coll-first has real virtues — one `->`, internal consistency). The identity settles it: **we claim Clojure; Clojure's HOFs are fn-first; ours must be.** Preference loses to honesty.

## The flip set — 5 seq-HOFs, coll-first → fn-first

| op | now | Clojure target | runtime impl |
|---|---|---|---|
| `:wat::core::map` | `(map xs f)` | `(map f xs)` | `eval_vec_map` (runtime.rs:11300) |
| `:wat::core::filter` | `(filter xs pred)` | `(filter pred xs)` | `eval_vec_filter` (:11410) |
| `:wat::core::foldl` | `(foldl xs init f)` | `(foldl f init xs)` | `eval_vec_foldl` (:11336) |
| `:wat::core::foldr` | `(foldr xs init f)` | `(foldr f init xs)` | `eval_vec_foldr` (:11373) |
| `:wat::core::sort-by` | `(sort-by xs keyfn)` | `(sort-by keyfn xs)` | `eval_vec_sort_by` (:11225) |

Each flip touches three places: the **runtime impl** (arg-index extraction — `args[0]`/`args[1]`/`args[2]` swap), the **check-time inference** (the type scheme / custom arm for arg order + the `f`'s expected signature), and **every call site** (the cascade).

**Out of scope (already correct):** `apply` (`(apply f args)` — fn-first already). Collection ops `get`/`conj`/`assoc`/`length`/`empty?`/`contains?` stay **coll-first** — those match Clojure; do not touch them.

## The cascade

Flipping arg order breaks every call site: `(map xs f)` → `(map f xs)`, etc. Across `wat/` (core.wat, list.wat, stream.wat, Record.wat, holon/*), `wat-tests/`, `tests/`, `examples/`. Substrate-as-teacher: the type checker flags each mis-ordered call (the `f` arg now type-mismatches in the old position); migrate in turn. Predicted: moderate (HOFs are used, but not as pervasively as `i64::+`). The fold call sites in `core.wat:101-187` (the arithmetic defclauses' `(foldl rest seed f)`) are internal and flip first.

## The threading consequence (note, possibly a sibling need)

Clojure pairs fn-first seq-HOFs with **thread-last `->>`** (so the collection lands last and flows). Confirm whether the substrate has `->>`; if only `->` (thread-first) exists, fn-first HOFs won't thread ergonomically and `->>` becomes a near-term sibling need. **Scope check at strike**, do not bundle unless trivial — this arc is the arg-order flip.

## Slicing

**One stone** (247.0). The 5 flips + the cascade are coupled (the checker can't be green with a partial flip). Probe-locked, struck together. Predicted Mode-A: 40–70 min (cascade-bound).

## FM-2-bis probe

`tests/probe_arc247_hof_fn_first.rs` (to author). Gates:
- **fn-first works:** `(map (fn [x] (+ x 1)) [1 2 3])` → `[2 3 4]`; `(filter even? [1 2 3 4])` → `[2 4]`; `(foldl + 0 [1 2 3])` → `6`; `(sort-by f xs)` orders by `f`.
- **coll-first now errors:** `(map [1 2 3] (fn ...))` → check error (the `f` is no longer in arg2; the vector-in-arg1 / fn-in-arg2 mis-types). The old order is *gone*, not aliased (HARD CUT).
- **regression:** the arithmetic defclauses (which use `foldl`) still compute correctly after their internal foldl calls flip.

## Constraints

- Edits: `src/runtime.rs` (the 5 `eval_vec_*` impls), `src/check.rs` (their inference), `wat/*.wat` (call sites + the arithmetic defclauses' internal folds), cascade across `wat-tests/`/`tests/`/`examples/`.
- HARD CUT: the old coll-first order is **deleted**, not aliased. No `(map xs f)` compatibility shim.
- Green-gate: `cargo test --release --lib -p wat` + `cargo build --release --tests --workspace`, raw commands.
- No `holon-rs`. Do not touch the coll-first collection ops (get/conj/assoc — Clojure-correct).
- This unblocks the generative-macro arc (the comprehension stands on a Clojure-honest `map`), which unblocks 237.8c (equality as a macro-generated defclause).
