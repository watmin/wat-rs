# Step notes: #212 (the docs-wat gate collision) and #226 — VERIFICATION of a recorded ruling

⚠ **#212's disposition was ALREADY RULED and is recorded in the SEAM** (§ Batches):
> *"#212 #278 are NOT boundaries (finding 21's correction): … #212's two `130-…` `.wat` R100 → `.wat.bad`
> (both sides preserve them; the suffix IS grok's rune, whose gate scans `.wat` only → drop the rune,
> merge the README prose, log it)"*

This file does NOT re-open that. It records the measurements that confirm the ruling is correct AND
executable, so whoever replays #212 does not re-derive them. `HAERESIS EST ITERVM ROGARE.`

## What #212 is

`9ee04f945` — *"gate: every docs/arc .wat loads, or declares in a closed rune why it does not"*, 7 files:
a new lint `tests/lint/docs_wat_loads_or_declares_why_not.rs` (172 lines) walking `docs/arc/**/*.wat` and
requiring each to LOAD on the current runtime **or** carry `;; rune:lint(red-by-design|historical) — <reason>`,
plus the content edits that make its own corpus pass.

## Why the two sides collide, and why the ruling resolves it

Both sides independently solved "a docs `.wat` that does not load", by opposite mechanisms:

| | mechanism | escape hatch |
|---|---|---|
| main `c5f1ee487` (2026-08-25) | `every_tracked_wat_parses` — every tracked `.wat` must PARSE | **a RENAME to `.wat.bad`**, deliberately *"not an exemption list, so there is nothing to keep in sync"* |
| grok `#212` (2026-08-30) | `docs_wat_loads_or_declares_why_not` — LOAD or declare | **a rune**, appended at the FOOT of the arc-130 pair |

grok's rune does not make the file parse. Applying #212 literally — un-renaming `.wat.bad` → `.wat` and
appending runes — puts main's wall RED **by main's own demonstration**: *"Renamed substrate.wat.bad back
to .wat, re-ran: RED, naming the file."*

**The ruling works because the two gates cannot see each other's exempted files.** VERIFIED:
grok's walk filters `p.extension().is_some_and(|x| x == "wat")` (`collect_wat`, line 70), so
`substrate.wat.bad` has extension `bad` and is skipped — exactly as it is skipped by main's
`git ls-files "*.wat"`. Keep the rename, drop the two `historical` runes, and BOTH walls are green with
nothing lost. It also honours grok's own stated doctrine: its gate header says *"A rune EXEMPTS the file
from the load check — that is its whole function — so runing a file is a last resort"*; a rune on an
unparseable file is precisely the exemption-list entry main's rename exists to avoid. The rune's CONTENT
(the builder's *"we must not forget what bad looks like"*) is already preserved in our README.

## Executable detail for the step (all measured on this tree)

- **The corpus.** Our branch has 4 docs `.wat`; grok at #212 had 10. The 6 missing are NOT divergence:
  4 are `harness-experiri/*.wat`, which ARRIVE AT STEP #195 (inside batch 4b — main never had them:
  `git log a3218644d -- '*experiri*'` is empty), and 2 are the arc-130 pair, which exist here as
  `.wat.bad`. **So at #212 the gate walks 8 files** and its non-vacuity guard (*"no .wat found under
  docs/arc/ — the gate is measuring nothing"*) is satisfied.
  ⚠ **Re-take this census AT the 4b tip, not before** — #195 lands inside 4b and changes the number.
- **`probes/surface-field-dispatch.wat` — grok's `:holder` → `:nature` migration APPLIES.** Our copy still
  reads `:holder` at line 11, so the ~8-week rot grok found is LIVE on this branch too. It carries no rune
  by design (*"a rune here would have rebuilt the graveyard inside the gate"*). Single file, single
  keyword — not a corpus migration, so not an R21 codemod; this is grok's own content landing.
- **`probes/red-owner-signals-child.wat`** (+15: a `red-by-design` rune and header corrections) and
  **`harness-experiri/experiri-then-match.wat`** (+11, rune) land normally. Our `red-owner-signals-child`
  carries no rune today.
- **The arc-130 README:** grok appends 15 lines documenting the runes and the foot-placement reasoning.
  Our README already carries the whole preservation story INCLUDING a `## ⚠ The .wat.bad extension` section
  and the citation updated to *"the original line 64 of `substrate.wat.bad`"*. Per the ruling, merge the
  prose — do not import the rune-explaining section as written, since the runes are not landing.
- **The third `.bad` file is NOT a collision.** `tests/cli/wat_grep__malformed.wat.bad` is main's own
  (`c80aa5860` is an ancestor of main, NOT of grok); grok has no `tests/cli/*malformed*` at #212.
- **The gate never changes after it lands** (`git log 9ee04f945..37528f6e0 -- <the lint>` is empty).
- **Where it bites later:** docs `.wat` are ADDED at **#219** (1, inside 4b), **#328** (1) and **#385**
  (10 at once). #385 is a future step to watch — ten files that must each load or declare.
- **The collision is one-time:** grok never touches `docs/arc/2026/05/130-…/` again after #212.

## #226 — a step to watch, not a boundary

`7319c1ea4` *"fix(rete): with-network's scope is closed by a Drop, not a release call"* — 6 files,
`stdlib-files=1` (`wat/rete/syntax.wat`) with 5 shared `.rs` (`src/check.rs`, `src/runtime.rs`,
`src/rete/kernel/arm.rs`, `src/rete/purity.rs`, + its test). It is in `future-macro-changes.txt`: it
changes the stdlib MACRO `:wat::rete::defquery`. That is finding 19's G1 class — the door refusing a
divergent stdlib macro — **closed by 2a4c** (`retract_divergent_stdlib_macros`). One stdlib file, so the
ordinary 2a4 two-phase rule covers it; 2a4d's per-SET world is not needed. Not a boundary, but the first
divergent-macro step since 2a4c closed G1, so read its two-phase convert output rather than assuming.

**#221 is the real one to watch first** (`16f504e14`, `stdlib-files=2`: `wat/rete/oracle/fire.wat` +
`stratify.wat`) — the first MULTI-stdlib-file step since 2a4d, i.e. the first real exercise of the
per-SET world that stone was built for.
