# Arc 212 — runtime quasiquote substitution inside Vector<WatAST> constructor

**Status:** OPEN 2026-05-18 — opened to fix `t6_spawn_process_factory_with_capture_round_trips` whose failure was made readably diagnosable by arc 211's panic-as-EDN tooling. Arc 211 closure depends on this arc's resolution per the **tooling-proven-by-use** discipline (see INTERSTITIAL § 2026-05-18 (post-arc-211e)).

**Priority:** BLOCKING arc 211 INSCRIPTION + the broader arc 170 closure cascade.

## Origin

Arc 170 slice 6 (the substrate redesign retiring closure-extract) inscribed t6 as a known "downstream stone" in its SCORE:

> *"T6 substrate-discovery gap — `wat_arc170_program_contracts::t6_spawn_process_factory_with_capture_round_trips` originally tested closure-capture-across-fork. New substrate retires closure-extract; substrate-equivalent is runtime AST template construction via `:wat::core::quasiquote` + `:wat::core::unquote`. T6's migration to this shape FAILS — runtime quasiquote inside `(:wat::core::Vector :wat::WatAST ...)` constructor does not substitute unquoted symbols. Surfaced as downstream stone; T6's failure preserved with documenting comment."*

Arc 211c (panic_any! audit) confirmed t6 as a known consistent failure. Arc 211d's revert + Category D fixes addressed the dup-removal regression but NOT this substrate-discovery-gap. Arc 211e dedup work didn't touch the macro substrate.

**Today (2026-05-18, post-211e):** running `arc170_program_contracts` binary with `--no-fail-fast` shows 23/24 tests pass; the ONE failure is t6 with exactly the structured EDN diagnostic that arc 211b shipped:

```
#wat.kernel/ProcessPanics [#wat.kernel.ProcessDiedError/RuntimeError 
  ["<entry>:11:61: unknown function: :wat::core::unquote"]]
```

The substrate is now telling us — readably, via the panic-EDN format — exactly what's broken. This arc fixes it.

## Scope

**In scope:**
- Extend substrate's quasiquote/unquote substitution to work inside `(:wat::core::Vector :wat::WatAST ...)` constructor calls
- Ship the fix
- Verify t6 passes in isolation + under `--no-fail-fast` run
- SCORE inscribes the actual mechanism (where in macros.rs / runtime.rs the substitution path needed extension)

**Out of scope:**
- Broader quasiquote behavior on other container shapes (decide per honest diagnostic data; sibling arc if surfaces)
- Closure-extract retirement work (slice 6 territory; already shipped)
- t6 test rewrite (the test exercises the intended pattern; substrate should support it)

## Closure conditions

1. Substrate change ships
2. `t6_spawn_process_factory_with_capture_round_trips` passes in isolation
3. `arc170_program_contracts` binary passes 24/24 with `--no-fail-fast`
4. SCORE doc inscribes mechanism + delta
5. Arc 211 closure becomes unblocked (one of two pre-conditions; arc 213 is the other)

## Cross-references

- Arc 170 SCORE-SLICE-6 (the original substrate-discovery-gap inscription)
- Arc 211 SCORE-211C-AUDIT (confirmed t6 as consistent failure)
- Arc 211 DESIGN § "Tooling-proven-by-use closure condition" (the blocking relationship)
- Arc 211 INSCRIPTION (pending; awaits this arc)
- INTERSTITIAL § 2026-05-18 (post-arc-211e) "Tooling proven by use — closure-discipline extension"
- `tests/wat_arc170_program_contracts.rs:483` (t6 source)
- t6 panic output: `<entry>:11:61: unknown function: :wat::core::unquote`

## Tooling-proven-by-use principle

This arc serves dual purpose:
1. **Fix t6** (substrate correctness)
2. **PROVE arc 211's tooling enabled this fix** (substrate-tooling-validation)

The arc 211 panic-as-EDN format made t6's failure honestly diagnosable. Pre-arc-211, the same failure surfaced as `Box<dyn Any>` placeholder — a kind of "we know SOMETHING is broken but cannot say what." Arc 212's existence — its ability to scope precisely from the visible diagnostic — IS the proof.

When arc 212 closes, the SCORE will reference the EDN diagnostic as the entry point. That reference is the validation evidence arc 211 needs.
