# BRIEF — STONE 255.72: the wall has no exceptions — type the last two template constructors

**Drawn 2026-10-01 against `main` @ `39eb6d4b5`.** **Executor: a Sonnet subagent.** A small strike: two sites, maybe a
run-time lookup, tests. Commit locally on `main` (`git add -- <paths>`, never `-A`); **do not push**. Your final message is
your report.

## Where this sits

255.71 made an untyped collection constructor call illegal (`WEIGH-STONE-255.71-the-untyped-constructor-wall.md`). Two
bracket-less sites remain, the wall's only exceptions. Both are **templates that build code** at run time:

1. **`wat-scripts/probes/arc-170/probe-s3b-astsplice.wat:75`**: a quasiquoted runner with
   `(:wat::core::Tuple (:wat::core::first pair) (:probe::__work (:wat::core::second pair)))`. The probe computes the concrete
   types as text a few lines above (`":(wat::core::i64," arg-t ")"`, under the comment *"splicing the concrete types"*).
2. **`wat/rete/oracle/accum-pass.wat:135`**: `(acc-hd (:wat::core::PersistentVector (:wat::core::unquote-splicing vals)))`,
   built by quasiquote and run with `eval-ast!`. `vals` come from rule bindings; their type depends on the rule's custom
   accumulator fold `acc-hd`.

## The rulings

- The builder's wall (2026-09-28): every constructor is `(wat.type/X :- [T…] items…)`; untyped is illegal.
- Types are headers on function definitions; a value's type is pinned by a typed consumer (2026-09-27).
- For site 2, **A1** (orchestrator's four questions, pending the measurement below): splice the fold's **own declared**
  element type into the bracket, read off `acc-hd`'s signature. **A2**, `wat.type/Value` as the element type, failed
  Honest: a fold declared over `i64` would receive a vector of `Value`, a guess that hides a mismatch.

## The work

1. **Site 1:** splice the probe's own concrete types into the template's bracket, as `wat/bracket.wat:491`'s `~ret-ty`
   precedent does: `(wat.type/Tuple :- [<first's type> <__work's return type>] …)`, using the type text the probe already
   builds (convert it to a type form the bracket accepts; do not re-derive it by hand). The probe must still do what it
   proves today: run it before and after and compare its output.
2. **Site 2, measure first:** can the oracle read `acc-hd`'s declared parameter type at run time? `acc-hd` is the custom
   fold's head (a keyword or symbol naming a function). Find what the runtime offers (`signature-of`, `metadata-of`,
   `type-of`, the function registry, reflection verbs in `src/reflect/`), and what it returns for a user fold declared
   over a vector of some `T`. Report the call and its result for a real fold from the oracle's own tests.
   - **If it is readable:** splice that element type into the bracket (A1). The oracle's differential tests must agree
     with the native rete before and after.
   - **If it is not readable:** STOP-1 (below). Do not use `wat.type/Value`.
3. **The wall's reach gate** then counts **0** bracket-less sites outside `.wat.bad` and `.wat.golden`. If a named-exception
   list exists for the two sites (255.71 may have recorded one), empty it.
4. **Tests:** whatever proves each site's behaviour is unchanged (the probe's own output; the oracle's differential rows).

## Gates

| what | how | expected |
|---|---|---|
| release floor | `scripts/floor.sh`, **one run at a time, in the foreground, nothing else running** | all passed; the count against 6235 at `a34dc3406` |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |
| the wall's reach | a corpus-wide search for an untyped collection constructor | **0** outside `.wat.bad` and `.wat.golden` |

## Reds and STOPs

- A red caused by this stone's own change: capture it **verbatim** from `.floor/<stamp>/`, cure it, run a **new** floor.
  Never re-run unchanged code for a green; never call a red "pre-existing" without the prior green floor's line
  (`.floor/2026-10-01T00-21-52Z`, 6235/6235).
- **STOP-1:** `acc-hd`'s element type cannot be read at run time. Report exactly what the runtime offers and what it
  returns, and STOP on site 2. Finish site 1 anyway.
- **STOP-2:** typing a site changes what it computes (the probe's output, or the oracle's agreement with the native
  rete). Quote it and STOP.
- A STOP means STOP.

## Doctrine

`holon/CLAUDE.md` binds you (`.wat` edits: these two are template sites a codemod cannot reach; edit them directly and
say so, as 255.69/.71 did for `wat/bracket.wat:491`). Capture `rc=$?` on the next statement. Never wait with `pgrep -f`.
Never write a number, file:line or example you did not measure. If this brief contradicts the code, the code wins: say
so. Write `SCORE-STONE-255.72-the-wall-has-no-exceptions.md` beside this brief, commit it, **do not push**.
