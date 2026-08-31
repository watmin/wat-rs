# SCORE — excursus 002 stone 1: the creation-scope escape wall

**STRUCK.** Executor: grok, 2026-08-31. Scored against my OWN re-run of every row, never the
report. One row failed, and **the fault was in the brief, not the strike.**

```
Summary [ 308.372s] 5132 tests run: 5132 passed (3 slow), 15 skipped
FLOOR=0
```

5132 = 5131 + `probe_ex002_creation_escape`. Floor green after the specification fix below.

## The rows

| # | what | result |
|---|---|---|
| 1 | real escape rejected | ✅ `HandleCreationEscape`, names `:red::dial-and-drop` |
| 2 | `conn` helper still compiles | ✅ never named — the rule is keyed on creation, not the param |
| 3 | 16 safe sites compile | ✅ re-run over every Peer-returning file: 13 clean, 1 rejected (the target). stdlib `stdio-connect-*` clean |
| 4 | census still 18 | ✅ 18 |
| 5 | 1a AND 1b both fire | ✅ two errors, `135:5` (let's value) and `132:3` (function body) |
| 6 | no runtime change | ✅ `git diff --stat src/runtime.rs` empty |
| 7 | severed gate holds | ✅ 30/30 on my own stress run, after the rune |
| 8 | rune is a rune | ✅ `rune:check(handle-lifetime-creation-escape)` with a stated reason; function body untouched |
| 9 | floor | ⛔→✅ red as struck; green after the fix below |
| 10 | case 2 not smuggled | ✅ still no tail-position concept — every `tail` hit is `strip_prefix` |
| 11 | the error teaches | ✅ names the service, the creating span AND the escape span |

## ★ Row 9 was MY error, and the executor's refusal was the right call

The strike came back with a red floor: `every_wat_scripts_file_loads` failing on
`wat-scripts/scratch-pad/probe-handle-to-surface-relation.wat`, because the wall correctly rejects
the escape that file carries. The executor declined to silence it, reporting: *"Row 1 requires that
`--check` to REJECT; a rune there would make a green floor that fires on nothing."*

**That is exactly right, and it is the trap EXPECTATIONS named in advance** — a wall that fires on
nothing passes every row but the first. Runing the acceptance criterion would have destroyed the
only proof the wall works.

The contradiction was in my BRIEF: I put the must-be-REJECTED target under `wat-scripts/`, whose
loader gate type-checks every file and therefore demands it PASS. Two requirements that cannot both
hold, by construction, and the floor went red the moment the wall started working.

**The repo already had the answer and I walked past it**: `docs/arc/2026/06/278-rules-engine/probes/red-*.wat`.
Deliberately-red probes live in a `probes/` directory in their own arc/excursus, ungated. I noticed
that convention EARLIER IN THE SAME SESSION, while looking for somewhere to put a red probe, and
still wrote the target into `wat-scripts/scratch-pad/` — because CLAUDE.md says scratch `.wat` goes
there, and I never reconciled the two rules.

Fixed by splitting along the convention rather than by weakening anything:
- `docs/excursus/2026/08/002-handle-lifetime-wall/probes/red-creation-escape.wat` — the red target,
  self-contained, carrying `:red::conn` (must compile) beside `:red::dial-and-drop` (must not).
- `wat-scripts/scratch-pad/probe-handle-to-surface-relation.wat` — keeps the GREEN feasibility half
  (the Handle→surface relation), which the loader gate goes on proving. The move is recorded in
  the file, with the reason.
- `tests/services/probe_ex002_creation_escape.rs` repointed at the new path.

## Deltas

- **The executor added a gate I did not brief** — `probe_ex002_creation_escape.rs` — and it is
  better than the row it satisfies. It matches `CheckErrorKind::HandleCreationEscape` structurally
  rather than on message text, asserts the escape IS named and `conn` is NOT, and its panic message
  states the trap outright: *"a green check here means the wall fired on nothing."*
- **The implementation is sounder than the brief's sketch.** The brief said a `/start` call names
  its service in its FQDN. The strike does not consult the name at all: a call creates a Handle when
  its SCHEME returns a service Handle (an aggregate with `handle` + `addr`, `addr` being
  `(Address :- [Op Reply …])`) and takes none. That is structural where mine was nominal, and it
  cannot be fooled by a user function named `…/start`.
- `(Handle :- [Shared])` matched on the parametric head, per the trap-door note. Had it matched a
  bare path the wall would have fired on nothing — and rows 2, 3, 9 would all still have passed.

## Not done, and named rather than deferred

Case 2 — a peer leaving via a TAIL CALL — is untouched, per STOP-4. The checker has no notion of
tail position. That is stone 2 and it remains undrawn.
