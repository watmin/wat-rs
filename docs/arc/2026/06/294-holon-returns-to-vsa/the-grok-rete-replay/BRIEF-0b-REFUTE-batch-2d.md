# REFUTE — 0b batch 2d: close stone 0b

Batch 2c (`d1c47b2a4`) is green and pushed. The orchestrator re-ran it: gate 18/18, floor 5391/5391,
clippy 0. The ORACLE arm is real: (i), (j) and (k) reproduce RED, and all 67 history citations are
on main's history, with no self-citation. **KEEP ALL OF IT.** Seven things remain. Six are yours.
One is affirmatively out of this stone.

## D1 — `git_show` must fail LOUDLY

Today, `fn git_show` returns `String::new()` on a non-zero exit. The orchestrator reproduced two
consequences:
- `history 110335bd5 <real path> no/such/file.wat` **passes** (rc=0). The bogus path contributes
  nothing, silently.
- A nonexistent commit goes red, but for the WRONG reason: *"changed after.post line not in
  history"*. That names the wrong mechanism.

This is "none means skip": it conflates *could not look* with *looked and found nothing*. Fix it:
- A `history` commit that does not resolve is RED, and the message names the commit.
- A cited path that exists at NEITHER `<commit>` NOR `<commit>^` is RED, and the message names the
  path. A path that exists only at the parent is the legitimate rename case: it contributes to the
  before-check only.
- Prove both new arms, RED then reverted, with the verbatim text in the SCORE:
  - (l) a bogus path next to a real one;
  - (m) a nonexistent commit. The message must say the commit does not exist.

## D2 — mixed fixtures check their before-lines

Today a fixture with ANY `spec` entry skips the before-check entirely (*"History-only fixtures still
check every before"*). The input side of its history-derived cases is never verified. Fix it with a
new entry kind:
```
spec-before <a before.pre line, verbatim> :: <a verbatim substring of a header comment line>
```
- In EVERY fixture, each changed `before.pre` line must be found in the union of `<commit>^:<path>`
  over its `history` entries, OR be named by a `spec-before` entry whose quote is in the header.
- The `after` rule is unchanged.
- Backfill `spec-before` into all 29 spec-bearing fixtures.
- Prove it: (n) a mixed fixture with a hand-altered history before-line goes RED.

## D3 — `namespace-bare-top-level-names` has history. Use it.

The orchestrator ran today's tool on the landing `72a1ac3d0`'s parent files. It **REPRODUCES
history** on 5 of 6:
- `tests/types/probe_arc237_sA1_assignable_probe01.wat`, `…probe02.wat`, `…probe04.wat`
- `tests/types/probe_arc237_sC3_macro_split.wat`
- `tests/wat_lang/probe_def_not_special.wat`

Its spec reason (*"no-op under today's tool"*) is false. Rebuild the fixture from those forms and
cite them as `history`.

## D4 — `rule-record-to-defrule`'s reason is wrong

Today's tool on the landing `5b86828f8`'s parent where-grid files dies with rc=2:
`lex error at byte 13765: angle-bracket type parameters are illegal …`. Those whole files are
unreadable today. The stated reason (*"rete probes are no-op"*) is not the mechanism.
- Extract whole top-level forms from those files that the reader accepts and that the landing
  commit changed ONLY by this tool. Cite them as `history`.
- If every changed form is itself angle-bracketed, keep spec, and record THAT reason in the SCORE.

## D5 — RULED by the builder, 2026-09-12: RESTORE `rename-record-def-to-defrecord`

- `dedcb74a7` (the `:wat::Record` → `:wat::core::Record` sweep) rewrote the tool's own second
  call, `":wat::Record::def"`, into `":wat::core::Record::def"`, a name that never existed.
- The landing `60d7d99a0` removed **44** `:wat::Record::def` sites, and **0**
  `:wat::core::Record::def`. Its header documents `:wat::Record::def -> :wat::core::defrecord`.
- Restore that string to its landing value. Check the header prose against `60d7d99a0` too, and
  restore any line the sweep mangled (comment-only).
- Rebuild the fixture from `60d7d99a0`'s real sites, choosing forms it changed ONLY by this tool,
  and cite them as `history`. Keep a near-miss: `:wat::holon::defrecord` and `:wat::core::defstruct`
  stay unchanged.

## D6 — "the tool was itself migrated" is ROT, never a spec reason. Audit every spec-bearing stem.

A tool that cannot reproduce its own landing on READABLE input is broken. It is repaired, not
fixtured against its broken behaviour. For EACH stem whose ORACLE carries a `spec`:
- run today's tool on `<landing>^` of each `.wat` its landing commit changed (outside
  `wat-scripts/fixes/`);
- classify each file as one of: **REPRODUCES** · **NO-OP** · **DIFFERS** · **UNREADABLE** (a lex
  error on the pre-image).

Then act on the table:
- **Any REPRODUCES:** that fixture moves to `history`.
- **NO-OP or DIFFERS on readable input, with no mixed-commit explanation you can cite:** it is ROT.
  Repair it (restore from landing, as in D5), and cite the root cause. STOP-1 applies: no `src/`, no
  change to documented behaviour.
- **Only UNREADABLE or genuinely mixed:** spec stays, and the SCORE records which.

Put the whole table in the SCORE. The orchestrator ran exactly this check on two stems (D3, D4); run
it on all of them. **Time it on one stem first.**

## D7 — OUT OF THIS STONE, affirmatively: the `positional-ctor-to-map` ordering hazard

Confirmed on today's binary:
- `49f03f179` qualified the tool's match strings, so a bare `(:wat::core::Some 1)` prints UNRESOLVED
  and is left unchanged;
- `bare-variant-to-qualified`, the next step in landing order, then leaves it POSITIONAL.

**Do not change either codemod's order or behaviour here.** Each tool is correct on its own fixture.
The failure is in COMPOSITION, and it belongs to the replay's first stone: a chain-composition check
(chain(base) vs main) that proves the order before any grok commit is replayed. It is tracked in
`SEAM.md`.

## EXPECTATIONS (batch 2d), fixed before the strike

| # | what | expected |
|---|---|---|
| D1 | git_show is loud | (l) a bogus path → RED naming the path; (m) a missing commit → RED naming the commit; verbatim text in the SCORE |
| D2 | before-lines checked everywhere | the `spec-before` entries backfilled; (n) RED on a hand-altered before-line in a mixed fixture |
| D3 | namespace-bare is history | its ORACLE has no `spec`; `history 72a1ac3d0 …` passes the gate |
| D4 | rule-record's true reason | `history` from readable extracted forms, OR spec with the lex-error reason recorded |
| D5 | the restore | the call's first arg is `":wat::Record::def"`; the fixture is `history 60d7d99a0 …`; near-misses stay |
| D6 | the audit table | every spec-bearing stem classified per landing file; each REPRODUCES → history; each rot → repaired, with its cause |
| D7 | untouched | `git diff d1c47b2a4 -- wat-scripts/fixes/positional-ctor-to-map.wat wat-scripts/fixes/bare-variant-to-qualified.wat` is empty |
| D8 | floor / clippy | 0 failed · 0 lines. Per-shard times reported |

## Tier

Commit on green. **Do not push. Do not touch main.** Yield once, with `SCORE-0b-batch-2d.md`.
