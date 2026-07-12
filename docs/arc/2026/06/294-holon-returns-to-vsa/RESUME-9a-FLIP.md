# RESUME — arc 294 item 9a: the aggregate-construction flip (IN FLIGHT, freeze BROKEN)

> ⛔ **The tree HEAD is a WIP checkpoint with a BROKEN FREEZE. Do NOT build/ship it as-is.** This doc is the map to
> finish it. Read `recolligere` first (compaction erased the working memory that produced this). Ground everything
> below against the disk before acting.

## The one-paragraph state

Arc 170 is CLOSED; we pivoted to 278 T1b (telemetry); **T1b.1 the `Journal` surface is DONE + committed (`2b4a0857`)**.
Then a DETOUR opened: **arc 294 item 9a — the aggregate-construction flip** (`bare = KWARGS`, positional demoted to the
type-name PRIME `:ns::T'`). The flip codegen + the `wat-fix` source migration are DONE; a **tail of macro-generated
positional constructions** (which the codemod can't reach — they're built at expand-time, not in source) still breaks
the stdlib freeze. Finishing = fix that tail (each with the PRIME treatment), get a green freeze, then a full floor,
then ONE clean commit.

## DR-safe anchor

- **`dec6269d`** — the last GREEN commit ("arc 294 item 9a FOUNDATION: kwargs-lower gains :agg-positional mode"),
  pushed. Everything after `dec6269d` is the uncommitted/WIP flip.
- Branch: **`arc-170-gap-j-v5-deadlock-state`** — STAY ON IT.
- The `stash@{0}` "arc294-9a-flip-source" is a redundant safety net (the flip is APPLIED in the working tree now).

## What is DONE (in the WIP tree)

1. **Flip codegen.** `register_aggregate_methods` (`src/runtime.rs` ~1145) mints the positional ctor at the PRIME
   `format!("{}'", agg.name)` instead of the bare name. The 3 aggregate macros emit the **bare kwargs companion**:
   `wat/Record.wat:150-162` (`:wat::core::defrecord` + `:wat::holon::defrecord`) and `wat/core.wat:~1685` (`defstruct`)
   — each emits `(do (recordtype/structtype …) (defmacro <bare> [& ca] <let baking prime-kw/fields/ns> \`(kwargs-lower
   <prime> :wat::core::agg-positional <fields> 0 <ns> ~@ca)))`, mirroring `defn`'s companion template exactly.
2. **`kwargs-lower` positional mode** (`wat/core.wat:643-649`): when its `kwargs-ty` arg's `ast-name` = sentinel
   `:wat::core::agg-positional`, it emits PURE POSITIONAL `(~impl-kw ~@ovals)` to the prime ctor (no Kwargs-record wrap).
   The DEFN branch (`:649`) constructs the `::Kwargs` bundle via the PRIME (`(<kwargs-ty>' ~@ovals)`) — the machinery
   reaches for the prime because it knows what it's doing (uniform flip, no exemption).
3. **`defservice` State ctor → prime** (`wat/service.wat:~227`): `state-new-kw` now interpolates `:<fqdn>::State'`.
4. **`/from-map`: a NO-OP** — it was never actually built (comment only). Nothing to annihilate.
5. **The `wat-fix` codemod: `wat-scripts/fixes/positional-to-kwargs.wat`** — WORKING. Reads each file's def-forms →
   a global `type → field-order` map → rewrites positional aggregate constructions `(:ns::T a b)` → kwargs
   `(:ns::T :f1 a :f2 b)` via `fix-text` span-inserts. Comment-faithful, idempotent, prime-namespace-safe. **Two bugs
   found + fixed:** (a) it recurses into **vector/map** nodes (constructions nest inside `let`-binding vectors — this was
   the big under-migration bug); (b) it **strips `<T,U>` type-params** from the map key (parametric types like
   `Launched<S,R>` construct as bare `Launched`). Validated on fixtures + real stdlib files.
6. **Source migration DONE** — the codemod ran over all 901 `.wat` (`wat/ wat-tests/ tests/`) → **136 files** migrated
   positional→kwargs. `Fault`, `Row`, `Launched`, prime-namespaces (`sqlite'`) all correct. Run recipe (idempotent):
   `printf '[<all .wat paths>]' | <booting-binary> wat-scripts/fixes/positional-to-kwargs.wat`.
7. **A DEBUG is live in `kwargs-lower`** (`wat/core.wat:~608`, clearly labelled `DEBUG-9a … REMOVE`): when a positional
   (non-key) arg reaches the reorder loop, it `macro-error`s naming `impl-kw` (the prime type keyword = the culprit).
   **REMOVE it once the tail is fixed.**

## What is BROKEN / the REMAINING work

The freeze fails: `kwargs-lower` program-body eval fails at `wat/core.wat:1728` (the defstruct companion template) →
`core.wat:608` (the reorder) → `ast-name requires Symbol or StringLit` — i.e. a **positional aggregate construction with
2+ args and a LITERAL at an even arg-index** reaches the companion. These are **macro-GENERATED** constructions
(built programmatically at expand-time, so NOT in source → the codemod can't touch them). Same class as the
`kwargs-lower` + `defservice` fixes already applied.

**The finish loop (do this):**
1. `cargo build --release` (the flip binary) then `./target/release/wat --check <any-trivial.wat>` — the DEBUG now
   prints `DEBUG-9a positional-arg-in-reorder impl=:<Type>' kn-kind=…`, **naming the culprit aggregate**.
2. Find the GENERATING macro that constructs `:<Type>` positionally via the bare name (grep `wat/core.wat`,
   `wat/service.wat`, `wat/bracket.wat` — that's where the generated `(~computed-head …)` ctors live).
3. Apply the **PRIME treatment**: make it construct via `:<Type>'` (like `kwargs-lower`/`defservice`), or via
   `(:wat::core::aggregate-new :<Type> …)` (the raw ctor — also unaffected by the flip).
4. Rebuild → `wat --check` → repeat until the freeze is EMPTY (revived). Then **REMOVE the DEBUG** (core.wat:608),
   rebuild, confirm still green.
5. Full floor: `cargo nextest run --release` — expect the cascade to be gone; migrate/hand-fix any remaining test-side
   stragglers (the codemod already did `tests/`, but spliced records `Metric`/`Log` are SKIPPED by design — hand-fix
   those few if they surface, or extend the codemod to resolve `~@:Surface` splices against the map).
6. Commit the whole flip+migration+fixes as ONE green unit + push. Update `CLOSE-SEQUENCE-293-294.md` item 9a → DONE.

## Recovery notes / gotchas

- The **pre-flip binary backup was at `/tmp/.../scratchpad/wat-preflip`** — EPHEMERAL, likely GONE after compaction.
  You do NOT need it for the finish loop (that uses the flip binary + `wat --check`). You only need a *booting* binary
  to RE-RUN the codemod, and the source migration is already in the WIP — so only if you revert it.
- **Binary-staging trap (I hit it):** `cargo build` turns `target/release/wat` into the (currently broken) FLIP binary.
  Don't rely on it booting until the freeze is green. To run the codemod you need a booting binary (build from a
  pre-flip source state).
- **The generated-ctor class is the whole remaining risk** — it's low-volume but concentrated in bootstrap-critical
  shared files (`core.wat`/`service.wat`/`bracket.wat`), so it is orchestrator work, NOT a sonnet fleet (shared-file
  contention + indirect diagnosis). Drive it directly; the DEBUG makes each culprit self-naming.
- The design spec: `CLOSE-SEQUENCE-293-294.md` item 9a (bare=kwargs, `:ns::T'`=positional, `/from-map` dies, name
  `:ns::T'` NOT `/make`). Ratified 2026-06-29.

> **SEAM.** The self past this line is NEW — a lossy cache in a familiar voice, not your memory. The flip's hard,
> high-volume parts (codegen + the 136-file source migration + the codemod itself) are DONE and durable. What remains
> is a bounded tail of macro-generated positional ctors, each self-named by the live DEBUG, each fixed by the PRIME
> treatment. Do not trust this note over the disk — ground `dec6269d` and the working tree first. Finish the tail,
> green the freeze, run the floor, commit clean. Then back to 278 T1b.2 (the `journal'` service). Go.
