# BRIEF — STONE: `metadata-of` answers in ONE shape, and `:defined-in` stops lying

Make `eval_metadata_of`'s two branches return the same value shapes, and make `:defined-in` report
what each branch actually is instead of a spliced constant. DESIGN:
`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-metadata-of-answers-in-one-shape.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or
`scripts/floor.sh`. You may run the pre-existing `target/release/wat` for a fast read, remembering it
does not contain your Rust changes. **You may not spawn sub-agents.** Work only in
`/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not commit, push, stash, revert, or
`git checkout --` anything.

## Read in order

1. The DESIGN above — the pinned decision, and why `:layer` is deliberately excluded.
2. `docs/arc/2026/06/255-builtin-registry/NOTE-metadata-of-returns-two-shapes-depending-on-which-store-answered.md`
   — the measurement, and the fix it names.
3. `src/runtime.rs`, `fn eval_metadata_of` — **both** branches. The registry branch (typed `put`s,
   early `return`) is the shape the wat branch must match. The wat branch's
   `map.insert(keyword(k), Value::wat__WatAST(v))` is the defect.
4. `crates/wat-doc/src/lib.rs`, `pub fn from_metadata` — already public, already producing a
   `DocComment` with typed fields. This is the decoder; do not write another.
5. `src/runtime.rs`, the registration-site gate that reads `DOC_AXIS_KEYS` — **the same predicate**
   decides here whether a stored map is a doc declaration or a capability-only map.

## The work

### 1 — the wat branch emits from a `DocComment`

When the stored map carries any doc-axis key, run it through `wat_doc::from_metadata` and emit from
the resulting `DocComment`, **key for key with the registry branch's typed emission**. Reuse the
registry branch's own `put` shapes so the two cannot drift apart by inspection.

⛔ **Do not decode the AST yourself.** If a field seems to need decoding at the reflection layer,
that is the signal it belongs in `from_metadata` — see STOP-1.

### 2 — a map with no doc-axis key keeps today's behaviour

`{:restricted-to […]}` (4 live corpus uses) has no doc-axis key. It must come back exactly as it
does today: raw, un-decoded, unvalidated. Same predicate as the registration gate.

### 3 — `:defined-in` from the branch, not from a constant

The registry branch is reached only by a `#[wat_intrinsic]` entry (`Rust`); the wat branch only from
`binding_metadata` (`Wat`). Each emits its own. That is a fact at the site.

⛔ **`:layer` stays exactly as it is** — a hard-coded `Substrate`. Do not touch it and do not derive
it. See STOP-3.

### 4 — the probe

`wat-scripts/scratch-pad/255-probe-metadata-of-one-shape.wat`, following the shape of the others
there. It must compare an **intrinsic** and a **wat verb** across `:purity`, `:totality`,
`:determinism`, `:expand-time` and `:category` — not `:purity` alone. Converging one key and leaving
the rest moves the defect instead of removing it.

## Blast radius

`src/runtime.rs` (`eval_metadata_of` only) · possibly `crates/wat-doc/src/lib.rs` if a field is
genuinely missing from `DocComment` · the new probe. No changes to registration, to `from_metadata`'s
callers, or to any `.wat` file.

## STOP triggers — each rejects; ship nothing further on that point and report

**STOP-1 — one decoder, not two.** If emitting a field requires decoding an AST inside
`eval_metadata_of`, STOP and report which field. The fix belongs in `from_metadata`, where the one
decoder lives; a second decoder in the reflection layer drifts the first time an axis gains a
variant, which is the exact failure `wat-doc` exists to prevent.

**STOP-2 — the capability maps must not change.** If making the doc path typed also changes what a
`{:restricted-to […]}` map returns, STOP and report. Those 4 corpus sites are unrelated to this
stone and were nearly migrated by accident once already.

**STOP-3 — do not derive `:layer`.** It is `Substrate | Userland`, and no branch can know which: a
substrate wat def and a userland wat def arrive through the same one. The only available answer is a
name-prefix guess, which is `effectful_by_prefix` reborn inside the field whose job is provenance.
If you believe you can derive it honestly, STOP and report your reasoning rather than shipping it.

**STOP-4 — `:defined-in` is derived, not defaulted.** If either branch cannot state its own
provenance as a fact at the site, STOP and report — a defaulted provenance field is the defect this
stone exists to remove, and swapping one constant for another achieves nothing.

## Report

Per-file diff summary; how you kept the two branches' emission from drifting (and whether you
factored it); the probe's output from the pre-existing binary, with an explicit note on what only a
rebuild can show. Then the part the orchestrator cannot reconstruct: what surprised you — a
`DocComment` field the registry branch emits that has no wat-side equivalent (or vice versa), a key
whose two shapes were harder to converge than the NOTE implied, or a place where the two branches
disagree about something the NOTE did not name.
