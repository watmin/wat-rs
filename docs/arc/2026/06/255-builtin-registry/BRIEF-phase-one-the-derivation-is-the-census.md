# BRIEF — ③b-ii phase ①: the derivation IS the census

Anchor: `/home/john/work/holon/wat-rs`. Verify with `pwd`; a path containing `.claude/worktrees/`
is harness state — use `git -C /home/john/work/holon/wat-rs` for git.

Read `DESIGN-the-flip-asks-843-times.md` (this brief's sibling) first — it carries why one global
ask was thrown away.

## ⛔ THIS PHASE REWRITES NOTHING

No `.wat` corpus file is modified. No `src/` file is modified. Phase ① produces **a pair list and
the instrument that made it**, and nothing else. The flip is a later stone.

## The work, in one paragraph

A variant cannot be recognised by its spelling — `:wat::program::PeerKind::thread` is a variant and
`:wat::core::Record::def` is a surface method, and they are the same shape. So the substrate is
asked: `:wat::runtime::variant-parent-of`. But the ask consults the **running** type env, so a
single global pass sees only loaded code — measured, it confirmed 217 names while declining 345
that are real `defenum` variants in unloaded corpus files. The fix is to ask **once per file, with
that file loaded**: `(:wat::load-file! "<absolute path>")` at top level brings a file's own
`defenum`s AND its macro expansions into scope while the stdlib's answers survive alongside. Your
job is to run that over every corpus `.wat` file and emit the union of confirmed renames.

## What is already proven, and on disk

`wat-scripts/scratch-pad/dot-flip-derive-renames.wat` — the ask loop, working. It reads a vector of
BARE names (no leading colon) on stdin and prints `:old::spelling :new.spelling` per confirmed
variant. Its `pair-for` composes the new spelling **from the answer**, never by editing the old
string — the parent the substrate returns IS the left half, so the dot lands where the substrate
says the boundary is. Reuse that function verbatim; it is the part that must not be re-derived.

`wat-scripts/scratch-pad/dot-flip-ask-the-substrate.wat` — the four-name discrimination probe.

Measured facts you can build on:
- `(:wat::load-file! …)` is FREEZE-TIME. It has no expression position — it must sit at top level,
  and it needs an ABSOLUTE path.
- A driver defining `:user::main` does NOT collide with a loaded file's own `:user::main`. Verified
  on `wat-tests/counter-actor-proof-process.wat`, which mentions it five times.
- `keyword::from-string` refuses a leading colon and refuses angle brackets.

## The shape

For each `.wat` file under `wat/`, `wat-tests/`, `wat-scripts/`:

1. collect that file's distinct namespaced keyword tokens, strip the leading `:`, drop any carrying
   `<`, `>` or `/` (malformed input ABORTS a run rather than answering `None`);
2. generate a driver: `(:wat::load-file! "<abs path>")` at top level, then the proven `pair-for`
   plus a `:user::main` that reads the candidate vector on stdin and prints confirmed pairs;
3. run it, capture stdout.

**Over-generate candidates freely** — the ask is the filter. Do not add a shape predicate to narrow
them; five predicates have already been wrong in this campaign, and removing the last one is the
point of asking.

The union of all files' output, deduplicated and sorted, is the deliverable.

## Deliverables — all durable, none in /tmp

```
the generator + driver template        in-repo (scratch-pad for .wat; scripts/ for a shell driver)
the pair list                          committed as an artifact, sorted and deduplicated
```

The instrument must outlive the number: a reader six months from now must be able to re-run it and
get the same list, without you.

## The rows this phase owes

```
:wat::core::Record::def          ABSENT  from the pair list   (a method, not a variant)
:wat::cache::Cache::GetRequest   ABSENT                        (a defrecord lookalike)
:wat::program::PeerKind::thread  PRESENT                       (a variant with a LOWERCASE leaf)
:counter::Request::Get           PRESENT                       (the one a global ask missed)
:wat::cache::Cache::GetResponse::Ok  PRESENT                   (macro-generated, no defenum exists)
```

Those five together are the non-vacuity control: two that must not appear, three that must, each
from a different reason.

## STOP triggers — each REJECTS the stone; ship nothing further and report

**STOP-1.** A corpus file fails to load. Do NOT skip it silently and do NOT exclude it. Collect
every such file and report the list with one verbatim error per distinct failure shape. A file
whose variants were never asked about is a file the flip will break, and a silent skip is exactly
how that ships.

**STOP-2.** `:wat::core::Record::def` (or any `::Record::`/`::Lru::`/`::HolographicLru::` method)
appears in the pair list. The ask is answering wrong and the whole premise is void.

**STOP-3.** `:wat::program::PeerKind::thread` or `:counter::Request::Get` is ABSENT. The per-file
load is not doing what it was measured to do.

**STOP-4.** A driver collides with a loaded file (duplicate define, or the load runs the file's
own program). Report the file and the verbatim error — the measurement above says it does not
happen, and a counterexample refutes the design's mechanism.

## What to run

Your own generated drivers, and nothing else. **Do not run `scripts/floor.sh`, do not run clippy,
do not `cargo build`** unless a driver needs it — the binary at HEAD already has every verb you
use. Run every command in the FOREGROUND and block on it. Do not commit. Do not contact any peer.

## Report

The pair count; the five control rows above, each shown present or absent verbatim; any file that
failed to load; how long the full sweep took; and anything that surprised you. If the pair count
differs from what a shape-based grep would predict, say by how much and in which direction — that
delta is the finding this phase exists to produce.
