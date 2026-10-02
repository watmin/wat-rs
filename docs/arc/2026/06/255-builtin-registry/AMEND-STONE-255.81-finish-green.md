# AMEND — STONE 255.81: cutover 4b lands green

**Drawn 2026-10-02.** **Executor: grok via pulsare, working solo** (it runs the floor). Continues the stone at the four
local commits `f9a5757ab` (key flip, door deleted, refusal, printer), `d6605a481` (corpus call sites), `53eef498f`
(golden and fixture recapture), `6f6f702ad` (SCORE). Commit locally on `main` (`git add -- <paths>`, never `-A`);
**do not push**.

## Where it stands (read first)

`BRIEF-STONE-255.81-cutover-4b-the-new-spelling-is-the-key.md` and `SCORE-STONE-255.81-…` (the Sonnet executor's honest
account). **Floor RED: 6053 passed, 309 failed** (`.floor/2026-10-02T04-53-55Z`; types 95, function 52, rete 43, value
22, kernel 20, comms 17, process 16, services 12, resolve 8, lint 8, other 16). Census pre/post was not run.

## The orchestrator's findings since

1. **The "untraced" rot is traced:** `wat-scripts/probes/arc-170/probe-m1-ann-erase.wat:80` contains
   `(:wat::program::self-peer :wat::core::String :probe::PMsg)` literally, inside a nested program. A **type passed as an
   argument to a verb** is a type position that `types-to-wat-type.wat`'s rules A–F do not know, so 4a never converted
   it, and 4b's refusal now (correctly) fires. (The SCORE's "no literal occurrence" was wrong.)
2. **The keyword-bodied fn type must go in this stone.** `:wat::core::Fn(A)->B` and bare `:fn(A)->B` still parse
   (`src/types.rs:7064-7069`, `parse_fn_body`); arc 109 retired them only as printed text. 98 occurrences at
   `202eb5533` in `.wat` and `.rs`. Five STOP sites now fail because the flip retired the spelling **inside** the keyword
   (`wat-scripts/lib/wat-grep.wat:73,87`, three arc-170 probes, `j2-holon-rete-classify.wat`). **The builder's ruling
   stands:** the only arrow is the bracket fn type `[A :-> B]` (251.4c). Patching text inside the keyword is wrong.
3. **A mass golden recapture (`UPDATE_EDN=1`) is a migration that can forge its own green:** it writes whatever the
   program prints now, wrong output included. It must be audited.

## The work

1. **Audit the recapture (first):** for every golden or fixture changed in `53eef498f`, show that it differs from its
   `202eb5533` version **only** by the 24's spelling (normalize `wat.type/X` ↔ `:wat::core::X`, `wat.type/AST` ↔
   `:wat::WatAST`, then diff: must be empty). List every file that differs otherwise: each is **STOP-1**.
2. **Rule G, types as verb arguments:** find the verbs whose parameters are types (the refusal names every site: run the
   floor's failing programs or `wat --check` over the rotted list), and add the position to the recorded codemod as rule
   G, keyed on the verbs' declared signatures, not a hand list of names, with its replay case. Apply to `.wat`, and through
   `wat-fix-rust` to embedded wat.
3. **The fn type in a keyword dies:** a recorded codemod (`wat-scripts/fixes/fn-keyword-to-bracket.wat`) rewrites
   `:wat::core::Fn(A …)->R` and `:fn(A …)->R` (zero or one argument; more is already unparseable) to `[A :-> R]`, with the
   argument and return types themselves in `wat.type/` spelling. Apply to `.wat` and, through `wat-fix-rust`, to embedded
   wat. Then the parser **refuses** a `fn(`/`Fn(`-bodied keyword, and the keyword-bodied tuple `:(A,B)` with it
   (`parse_tuple_body`), naming the bracket/form remedy. The rete `NAMING_RULE_EXCEPTIONS` entries 255.81 added for the
   five rete container names (14 → 19) go away with the form they excused; if they cannot, say why.
4. **Finish the recapture:** the remaining reds that are only the printed spelling are re-captured from the program's new
   output and pass item 1's audit.
5. **Census:** a pre-image from `202eb5533` (check it out in a separate **clone**, e.g. `git clone --shared . /tmp/wat-pre`
   at that commit, never a worktree) and `scripts/replay/census.sh --diff` against the final tree.

## Gates

| what | how | expected |
|---|---|---|
| recapture audit | item 1 | every changed golden/fixture differs only by the spelling |
| no fn type in a keyword | search `.wat` + `.rs` for `Fn(` / `:fn(` type keywords | only the refusal, its remedy, its tests, history comments |
| embedded wat | `wat-fix-rust` dry run with each codemod over every tracked `.rs` | `0 changed` |
| census | item 5 | no rc flips except programs whose only change is spelling (list each) |
| release floor | `scripts/floor.sh`, in the foreground, nothing else running | all passed; the count against 6362 at `5b4d963b2` (`.floor/2026-10-01T21-57-10Z`), plus new tests |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |

## Reds and STOPs

- A red caused by this stone's own change (spelling in a message, a site a rule missed): capture it **verbatim** from
  `.floor/<stamp>/`, cure it, run a **new** floor. Never re-run unchanged code for a green.
- **STOP-1:** a recaptured golden or fixture differs by more than the spelling. List each and STOP.
- **STOP-2:** a red that is not a spelling (a type no longer checks, a value computes differently, a dispatch misses).
  Quote it and STOP.
- **STOP-3:** a `Fn(`-bodied keyword the codemod cannot express as a bracket (two or more arguments, a nested keyword
  body). List it and STOP.
- A STOP means STOP.

## Doctrine

`holon/CLAUDE.md` binds you: `.wat` and embedded wat move only by the recorded codemods; read `wat/fix.wat`'s header
(STASH-DANCE) before making a form illegal in the same stone as its codemod. Capture `rc=$?` on the next statement. Never
wait with `pgrep -f`. Never write a number, file:line or example you did not measure. If this amendment contradicts the
code, the code wins: say so. Append to `SCORE-STONE-255.81-…`, commit, **do not push**.
