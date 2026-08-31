# BRIEF — STONE: a declaration cannot be STORED unvalidated

Make storing binding metadata and validating it **one operation**, so a metadata map that claims
substrate axis properties cannot be written into `sym.binding_metadata` without passing
`wat_doc::from_metadata` — and so the error names the author's line instead of a later reader's.
DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-a-declaration-cannot-be-stored-unvalidated.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or
`scripts/floor.sh`. You may run the pre-existing `target/release/wat` for a fast read, remembering it
does not contain your Rust changes. **You may not spawn sub-agents.** Work only in
`/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not commit, push, stash, revert, or
`git checkout --` anything.

## Read in order

1. The DESIGN above — the pinned decision, and why five more call sites is explicitly *not* the fix.
2. `src/runtime.rs` — the **six** `sym.binding_metadata.insert` sites (`861`, `978`, `1507`, `2848`,
   `4007`, `4082`). Read each: they differ in what they hold at that moment (a `HashMap`, a peeled
   pair list, a span or not). That variation is the real work.
3. `src/runtime.rs`, the wired site (`~1507`) and `meta_has_doc_axis_key` / `AXIS_DECLARATION_KEYS` —
   the predicate and the existing validation call, which the chokepoint absorbs.
4. `crates/wat-doc/src/lib.rs`, `pub fn from_metadata` — unchanged by this stone; you are moving
   *where* it is called, not what it does.

## The work

### 1 — one chokepoint

Add a single function that is the **only** way binding metadata enters the symbol table. It takes
what a caller has (the name, the metadata, and a span for the declaration), validates when the map
carries an axis key, and inserts. All six sites route through it.

After this, `grep -c "binding_metadata.insert"` should be **1**.

### 2 — the span must be the DECLARATION's

Today a bad map's error surfaces at the `metadata-of` **call site** — measured: the map is on line 2
and the error points at line 7. The chokepoint must raise at the declaration's own span, so the
message reaches the author.

If a call site does not have a span in hand, that is the interesting part of this stone — say what
you found rather than passing a synthetic one.

### 3 — nothing changes for capability-only maps

`{:restricted-to […]}` carries no axis key. It stores exactly as today, unvalidated. Same predicate.

### 4 — the probe

`wat-scripts/scratch-pad/255-probe-a-declaration-cannot-be-stored-unvalidated.wat`, following the
shape of the others there. It should show that a partial declaration is refused, and that a complete
one and a capability-only map are both unaffected.

## Blast radius

`src/runtime.rs` only, plus the new probe. No changes to `crates/wat-doc/`, to `eval_metadata_of`'s
emission, to any `.wat` corpus file, or to any registration's *behaviour* for maps without axis keys.

## STOP triggers — each rejects; ship nothing further on that point and report

**STOP-1 — one door, not six checks.** If the six sites cannot be routed through a single function,
STOP and report what blocked it. Adding a validation call to each site is the CHECK rung and is
explicitly disqualified: the arc has paid twice this week for a gate that had to be remembered at N
sites (`dispatch_verbs`' two anchors, the completeness gate's `read_dir`).

**STOP-2 — do not invent a span.** If a call site genuinely has no declaration span, report it. A
synthetic or borrowed span would put the error back on the wrong line, which is the defect this
stone exists to remove.

**STOP-3 — capability maps must not start failing.** Four corpus sites carry `{:restricted-to …}`
and one nearly got migrated by accident already. If routing them through the chokepoint changes what
they do, STOP and report.

**STOP-4 — no new validation vocabulary.** The chokepoint calls `from_metadata` and surfaces its
`DocError`. If you find yourself writing a second set of checks, STOP — that is the drift `wat-doc`
exists to prevent.

## Report

Per-file diff summary; the chokepoint's signature and how each of the six call sites met it
(especially any that lacked a span); the probe's output from the pre-existing binary with an explicit
note on what only a rebuild shows. Then the part the orchestrator cannot reconstruct: what surprised
you — a call site whose metadata was not what the others hold, a path that turned out to be dead, or
a place where "one door" fought the borrow checker in a way worth knowing.
