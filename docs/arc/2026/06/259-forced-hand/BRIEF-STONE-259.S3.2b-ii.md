# BRIEF — Stone 259.S3.2b-ii — the coordinator + `brackets/map`

## The work (one paragraph)

Write the brackets pool coordinator in `wat/bracket.wat` (after the existing
`runner-loop`): two top-level wat defns, `:wat::bracket::collect-loop<I,O>` and
`:wat::bracket::map<I,O>`, that make a bounded, dynamically-balanced worker pool over
`spawn-program'` + the runner-loop, returning results in INPUT ORDER. The full design,
the grounded type-threading, the (1)–(5) algorithm, and the exact implementation
skeleton are in `docs/arc/2026/06/259-forced-hand/DESIGN-STONE-259.S3.2b-ii.md` —
**read it whole first**. Pure wat; no Rust changes.

## CWD discipline (FIRST, every git/build)
- Anchor `/home/watmin/work/holon/wat-rs`. Run `pwd` first. Any `.claude/worktrees/` path
  is harness state — ignore it; operate in the anchor. Do NOT commit; the orchestrator weighs and commits.

## Rooms (read in order)
1. `docs/arc/2026/06/259-forced-hand/DESIGN-STONE-259.S3.2b-ii.md` — the design + the
   exact skeleton you fill (the shape is FIXED; you fill bodies + resolve the 3 watch-points).
2. `wat/bracket.wat` — the file; `runner-loop` (lines 20-26) is your anchor. Add the two
   new defns after it.
3. `tests/nursery/probe_arc259_brackets_map.rs` — the committed gate (READ; do NOT edit).
   It calls `(:wat::bracket::map (:wat::spawn::thread) items work-fn)`.
4. `wat/spawn.wat:64-73` — the `spawn-program'` defclause (thread clause type:
   `[Peer'<S,R>]->nil → Thread'<R,S>`). 5. `wat/core.wat:342-359` — `sort-by` 2-arity `(keyfn coll)`.

## The 3 watch-points (resolve against the checker — these have known fallbacks)
1. Tuple type spelling in annotations: `:(wat::core::i64,I)` (the select probe uses
   `-> :(wat::core::i64,wat::core::i64)`). If a type-var inside a tuple type won't parse,
   that is a real finding — surface it (do not work around silently).
2. The empty `Vector<(i64,O)>` seed for `pairs-acc` — spell it as an empty typed vector is
   spelled elsewhere; if the tuple-element type-arg fights, fall back to let-destructure.
3. `first`/`second` on a Tuple — `wat/stream.wat:88-89` reads tuples this way directly (no
   `Option/expect`). If the checker disagrees, use let-destructure `(let [[a b] tup] …)` (the
   guaranteed typed tuple read). Mention which you used.

## STOP triggers (REJECTION criteria — ship nothing, surface the exact checker error)
- **STOP-1 (load-bearing):** if the generic `<I,O>` types will not check, STOP. Do NOT
  specialize to i64, do NOT drop the `<I,O>` generics, do NOT add a Rust shim. The
  genericity IS the feature (this is the consented-MapReduce engine; it must be generic over
  the work type). Surface the verbatim checker error and the form it points at.
- **STOP-2:** do NOT edit `tests/nursery/probe_arc259_brackets_map.rs`. It is the gate. If it
  looks wrong, surface that — do not change it.
- **STOP-3 (architectural):** the coordinator touches runners ONLY via `select'` / `send'` /
  `recv'` (the Peer). NEVER a shared crossbeam queue or any shared-memory channel. If the
  skeleton seems to need one, you have misread it — re-read; do not introduce one.

## Gate (run each, READ the output, report REAL results — never chain test+commit)
1. `cargo test --release -p wat --test nursery probe_arc259_brackets_map -- --test-threads=1`
   → `brackets_map_doubles_in_order` + `brackets_map_small_in_order` both GREEN.
2. `cargo test --release -p wat --test nursery probe_arc259_bracket_runner -- --test-threads=1`
   → still GREEN (runner-loop unchanged).
3. `cargo build --release` clean.

## Report back
- The full text of the two defns you wrote.
- Which watch-point fallback (if any) you used and why.
- The verbatim final line of each gate command.
- Any STOP trigger hit, with the verbatim checker error. Do NOT commit.
