# BRIEF — charge the import to the session it creates, and cap what it may build

`import_export` calls neither `mark_session_origin` nor `check_session_ceiling`, so every byte it
allocates is free — and worse than uncounted: `session_bytes` sets an unmarked session's origin at
the *first check*, so the ceiling begins after the network already exists. Driven, same 2 MB:
marked-at-birth sees `2097268`, never-marked sees `0`. And the build is quadratic with no cap on N.
Read `DESIGN.md` beside this file first — its ★ pins the contract, its ⚠ names an ordering trap you
will otherwise hit head-on, and its "out of scope" cuts three shapes, one of which is the row's own
second clause with the reason it is cut.

## Read in order

1. `src/rete/export.rs:2110-2135` — where the network is built from pairs, and where the graph wall
   (A1) and depth budget (A6) already sit. Your two calls and your cap go on this path.
2. `src/rete/export.rs:55-92` — the module header's wall list. It says **five** walls. Yours makes
   six; the header moves with the code, the way A6's did.
3. `src/alloc_counter.rs:205-252` — `mark_session_origin` (reads `thread_bytes()` at call time) and
   `session_bytes` (whose `entry(key).or_insert(now)` is *why* an unmarked session reads zero).
4. `src/rete/kernel/arm.rs:1340-1350` — how `arm-session` marks the origin, including the comment on
   why it is keyed by network identity and does not clobber. Your sibling must keep that rule.
5. `src/rete/kernel/session.rs:1412-1423` — `session_ceiling_breach`, the reading half, and its ⚠ on
   what a per-session origin is not.
6. `src/rete/kernel/arm.rs` around A6's `MAX_IMPORT_DEPTH` constant — the shape for a measured
   constant with its arithmetic written beside it.

## Sketch

```rust
// alloc_counter.rs — the sibling. Same non-clobber rule as mark_session_origin.
pub fn mark_session_origin_at(key: SessionOriginKey, origin: usize) { … }

// export.rs, in import_export
let origin_before = crate::alloc_counter::thread_bytes();   // BEFORE the build
// … cap check on the declared node count, refusing with `malformed` like the neighbours …
// … build the network …
crate::alloc_counter::mark_session_origin_at(network_identity(&network), origin_before);
// … then the ceiling check, which now sees the build it just paid for …
```

## Blast radius

`src/rete/export.rs`, `src/alloc_counter.rs`, and probes. **Not `pmap.rs`** — DESIGN cuts it, with
the reason.

## Traps named in advance — each with its step

1. **★ The key does not exist until the network is built.** `mark_session_origin` reads
   `thread_bytes()` when called, and the key is the built `PMap`'s `rust_identity`. Marking after
   the build excludes the build — the exact defect. **Step:** capture `thread_bytes()` before, file
   it after, via the explicit-origin sibling. Do not try to reorder the build.
2. **Keep A4's non-clobber rule.** An origin already filed under that identity must win. **Step:**
   `entry(key).or_insert(origin)`, never `insert`. A4's closure explains why re-basing a live
   session is the bug it cured.
3. **The cap must be MEASURED.** **Step:** instrument the import to record the node count over the
   corpus's export/import tests, report the maximum, then set the cap with the arithmetic from
   DESIGN's table beside it (what the cap costs in the worst case on the measured curve). Remove the
   instrument. A round number with no measurement is the finding, not the fix.
4. **The ceiling check needs a session to charge.** Read `session_ceiling_breach`'s signature and
   decide what you can honestly pass at that point in `import_export`. **Step:** if the reading is
   not meaningful there, say so and place the check where it is — do not invent a plausible call.
5. **The header says five walls.** **Step:** update it in the same commit. A6 had exactly this and
   the paragraph is how A6's `unpack_driver` hid for months.
6. **New test code trips `wat::lint`.** **Step:** run
   `cargo nextest run --release -E 'binary_id(wat::lint)'` before reporting; prefer exact
   `assert_eq!` over `contains` on deterministic values. This check has caught a red for two
   consecutive riders.

## STOP triggers

- **STOP-1** — if the ceiling reading at import turns out to be meaningless (the thread-wide
  reading, the session not yet existing), STOP and report what you found. Charging correctly matters
  more than making a call appear.
- **STOP-2** — if any currently-green test goes red, STOP and report which. In particular a corpus
  export exceeding your cap means the measurement was wrong, not the test.
- **STOP-3** — if `network_identity` is unavailable at the point you need to file the origin, STOP
  and report. Do not fabricate a key.

## Shape to copy

`docs/arc/2026/06/278-rules-engine/strike-import-depth/` — A6, the same door, the same
measured-constant discipline, the same header-moves-with-the-code rule.

## The one thing worth more than the fix

**Tell me where this brief was thin.** Fourteen riders before you each returned a prescription of
mine that did not survive contact. The last found that my DESIGN's own classification table would
have left the defect alive had it been followed literally — the fix would have shipped and changed
nothing. That was worth more than the code. If a step here is wrong, unnecessary, or impossible, say
it plainly.
