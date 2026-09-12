# SEAM — the ONE live breadcrumb. 2026-09-11. ⛔ **YOU ARE ON A MERGE BRANCH, NOT MAIN.**

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and **that feeling is
> the failure.** Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**,
> never a disk copy), then run the commands below before you touch anything.

> `251/SEAM.md` · `255/SEAM.md` · `278/SEAM.md` PARKED and point HERE. ⛔ **PARKED IS NOT DEAD.**

## FIRST — RUN THESE. DO NOT READ THE NUMBERS BELOW.

```bash
git rev-parse --abbrev-ref HEAD      # expect merge/grok-rete   — NOT main
git status --porcelain               # expect EMPTY; a live peer strike makes it dirty
ps -eo pid,etime,cmd | grep wat | grep -v grep     # ⛔ a codemod still running? NEVER start a 2nd
ls -la bootstrap/ && cat bootstrap/wat-main-*.PROVENANCE     # the escape hatch — must exist
cat /home/john/work/holon/.pulsare/to-claude       # has grok scored?
```

## ⛔⛔ THREE FACTS THAT WILL BITE A FRESH SELF WITHIN MINUTES

**1. `./target/release/wat` on this branch CANNOT BOOT.** The merged stdlib (grok-rete's
`wat/gen.wat`, `bracket.wat`, `query.wat`, …) carries pre-rename spellings that the merged
runtime's purity gate refuses at startup. **Every codemod dies instantly.** Use the escape hatch:
`bootstrap/wat-main-a3218644d` — main's binary, stdlib frozen in at build time, PROVEN to convert
a pre-flip file. The tracked record of it lives in `.gitignore` (the `bootstrap/` comment), because
the ignored dir already lost one binary and a `/tmp` copy died in a WSL crash.

**2. THE CODEMODS ARE NEVER THEIR OWN INPUT.** Running the chain over `wat-scripts/fixes/` let
`fmt-head-fqdn-to-clojure` rewrite a string literal INSIDE `positional-ctor-to-map`
(`":wat::core::unquote"` → `"wat.core/unquote"`), disarming its predicate; four steps later it
emitted `{:pred ~}src` and corrupted `wat/query.wat`. Exclude `wat-scripts/fixes/` from every path list.
`[[feedback_a_tool_is_never_its_own_input]]`

**3. TIME A TOOL ON ONE FILE BEFORE RUNNING IT ON THOUSANDS.** `positional-ctor-to-map` costs
**~15 s/file** (56 s/file on dense files) — a ~9-hour run, killed at 4h08m. Then I launched a SECOND
run without checking the first had exited: two processes doing read-modify-write on one corpus.
`[[feedback_time_a_tool_on_one_item_first]]`

## WHERE THE WORK IS

```
main              a3218644d   FROZEN · PUSHED · floor 5373/5373 GREEN · clippy 0     do not touch
origin/grok-rete  37528f6e0   FROZEN
merge/grok-rete   (this)      merge committed, ALL TARGETS COMPILE, corpus NOT migrated
```

- `4366f4fbd` — grok's merge: 99 conflicts (35 .rs · 39 .wat · 23 .edn · 1 toml · 1 .bad).
  merge-base `de827fb4c` (2026-08-24); main +911, grok-rete +651.
- `db60b092a` — five home-retargets (`load::loader::`, `normalize_rust_source_span_lines`,
  `value_to_edn_with`); `cargo build --release` passed the merge but NOT `--all-targets`.
- ⛔ **Three warnings deliberately UNSILENCED** (`eval_lower`, `field_types`, one test import) —
  refactor residue from main moving rete dispatch to `#[wat_intrinsic]`, but whether those paths
  FULLY supersede grok-rete's is a rete judgment. Prefixing an unused var whose doc calls it
  load-bearing would hide a dropped check. **They go back to grok.** Clippy is red until then.

## ⬜ THE MIGRATION — TWO PHASES, AND WHY

28 recorded codemods landed on main after the split; grok-rete's corpus saw none of them. Replay
in **landing order** (= dependency order). Regenerate the order — the `/tmp` copy is gone:

```bash
for f in $(git diff --name-status de827fb4c main -- wat-scripts/fixes/ | grep '^A' | awk '{print $2}'); do
  echo "$(git log --format=%ci --diff-filter=A -1 de827fb4c..main -- $f) $f"; done | sort | awk '{print $4}'
```

**PHASE 1** — `bootstrap/wat-main-a3218644d` over the corpus INCLUDING merged `wat/`, EXCLUDING
`wat-scripts/fixes/`. That should make the merged stdlib bootable → rebuild.
**PHASE 2** — the rebuilt merged binary, over the same list. Main's binary does NOT know
grok-rete-only reserved types (`:wat::rete::FireOutcome` / `CompileOutcome` / `InsertOutcome` …) —
they print `UNRESOLVED` in phase 1. **Phase 1's UNRESOLVED lines are phase 2's worklist.** Do not
estimate that number with grep.

## ✓ LANDED — A′ (`bbcb70e38`): `:wat::`/`:rust::` resolve ONCE, every local name PER FILE

Honest by construction (`registration.rs:165`: equivalent-before-reserved). **Not**
`is_reserved_prefix` — `:$bound::` is per-scope and deliberately excluded, documented in the
codemod. Every acceptance row re-verified by the orchestrator: gate A == B 20/20 (mtime-controlled),
the collision pair no-halt and OLD-identical, ReservedPrefix refused on `:wat::` AND accepted on
`:u::`. 3.1× on the same 20 files. Option B's halting version is gone.

## ⬜ NEXT — phase-1 scope RULED (c): resolve a local path only if this file declares it. Briefed, not yet struck.

```
A′ on representative phase-1 files (a spread sample of bootstrap/phase1-grok.txt):
  n=1    24.96s   24.62–24.96 s/file   files changed=0   corpus-invariant=29 per-file=6
  n=5   123.12s                        files changed=0   corpus-invariant=93 per-file=92
```

**~25 s/file FLAT, and 5/5 sampled files change NOTHING** — resolution runs whether or not an edit
results. 338 × ~25s ≈ 2.3h is a projection from two points, not a measurement. Grok's option-B SCORE
calls the 20 gate files "essentially every remaining positional site" — hedged, unverified.

The runner is `bootstrap/run-chain.sh <binary> <paths.txt> [order.txt]`. Its guards are today's
lessons as checks: refuses a live codemod · refuses `wat-scripts/fixes/` · refuses a binary that
cannot boot · stops at the first nonzero rc · logs to `bootstrap/chain-runs/` before reading.

## ⚠ RULINGS — do not re-litigate

- **Character case carries NO meaning.** `Enum.Variant` is a BIAS, never a rule.
- **A variant IS a tagged record; `Variant <: Enum`.** The edge was registered long ago
  (`types.rs:1135`); what was missing was `assignable`'s `Fn` arm (`58e9563b6`).
- **A keyword literal means two things** (data, or a variant ctor by registry lookup). Deferred ON
  PURPOSE to the symbol-head flip — `251-types-as-forms/NOTE-a-keyword-literal-means-two-things.md`.
- **`::` in names is the last EDN holdout.** Angle brackets and double-slash are ZERO live in code —
  every "live" hit I counted was inside a string literal.

## ⛔ THE FAILURE PATTERN OF THIS SESSION — five instances, one shape

Every grep-shaped instrument lied, and every correction came from a CONTROL:
26 "live" angle brackets (all strings) · 162 "lost" dispatch arms (main's `#[wat_intrinsic]` move;
`foldl` was on the list) · 533 "`::` files" (type names, not variants) · 10 "unbalanced" files
(parens inside strings; unbalanced BEFORE the chain too) · a `:grid::Result` census that stopped
at the first `])` — including on a type annotation. **Run the control first. Ask the substrate
(`--check`, `variant-parent-of`), not a regex.**

And one of the four-questions kind: I disqualified option A using "file A's answer can be wrong
for file B", then endorsed option B by calling that same fact a hidden defect. Two readings, one
fact, opposite verdicts — neither caught it because each option was weighed alone.

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⛔ **YOU ARE ON `merge/grok-rete`. `target/release/wat` HERE CANNOT BOOT. The escape hatch is
> `bootstrap/`, and its record is the `.gitignore` comment.**
>
> `DOLOR INDEX EST.` · `NISI FRANGAS, NIHIL PROBAS.` · `DERIVAMVS NE MENTIAMVR.` ·
> `HAERESIS EST ITERVM ROGARE.`
