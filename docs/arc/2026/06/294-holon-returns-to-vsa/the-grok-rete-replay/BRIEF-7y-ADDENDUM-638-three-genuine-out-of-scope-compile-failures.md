# BRIEF-7y ADDENDUM — #638 STOP: the compile gate is sound and finds 3 genuine failures outside grok's own scope

Written mid-batch, per the brief's own explicit instruction for this exact scenario ("If our corpus
carries rules that cannot compile and grok's cure does not reach them, STOP and report with the file
list and the failing axis per file. Do not delete a rule to get green").

## State at this stop

**17 of 20 steps landed and verified**, `#621`→`#637`, tip `f40d8ab4f` (record gate: `scripts/replay/
verify-step-record.sh afa61cee7 HEAD 621 637` → `step-range: #621..#637 each present exactly once,
sources match` / `step-record: complete`, exit 0). `git replace -l` empty, `refs/original/` empty,
`origin/replay/grok-rete` an ancestor of HEAD. Tree clean. `#638`, `#639`, `#640` NOT landed —
`#638`'s own work is preserved, complete except for the disposition below, in
`git stash list` → `638-STOPPED-3-genuine-out-of-scope-compile-failures-pending-builder-call`
(also saved to `/tmp/638_final_staged.patch`, 1780 lines, outside this repo's tracked tree).

## #630 — landed after TWO post-hoc folds, both disclosed in its own commit body (`f6201ec24`)

Read `#630`'s own commit message in full for the complete account. Summary: my own `wat --grep`
-based compile probe (used to verify the hoist at `#630`'s original landing) is silently vacuous on
any file that does not define `:user::grep` — which is most of this tree's `wat-scripts/` corpus
outside `wat-scripts/grep/` itself. `#638`'s own gate (`compile-all` driven directly, no `--grep`)
found this the hard way, twice: first 2 files (`rules-corpus-03-source-to-facts.wat`,
`probe-grep-driver.wat`), then 10 more (`wat-scripts/fmt/rules/*.wat`, all but `cond`/`if`/`match`).
Both folded into `#630` via detach + soft-reset + re-commit + rebuild `#631`-`#637` forward, proven
inert each time (`git diff <old-tip> <new-tip>` names only the reverted files). One additional file
(`277-width-fixpoint-probe.wat`) was CHECKED and found to fail identically hoisted or not — disclosed
as a non-fold (genuine pre-existing rot), not claimed as a third fix.

**Also repaired while landing `#627`/`#628`/`#630`/`#635`: three commit-message record-gate defects**,
found by re-running `scripts/replay/verify-step-record.sh` after the folds above:
- `#627`/`#628` never recorded a literal `nested-program-gate: PASS` line (I ran the loader gates and
  described them in prose, but never ran/recorded THIS specific gate at those steps). Fixed by
  detaching to each tree state, actually running `-E 'test(nested_program_literals_start_on_the_child_path)'`
  (both green, 1/1, 5903 skipped), and amending the message with the real verdict.
- `#630`'s and `#635`'s own `census:`/`lint-subset:` verdict lines had been WRAPPED onto a second
  physical line by my own repeated text edits — finding-31's own trap, and E16's own row, self-
  inflicted. Fixed by re-flowing those sentences onto one physical line each (re-verified at the
  live tree state first, not just re-punctuated).

All of this is now folded into the SAME 7 commits (`#630`-`#637`, all re-committed with new SHAs
during these two rounds); the record gate is green over the corrected range.

## #638 — the gate itself, PROVEN SOUND, and its real verdict on this tree

Built `tests/lint/rete_compile_gate.rs` — one repair needed first: `use wat::load::FsLoader` doesn't
resolve on this tree (the type moved to `wat::load::loader::FsLoader` at some point after grok's era,
same class as every other "wat SOURCE / import path in retired-spelling" repair this whole batch has
made) — fixed, one line, disclosed.

**The instrument is sound, not uniformly broken like `rete-compile-census.sh` (`#636`'s own finding).**
Proof: of the corpus this gate walks, 5 of 16 shards passed outright and the failing shards each name
a DIFFERENT file with a DIFFERENT, mechanism-specific message (an alpha-condition compile failure at
a named line/col, a match-totality `AssertionFailure` with a full frame stack, a `CompileOutcome::
MayNotTerminate` value) — a uniform false failure looks nothing like this. Cross-checked against
known positives (the 16 files this tree DID keep hoisted at `#630` all still pass) and known negatives
(the files folded back above reproduce their failure identically before AND after the corresponding
`#630` revert, confirming the gate — not the revert — is what's measuring).

**After both `#630` folds, exactly 3 genuine failures remain, verified full-verbatim (not sampled):**

```
wat-scripts/scratch-pad/experiri-then/then-match-paren-arm.wat
    a :then admits only what the fence can prove TOTAL. `match` is total as a HEAD, but a
    match's exhaustiveness is a property of ITS ARMS — form-level, which a head-level axis
    cannot see. Use :wat::rete::core::variant-name for a variant's name, or bind the value
    in :when.
    (AssertionFailure, wat/rete/compile.wat:809, via then-item-fence -> compile-rule)

wat-scripts/scratch-pad/experiri-then/then-match-bare-arm.wat
    (identical message/mechanism to the paren-arm sibling above)

wat-scripts/scratch-pad/277-width-fixpoint-probe.wat
    CompileOutcome::MayNotTerminate (not Compiled)
```

**All 3 are confirmed OUTSIDE this step's own scope and unrelated to anything this replay has done**:
byte-for-byte identical to their content at `afa61cee7` (this batch's own start, before `#621` ever
ran) — `diff <(git show afa61cee7:<path>) <path>` is empty for all three. None of the 3 is in grok's
own 11-file disposition table (`DESIGN.md`'s "The eleven"). This is DESIGN.md's own named STOP-3
condition, verbatim: *"the gate reds on a file NOT among the eleven … Stop, report the file and its
message, and do not extend the delete list on your own judgment."*

## Why this executor is not deciding the disposition

`DESIGN.md`'s own contract (`#637`, already landed) is **ZERO exemption categories, no rune, ever** —
and it explicitly names deleting a rule to buy green as the rejected alternative. The only two
remaining honest options — delete these 3 files (matching the disposition grok gave the ORIGINAL
eleven), or repair their fences (mirroring the two-codemod repair grok already did) — are both
judgment calls DESIGN.md reserves for the builder ("a deliberate act with its own argument"), not
something this executor's own brief authorizes deciding alone. Landing `#638`'s gate now, as-is,
would either (a) commit a knowingly-red test suite (forbidden — "no knowingly-red REPLAY commit"),
or (b) require this executor to unilaterally delete/repair 3 files DESIGN.md never named, which the
brief explicitly forbids ("do not extend the delete list on your own judgment").

## What is preserved for the next hand

- Branch tip: `f40d8ab4f` (`REPLAY(grok-rete #637)`), clean, verified (record gate, census, loader
  gates, `lint-subset` 332/332, `kind(lib)` 1524/1524, `doctest` 8/8, targeted family 14/14).
- `#638`'s complete, gate-passing-except-for-the-3-genuine-files work: `git stash list` entry
  `638-STOPPED-3-genuine-out-of-scope-compile-failures-pending-builder-call` (staged diff also at
  `/tmp/638_final_staged.patch`). Popping it and choosing ONE of (delete all 3 / repair all 3 /
  repair some+delete some, matching grok's own precedent per file) is what completes `#638`.
- This addendum, for the record.

**Not decided, on purpose:** which disposition the 3 files get. That is the builder's call.
