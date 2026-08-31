# BRIEF — STONE: `@see` can cross the boundary

Let a `@see` reference resolve to a **declared** wat verb as well as a registered Rust intrinsic, so
`sort$native` can point a reader at `sort` — its own public wrapper, and the single most useful
cross-reference it has. DESIGN:
`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-see-can-cross-the-boundary.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or
`scripts/floor.sh`. You may run the pre-existing `target/release/wat` for a fast read, remembering it
does not contain your Rust changes. **You may not spawn sub-agents.** Work only in
`/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not commit, push, stash, revert, or
`git checkout --` anything.

## Read in order

1. The DESIGN above — why *declared* rather than *exists*, and why `contains_key` alone is not the
   test.
2. `src/intrinsic/reflect.rs`, `fn check_see_refs` — six lines, registry-only. This is what changes.
3. `src/freeze.rs:1185`, `pub fn startup_bare()` — a `FrozenWorld` with the stdlib loaded and no user
   program. `world.symbols()` reaches `binding_metadata`.
4. `src/runtime.rs` — `AXIS_DECLARATION_KEYS` and `meta_has_doc_axis_key`. **Reuse this predicate.**
   A third notion of "is this a declaration" is how three of them drift apart.
5. `wat/string.wat`, `:wat::string::capitalize` — the one existing wat declaration, and the shape to
   copy for `sort`/`sort-by`.
6. `src/intrinsic/collection.rs`, `sort$native`'s doc block — it carries a comment explaining why its
   `@see` was removed. That comment comes out with the restoration.

## The work

### 1 — `sort` and `sort-by` declare

Give both a metadata map in `wat/core.wat`, following `capitalize`'s shape. They are wat
`defclause`s; the map goes in the same position a `defn`'s does.

⚠ Their axis values are their own, not `sort$native`'s. Read what each clause actually does before
writing `:purity`/`:determinism`/`:totality`/`:expand-time` — a declaration copied from a neighbour
is the thing this arc keeps finding.

### 2 — the gate resolves against both stores

`check_see_refs` accepts a target that is either a registered intrinsic **or** a wat verb whose
stored metadata carries an axis-declaration key. Get the second store from `startup_bare()`.

⚠ Build the world **once** for the whole check, not once per `@see`.

### 3 — restore the reference

`sort$native` gets `@see :wat::core::sort` back, and the paragraph explaining its absence comes out.

### 4 — prove the rule still bites

The interesting half of this stone is the **negative**: an *undeclared* wat verb must still be
flagged. Add a test (or extend the existing one) that proves a `@see` at an undeclared wat name is
still reported dangling — otherwise this stone cannot be distinguished from "accept anything in the
symbol table", which the DESIGN disqualifies.

## Blast radius

`src/intrinsic/reflect.rs` · `src/intrinsic/collection.rs` (the `@see` line and its explanatory
paragraph) · `wat/core.wat` (two metadata maps) · one test. No changes to `crates/wat-doc/`, to
`record_binding_metadata`, or to `AXIS_DECLARATION_KEYS` itself.

## STOP triggers — each rejects; ship nothing further on that point and report

**STOP-1 — `contains_key` is not the test.** A `{:restricted-to […]}` map lives in
`binding_metadata` and declares nothing. If the gate accepts a target on presence alone, it points
readers at verbs with no documentation — the exact dead link this rule forbids.

**STOP-2 — reuse the predicate, do not restate it.** If `AXIS_DECLARATION_KEYS` is not reachable from
`reflect.rs`, STOP and report. Copying the five key names into a second list is how the storage door,
the reflection surface, and this gate drift apart.

**STOP-3 — the rule must still bite.** If you cannot construct a case where an *undeclared* wat
target is still flagged dangling, STOP and report — a gate that accepts everything is not the gate
this stone describes, and its green would mean nothing.

**STOP-4 — do not declare axes you have not read.** `sort` and `sort-by` get their own values from
their own clause bodies. If an axis is genuinely unclear for either, STOP and report rather than
copying `sort$native`'s or guessing.

**STOP-5 — if `startup_bare()` is not callable from that test context.** `freeze` depends on
`intrinsic`; if reaching it from `reflect.rs` creates a cycle or a `cfg(test)` problem, STOP and
report the shape. Do not work around it by duplicating the corpus knowledge.

## Report

Per-file diff summary; the axis values you gave `sort` and `sort-by` **with what you read to decide
each**; how you proved the negative case still bites; and where you built the world. Then the part
the orchestrator cannot reconstruct: what surprised you — a `defclause` whose metadata position was
not where a `defn`'s is, an axis that was genuinely ambiguous, or a place where one store's answer
disagreed with the other's.
