# FINDINGS — the chain-composition check

The replay converts each grok-rete `.wat` by running main's recorded migrations over it. Stone 0 made
every migration correct **alone**, against its fixture. This check proves the chain **in composition**:
chain(file@base) is compared with file@main over the 1,001 files main changed since `de827fb4c`, plus
the 488 files main never touched as a control.

The runner is `bootstrap/composition/run.sh`. It uses the FROZEN binary
`bootstrap/wat-replay-base-8a5b7eb20` and the exported tree `bootstrap/composition/tree-8a5b7eb20`, runs
at nice 19, and never touches the working tree. Runs land in `bootstrap/composition/runs/`. Before the
full run, it was timed on one file: 54 s, 46 s of it `positional-ctor-to-map`.

## Finding 1 — a later sweep co-updated an EARLIER tool (silent)

- `positional-ctor-to-map` (landing-order line 25) matches the qualified `:wat::core::Option::Some`,
  because `49f03f179` (step 26's own sweep) rewrote its match strings.
- A bare `(:wat::core::Some 1)` therefore prints UNRESOLVED and is left unchanged. Step 26
  (`bare-variant-to-qualified`) then leaves it **positional**, a form main retired.
- Each tool's fixture passes; only the composition fails.

## Finding 2 — an eval-based tool runs on TODAY's substrate (loud)

- Baseline run (landing order), step 23 `match-arm-to-bracket-map-pattern`, rc=2 on
  `tests/process/probe_arc278_init_crash_reason.wat`:
  `assertion-failed! takes kwargs … the positional form is retired` (`wat/kernel/assertion.wat:34`).
- Its `try-type-of` EVALUATES the file's declarations to learn field names. Today's runtime has retired
  a form that a LATER step (24, `assertion-failed-to-kwargs`) migrates.
- `order-v2` = landing order + two evidence-backed moves: assertion-failed before match-arm, and
  bare-variant before positional-ctor. With v2, match-arm completes (rc 0, 478 changed).

## Finding 3 — one unmigrated BODY poisons a whole file's type resolution

- v2 step 24: match-arm has **403 UNRESOLVED over 160 names in 77 files**. They are nearly all
  `<Svc>::<Method>Response::{Ok,RequestTooLarge,RequestMalformed}`.
- Main converted these arms (e.g. `tests/comms/probe_arc293_W2f_process_dials_thread.wat`), so they are
  CHAIN residuals.
- Mechanism: `try-type-of` = `eval-with-defs! (type-of E)` over ALL top-level `file-decls`.
  `decl-head?` admits pure type declarations AND `defservice` / `sift-rules-defsvc` / `defmacro`. A
  `defservice` whose `:impls` hold a base-era positional `::` constructor (`unknown callee:
  :probe::Echo::EchoResponse::Ok`) fails, so the whole eval fails and the fallback has no decls.
- That is a potential **cycle**: match-arm needs positional-ctor's and the dot flip's output, and
  positional-ctor also evaluates decls. Reordering cannot break a cycle.

### Two candidates, probed

- **(A) The era binary for the eval-based steps.** `git archive 480f38d05`, then `cargo build` in
  `bootstrap/era/` (needs the link `bootstrap/era/holon-rs` → `../holon-rs`; 52 s). The era match-arm
  on the era binary, on the example after steps 1–23: **UNRESOLVED 0, bracket arms 3 = main**. Today's
  gives 3 / 0.
  - Cost: positional-ctor's era text predates A′/(c), ~65 s/file.
  - Cost: era builds depend on today's `holon-rs` still compiling.
- **(B) Evaluate ONLY pure type declarations, at any depth; never bodies.** `file-decls` becomes a deep
  walk collecting `defenum`/`defrecord`/`defstruct`/`defsurface`/`newtype`/`typealias`/`typeunion`
  (`bootstrap/era/probe-B/match-arm-typedecls-only.wat`). Result on the example: **UNRESOLVED 0,
  bracket arms 3 = main**, on the frozen binary.
  - Indicator: 325 of 333 (name, file) pairs have an explicit in-file `defenum`, so (B) should reach
    ~98%.
  - **At scale (77 files, frozen binary, `probe-B/final.sh`): today 403 UNRESOLVED → (B) 16. (B)
    converted more arms in 73/77 files and fewer in none.** (Two earlier attempts measured nothing:
    one harness kill, and one relative-path bug in my runner that fed empty stdin.)
  - (B)'s 16, each classified:

    | n | where | what | class |
    |---|---|---|---|
    | 1 | `probe_diagnostic_c3…` | template arm in a `defmacro` | **not a residual**: main has 0 bracket arms there too |
    | 9 | `wat/service.wat` | `generated-template-without-fqdn` | **main hand content**: landing `480f38d05` hand-authored `init-arg-map-ast` in service.wat, and the era tool on `480f38d05^` cannot reproduce those 29 arms without it |
    | 1 | `probe_arc265_acronym_registry_svc` | `Waf::Op::CreateWebACL` | **(B) gap: acronyms.** Adding `:wat::string::declare-acronyms` to (B)'s heads → 0 UNRESOLVED, `{:req req}` = main (control: unmodified (B) on a copy reproduces the 1). `probe-D/`. ⚠ An earlier line here said "the era tool on `480f38d05^` reproduces the landing byte-for-byte". That was VACUOUS: main converted this arm at `498fbc1d6`, BEFORE `480f38d05`, so the era tool had nothing to change (`in` == `.orig`). The corpus conversion is split across `498fbc1d6` and `480f38d05`. |
    | 5 | `probe-m1-ann-erase2` | `PoolMsg`, `Echo::EchoResponse` | **(B) gap: program scope.** The file holds TWO programs: a parent, and a child inside `(:wat::core::forms …)`. Each declares its own `PoolMsg`/`Echo` ("SEPARATE typecheck universes", its header). The deep collector merges both, so the eval fails on the duplicate. Removing the child's copies → 0 UNRESOLVED. The real fix: a nested `forms` is its own scope. `probe-D/` |

### ⛔ (B) at FULL scale (1489 files, step 24's input, `bootstrap/era/probe-G/ab.sh`): it REGRESSES on 2 files

The 77-file run above covered only files that today's tool already failed on, so it could not see a
loss elsewhere. Its "fewer in none" was true of those 77 and is not a no-regression result.

| | today | (B)+acronyms |
|---|---|---|
| UNRESOLVED | 403 | 35 |
| files where (B) converted MORE arms | | 74 |
| files where (B) LOST or CHANGED an arm today converts | | **2** |

- The two are `tests/services/probe_arc278_sift_rules{,_arena}.wat`, 10 arms each. Their
  `SiftRulesResponse` enum is GENERATED by `:wat::query::sift-rules-defsvc`, a declaration macro that
  today's `decl-head?` evaluates. match-arm's own header records that it "freezes" alone. (B) drops
  it along with the body-carrying macros that poison the eval.
- So "pure type declarations only" is too narrow. What poisons the eval is a BODY (a `defservice`'s
  `:impls`), not a declaration macro as such.
- positional-ctor already has the right SHAPE, a fallback LADDER in its `try-type-of`
  (`positional-ctor-to-map.wat:166-187`): all declarations, then without `defservice`, then plain type
  declarations only, then stdlib. match-arm's `try-type-of` has one step (all, then empty). A ladder
  keeps every arm today converts and adds (B)'s recoveries: to be PROBED before it is briefed.
- (B)'s 35: 20 are this regression · 5 the `probe-m1` two-program scope gap · 9 main's hand-authored
  `service.wat` template · 1 c3, which matches main.

### ✓ The LADDER, measured at full scale (`bootstrap/era/probe-L/`, 2026-09-13 04:05–04:21Z)

`ma-ladder.wat` = today's match-arm plus one rung. At `fmap-for-src:864`, try today's declarations;
only when they cannot answer, try pure type declarations + `declare-acronyms` at any depth.

| | today | pure (B) | ladder |
|---|---|---|---|
| UNRESOLVED | 403 | 35 | **15** |
| arms LOST vs today | — | 20 (2 files) | **0** |
| files where it gained arms | — | 74 | **74** |
| arms pure (B) converted that the ladder did not | — | — | **0** |

The 15 left: 9 main's hand-authored `service.wat` template · 1 c3 (matches main) · 5 the `probe-m1`
two-program scope gap (the brief's scope rule). A positional-ctor ladder (`pc-L.wat`, same rung
in `fill-paths`) smoke-tests clean (acronym file +2 ctors = main, 0 lost); its full A/B and the chain
under the ladder (v3L) are running (`probe-L/pc2.sh`). The pure-(B) positional-ctor and v3 runs were
stopped as superseded.

## Finding 4 — positional-ctor's UNRESOLVED count is mostly noise (a CASE rule)

- `pascal-leaf?` asks `(= c (to-uppercase c))` of the leaf's first character. That is TRUE for `+`, `=`,
  `*`, `>`, so `(:wat::i64::+ …)` and `(:wat::core::= …)` are reported. It also reports collection
  ctors (`PersistentVector`) and rete patterns inside `quote`.
- v2 step 26 at 694 files: 1,896 reports over 306 names; the top ones are `PersistentVector` (270),
  `:wat::i64::+` (155) and `:wat::core::=` (128).
- It gates only the REPORT, never an edit, so the output is unaffected. But the channel cannot tell a
  real miss from noise, and it is a case rule, which the ruling forbids ("character case bears no
  meaning"). **For this step, the residual comparison against main is the instrument, not the count.**

## Finding 5 — the LANDING's match-arm leaked a type ACROSS FILES, and main carries it

- Main's `probe-m1-ann-erase2.wat` arms read `[:probe::PoolMsg.Setup {:deps addr}]` and
  `[:probe::PoolMsg.Work {:pair s}]`. Its own `PoolMsg` (both copies) declares `addr` and `s`.
  `deps`/`pair` are `probe-m1-phantom-d.wat`'s `PoolMsg :- [D I]`.
- **Reproduced:** the era tool (`480f38d05` tree and binary) on `480f38d05^` of erase2 ALONE writes
  `{:addr addr}` / `{:s s}`. With `phantom-d` in the same run it writes `{:deps addr}` /
  `{:pair s}`, byte-matching the landing. The landing changed both files. `probe-E/`
- So it is TOOL output, not a hand edit: one run's type state reached another file. The mechanism
  inside the tool is not traced yet (erase2 was listed FIRST, so it is not simple order).
- It is masked on main: the child program dies first at startup on an undeclared `:probe::CMsg`
  (main's binary, run: `UnknownNamedType … :probe::CMsg`). `CMsg` is declared only in the sibling
  `probe-m1-ann-erase.wat`. Grok-rete never touched erase2, so the file is broken on BOTH sides and
  its `EXPECT (green): "echo:z"` is false.
- **The class, not the case:** every arm main converted in the landing run could carry a same-named
  type from another file. The chain resolves in-file only, so it writes the declared fields. **The
  composition RESULT's residuals where chain = in-file declaration and main ≠ are this census.**

## v2 RESULT (order-v2, TODAY's match-arm and positional-ctor) — 2026-09-13 03:43Z

`bootstrap/composition/runs/2026-09-13T02-15-31Z-order-v2/`: all 27 steps rc=0.

| set | reproduces main byte-for-byte | residual |
|---|---|---|
| worklist (1001 files main changed) | 774 | 227: 50 comment-only (main's hand prose), 177 carrying code |
| control (488 files main never touched) | 486 | 2 (finding 6) |

A residual on a file grok-rete does not touch is harmless to the replay. For a file grok modifies,
`git merge-file` keeps main's side. **What a chain failure costs is grok's own lines, and grok's NEW
files.** The code residuals are classified on the chain UNDER THE RULING, (B), not on v2.

## Finding 6 — the two CONTROL residuals: one main miss, one tool over-match (both verified on main's binary)

- `docs/arc/2026/06/278-rules-engine/probes/surface-field-dispatch.wat`: main spells
  `:wat::core::i64::+`, which main's own `src/remedy/retirement.rs:190` retires in favour of
  `:wat::i64::+`. The chain renames it. **The chain is right; main never migrated it.** No gate
  loads `docs/**/*.wat`, and the file is broken on main anyway (`--check`: `MalformedDecl … defsurface
  … expected :nature`).
- `tests/macros/probe_arc241_stone17_defmacro_canonical_c03.wat`: the template
  `` `(:wat::core::Vector ~@items)`` becomes `` `(:wat::core::Vector :- [~@items])``.
  - ⚠ **CORRECTED 2026-09-13:** the first version of this finding (and commit `255c14881`'s message)
    named `one-param-spec`. That was read off the OUTPUT's shape. Replaying the file one step at a time
    (`bootstrap/era/probe-J/`) shows **step 14, `mandatory-typed-quasiquote-residual`**, makes the edit;
    `one-param-spec` never touches it.
  - The rule (its header): for the three MANDATORY-typed heads (Vector, HashMap, HashSet), "args.length
    == declared-arity is unambiguous evidence of an attempted param-spec", so an unquote-wrapped slot is
    wrapped in `:- [...]`. That holds for `~ty`. **It is false for a SPLICE:** `~@items` counts as one
    argument but expands to N forms, and may even carry its own `:- [T]`. The header includes
    `unquote-splicing` explicitly, and has since its landing `284cd7c93`. That landing's sweep touched
    three sites (`wat/bracket.wat`, `wat/service.wat`, a census scratch), so the splice case was never
    exercised until the chain ran the tool over the whole corpus.
  - Expanded on main's binary (`bootstrap/era/probe-H/`), with `(variadic-wrap 1 2 3)`:
    - chain: `type param-spec :- [...] must declare exactly one type (T); got 3`;
    - main: ALSO invalid, `first argument must be a (Head :- [T]) type param-spec`. Main's template has
      been unexpandable since the param-spec wall. Its test (`contract_03`, `startup_from_file` only)
      never expands it, an acceptance row the defect satisfies (both `--check` rc 0).
  - **Both sides are wrong.**

## The residuals, asked of MAIN's binary (v2) — `bootstrap/era/probe-K/classify.sh`

For each residual, `--check` both main's version and the chain's (each in a mirror of the repo
layout), on main's binary `bootstrap/wat-main-a3218644d`:

| class | files | meaning |
|---|---|---|
| CHAIN-FAILS | 127 | main passes, chain fails: a LOUD chain failure |
| BOTH-PASS | 74 | hand content, or a silent difference |
| BOTH-FAIL-SAME | 22 | broken on main already |
| BOTH-FAIL-DIFF | 5 | both broken, differently |
| MAIN-FAILS-ONLY | 1 | the chain fixes main |

- ⚠ Instrument limit: a stdlib file (`wat/*.wat`) that differs from the binary's own baked stdlib
  reports `DuplicateMacro`/`DuplicateType`/`ReservedPrefix`. Its classification is NOT a verdict;
  those files need a different judge.
- The 124 non-stdlib CHAIN-FAILS, by inner error: **173** × "retired clause; a match arm is a
  bracket" (positional arms match-arm left UNRESOLVED; (B)'s target) · **~50** × "variant arm head
  … is not namespaced" (finding 7) · a tail of ~30 (non-exhaustive open matches, map literal as a
  match sub-pattern, TypeMismatch, UnknownCallee, retired names, ArityMismatch). v3 re-asks under (B).

## Finding 7 — `variant-separator-to-dot` is CENSUS-bound: it flips only the 382 pairs main had

- It consumes `docs/arc/2026/06/255-builtin-registry/dot-flip-phase1-pairs.txt` verbatim: 382
  (old, new) pairs from main's corpus on 2026-09-10. A variant not on the list keeps `::`, and the
  tool prints nothing (its log has no UNRESOLVED channel).
- Sample: `tests/rete/probe_constructor_meta_enum_variant_green.wat` declares `:cg::Status` in the
  file. It is not on the list; main flipped it later, in `c15d76e01` (dot flip ⑦).
- **Main refuses it LOUDLY** (probe `bootstrap/era/probe-H/sep-{colon,dot}.wat`, main's binary):
  `(:u::E::A {:x 1})` with a `[:u::E::A {:x x} x]` arm → rc 1; the dot spelling → rc 0. So a miss
  is a `--check` failure (STOP-2 at replay time), not silent.
- **The replay exposure:** every enum grok-rete declares in its own files is off the census by
  construction. Which names are variants must come from the program's declarations and the
  registry, the same door as finding 3, never a frozen list.
- ⚠ **Main's refusal message is STALE:** "variant arm head `:u::E::A` is not namespaced; write
  `<enum>::<Variant>`". The remedy names the spelling it just refused.
- **It is a CLASS: text still teaches `::` as the variant separator after the flip.** The door
  itself composes with a dot: `crates/wat-reader/src/identifier.rs:362`
  `compose_variant` → `format!("{enum_path}.{variant_name}")`, and `decompose_variant` splits on `.`.
  Census (code strings, `src/` `crates/` `wat/`; no test pins any of them):
  - `src/match_arm.rs:108` (`<enum>::<Variant>`) and `:187` (`a qualified Type::Variant FQDN`);
  - `src/check.rs:2877` (`variant-parent-of :Ns::Enum::Variant`), `:6955`, `:7253`, `:7755`
    (`must be <enum>::<Variant>`), and `:7508` (`<enum>::<Variant>` or `:None`; the bare `:None` is
    ALSO retired);
  - the lint's own help text, `tests/lint/one_variant_separator.rs:260`: `compose_variant(…) ->
    {enum}::{variant}`;
  - the door's own doc comment, `identifier.rs:366-371`: "`compose_variant` writes `::`; this reads
    `::`". The code does the opposite.
  - Each message guards a different predicate. The honest fix drives each refusal with a probe and
    checks that following its remedy passes, one site at a time; it is not a text sweep.

## After 2b — three known flaws its verification surfaced (orchestrator re-run, 2026-09-13)

- **Three refusal arms are unreachable from wat.** grok found that `match_arm.rs:187`'s fallback
  (callers pass only the five retired bares), `check.rs:6955` and `check.rs:7253` (both behind
  `is_namespaced_variant`, then re-asking `decompose_variant`) cannot be fired by any input. 2b pins
  them by their source text. They are dead refusal code, and dead code should go, not stay pinned.
- **~21 doc/comment lines still teach `::` for a variant.** SCORE-2b's E3 listed 4. The orchestrator's
  census of `src/`/`crates/` comment lines (same pattern) finds: `closure_extract.rs:1338,1341`,
  `freeze/env.rs:207`, `rete/expr_ir.rs:58,602`, `record/construct.rs:123,245,249`,
  `reflect/verbs.rs:1585` (the documented signature of a PUBLIC verb), `declare/register.rs:1323`,
  `rete/validate.rs:82,84,1375`, `runtime.rs:1732,8781,8842,8979,13593,13623`, `types.rs:695`. E3 was
  scoped to refusal TEXT and holds; these are prose. Each is either a history note (which is kept) or
  a live description (which is updated), and has to be judged one at a time.
- **c03's template now hardcodes `:wat::core::i64`.** `variadic-wrap` =
  `` `(:wat::core::Vector :- [:wat::core::i64] ~@items)``, so a macro whose name says "wrap the items"
  wraps only integers. It is legal under the param-spec wall (the Vector head is mandatory-typed), and
  the test's subject is the `& items` rest binder. It is a narrowing, recorded for the builder.

## Finding 9 — why the eval-based codemods are slow: two full-world freezes per QUESTION

- **Measured** (`bootstrap/era/probe-P/pc-count.wat`: today's positional-ctor plus a counter in
  `try-type-of`):

  | input | time | `try-type-of` calls | per call |
  |---|---|---|---|
  | `tests/services/probe_arc278_sift_rules.wat` | 57.9 s | 125 | ~0.46 s |
  | `tests/types/enums_tagged_variant.wat` (small) | 17.6 s | 38 | ~0.46 s |
  | a no-op program (startup alone) | 0.26 s | — | — |

- **The mechanism** (read). `eval-with-defs!` → `eval_form_against_defs` (`src/runtime.rs:12360`),
  whose `freeze_forms` runs `startup_from_forms(_with_session)`, the WHOLE pipeline, stdlib included.
  It runs it TWICE per call: once for a BASELINE (`freeze_forms(defs.clone())`, to measure the
  session's residue length), and once with the form. That is REPL machinery (session TCO, residue
  diffing), paid by a codemod asking one question.
- **Waste 1: stdlib questions.** positional-ctor's pass 1 resolves every corpus-invariant path
  against stdlib, one `eval-with-defs!` each: 1,504 in v2, about 11.5 of its 64 minutes. The running
  program can answer them directly: `:wat::runtime::type-of` takes a computed keyword (its doc,
  `src/reflect/verbs.rs:1489`), guarded by `:wat::runtime::is-type?`.
- **Waste 2: one freeze per PATH.** A file's declarations do not change between its paths, yet each
  path is a new freeze (two). positional-ctor's fallback chain retries up to four times on a poisoned
  file; the ladder adds one more rung.
- **What it costs the replay.** `convert.sh` runs the chain per file per replayed commit. The
  pilot measured 29 s per probe and 51 s for `cache.wat`, positional-ctor dominating. That is a large
  share of the ~16.5 h estimate for the remaining 641 commits.
- **The shape that removes it.** The shared door (ruling 3) answers stdlib paths in the running
  world, and asks all of one program's questions in ONE freeze per rung. Whether a lighter substrate
  door exists (freeze once, no baseline) is NOT yet known; `eval-ast!` refuses `defenum`, per
  match-arm's own header.

## Finding 10 — the property we need does not demand the slowness: a MISSING DOOR

- The eval-based codemods need three things from a question:
  1. a program's declarations registered as the substrate registers them (macros, acronyms,
     generated types);
  2. ISOLATION: a scratch world, never the tool's own and never shared across files (finding 5 is
     the cost of breaking this);
  3. nothing else: no body check, no `main`, no REPL baseline.

  `eval-with-defs!` is the only verb with (1) + (2). It pays for isolation with two cold starts, each
  rebuilding stdlib from its forms (`src/freeze/env.rs:107`, no cache across startups) and checking
  every body.
- **Isolation does not need a cold start.** The environment types are `Clone`: `TypeEnv`
  (`src/types.rs:547`), `SymbolTable` (`src/value/symbol_table.rs:32`), `MacroRegistry`
  (`src/macros/registry.rs:54`). `build_env` registers stdlib BEFORE user forms at each step
  (`register_stdlib_defmacros` then `register_defmacros(user)`; `register_stdlib_types` then
  `register_types(user)`), so the stdlib half is identical every time. So: build it once per
  process, clone it per program, register that program's declarations on the clone, read the types.
  Isolation holds by construction; the cost scales with the program, not with stdlib.
- **And no body is checked, so nothing can POISON it.** Finding 3's poison is a check-time error
  (`unknown callee`, `check_program`, step 8), and types are registered at step 5. **The ladder
  exists only because the one door we have checks bodies.**
- Unknown until probed INSIDE the crate (`build_env` is `pub(crate)`, `src/freeze.rs:52`):
  - whether registration alone survives an unmigrated body, or step 7 (`resolve_references`)
    already trips on one;
  - how cleanly each step's stdlib half separates from its user half.

## Finding 11 — BATCH the questions: 10–33× per file, byte-identical, no new intrinsic

`bootstrap/era/probe-Q/pc-batch.wat` is today's positional-ctor with ONLY `fill-paths` replaced. One
generated program per batch of paths: each path appears as a LITERAL keyword in
`(if (is-type? :P) (Some (type-of :P)) None)`, so a non-type path yields `None` instead of failing,
and `is-type?`'s literal-keyword check is satisfied. It is evaluated ONCE per (paths, declarations),
with try-type-of's fallback chain applied to the whole batch.
- (`is-type?` refuses a computed keyword at check time; `type-of` accepts one. Probe
  `probe-Q/computed.wat`.)

Each file converted alone, as `convert.sh` does it:

| file | today | batched | output |
|---|---|---|---|
| `tests/services/probe_arc278_sift_rules.wat` | 56.1 s | **1.7 s** | byte-identical |
| `tests/types/enums_tagged_variant.wat` | 17.1 s | **1.6 s** | byte-identical |
| `tests/comms/probe_arc293_W2f_process_dials_thread.wat` (poisoned; fallback exercised) | 33.9 s | **1.8 s** | byte-identical |
| `tests/macros/probe_arc265_acronym_registry_svc.wat` | 24.0 s | **1.9 s** | byte-identical |

UNRESOLVED lines are identical too. The slow unbatched ladder run (`probe-L/pc2.sh`) was stopped:
batched variants answer its question in minutes.

**⚠ At FULL scale, v1 was NOT identical** (`probe-Q/run2.sh`, sharded 8 ways):
- positional-ctor differed from today's output in **28 files** (UNRESOLVED 7,663 → 7,814), and the
  ladder built on it lost constructors in the same 27;
- match-arm's batched ladder WAS identical to the unbatched ladder (0 files; UNRESOLVED 15).

The four-file smoke set simply contained no offender.
- **Cause, from the batch's own outcome:** `:wat::runtime::type-of: unknown type ':wat::core::Vector'`.
  One element that raises fails the WHOLE batch, and v1 then stepped every path down the chain.
- **A MAIN DEFECT underneath** (`bootstrap/era/probe-H/istype-vs-typeof.wat`, main's binary):
  `(is-type? :wat::core::Vector)` → `true`, but `(type-of :wat::core::Vector)` → `malformed … unknown type`.
  The two reflection verbs disagree, while `type-of`'s doc claims it answers for every type. There is
  no non-raising "is this an enum" verb (the `:wat::runtime::` list has none).
- **The fixes, both probed:**
  - v2 (`fill-batch2.wat`) falls back to the ORIGINAL per-path `try-type-of` on any failed batch.
    It is exact (sqlite_interop byte-identical), but slow wherever `Vector` appears (13.7 s).
  - v3 (`fill-batch3.wat`, `ma-fmap3.wat`) tells a DECLARATION-level failure (the declarations do not
    freeze on their own; the whole batch steps down one rung, as every path would) from an
    ELEMENT-level one (bisect; a lone offender gets the original `try-type-of`). It is exact by
    construction, at ~2·log₂n evals per offender.

### ✓ v3 (exact batching + bisection) at FULL scale — `bootstrap/era/probe-Q/run4.sh`, 2026-09-13 06:29–06:42Z, sharded 8 ways

| stage | wall | result |
|---|---|---|
| R1 positional-ctor, batched (`pc-batch3.wat`) | 249 s | **0 files differ** from today's output; UNRESOLVED 7,663 = 7,663 |
| R2 match-arm, batched ladder (`ma-batch-ladder3.wat`) | 250 s | **0 files differ** from the unbatched ladder; UNRESOLVED 15 |
| R3 positional-ctor, batched ladder (`pc-batch-ladder3.wat`) | 266 s | **0 files lose** a constructor vs today; 3 gain |
| R4 the chain under the batched ladder, vs main | — | identical **1,314** / differ **175** (v2: 1,260 / 229) |
| main-binary classifier over the 175 | — | CHAIN-FAILS **84** (v2: 127) · BOTH-PASS 68 · BOTH-FAIL-SAME 18 · BOTH-FAIL-DIFF 4 · MAIN-FAILS-ONLY 1 |

- ⚠ `run4.log`'s "evals-that-failed (BATCH-FAIL lines): 0" CANNOT SEE: v3 prints no BATCH-FAIL line.
  It is not a count of failed batches.
- The exact wat-side path is the FALLBACK. The primitive is the fix (finding 12, BRIEF-2a1).

## Finding 12 — the wat-side workaround CAPS; the missing primitive is "expand + register + query"

- **Measured in the replay's own mode** (`convert.sh`-shaped: five files, ONE invocation):
  - today's positional-ctor: 99 s;
  - exact batched v3 (`probe-Q/pc-batch3.wat`): 42 s, all five byte-identical.

  That is 2.4×, NOT the 10–33× of v1, which was inexact. Every batch still pays `eval-with-defs!`'s
  two full startups, and `is-type?`/`type-of`'s disagreement on builtins like `:wat::core::Vector`
  forces bisection.
- **The operation the codemods need is ordinary substrate work:** expand a program's declarations with
  the stdlib macros, register their types into a COPY of the stdlib registry, and query it. The Rust
  side does exactly that in a few lines: the test helper `stdlib_loaded()` + `check()` at
  `src/check.rs:23436` (`#[cfg(test)]`).
- **What wat cannot do:**
  - the reflection verbs (`type-of`, `is-type?`, `field-names-of`) query only the RUNNING world's
    registry;
  - the only wat-reachable way to expand and register a program's declarations is `eval-with-defs!`,
    a REPL turn (two full startups, stdlib rebuilt, every body checked).
- So a verb that exposes "expand + register into a scratch copy + return the types" is not a
  convenience. **It is the operation itself, and the workarounds around its absence are measured to
  cap at a small multiple.** The design pins already known:
  - a stdlib snapshot taken at STARTUP, never lazily from inside evaluation
    (`src/runtime.rs:10294` records that deadlock);
  - no body check, so no poisoning and no ladder;
  - a fresh copy per call, so no cross-file leak;
  - a structured failure naming the declaration that broke.

## Finding 13 — the door at full scale: excluding a form drops everything that form declares

Measured 2026-09-13 on `101626eea` (2a1 + its first refute), over the 1489-file step-24 input.
- **One wat process, 13.2 s** (~9 ms a file including read and parse): 1457 Ok, 32 Refused, 0 unreadable.
  The 32: 8 stdlib files colliding with the snapshot, 10 era-syntax defmacros, 8 era or negative-fixture
  declarations, 6 negative `defservice` fixtures. None is a user-macro call.
- **Silent drops under the head filter:**
  - `:wat::holon::defrecord` (`wat/Record.wat:224`, emits `recordtype`): 22 of 22 records missing; the
    control, `:wat::core::defrecord` in the same files, 17 of 17 found;
  - what `defn` declares: `::Kwargs` (`wat/core.wat:888`), `::GrantHandles`, `::Coords` (`:1188`, `:1195`);
  - declarations in a top-level `let` body.
- **A static derivation over-admits** (keyword walk, then emitted-head walk): both derive `defn`, and
  through it `deftest`, `deftest-hermetic`, `run-hermetic`, `rete::defrule`, `rete::defquery`.
- **The one-step walk is exact**: `expand_once` from the top, containers by `container_body_start`, keep
  what `classify_type_decl` accepts. 0 files lose a type, 47 gain, 0 new refusals; 18 s for both doors.
  Keeping `derive`/`extend-type`/`declare-acronyms` changes nothing (identical report), so no name list.
- **The committed D1 test read ignored `bootstrap/` files** (`175b49ea9`): green here, red on a clone.
- Instruments: `bootstrap/era/probe-R/` — `corpus.sh`, `door.wat` (records + refused form),
  `r3-macro.wat`, `probes-2a1-refute2.diff` (the three Rust probes), `one-step-vs-door*.txt`.
- Routed: `BRIEF-2a1-REFUTE-2.md` (R5 the walk, R6 tracked ground + a lint, R7 the oracle).
- **Closed** by grok's `ca0f5c2d7` plus one orchestrator line. Full-scale identity with the
  orchestrator's walk (`verify-refute2.sh`: 0 lost, 0 gained, 0 refusal delta). Mutation M1 (keep
  `defn` whole) turns `declared_types_body_is_never_expanded` RED, and the tracked sift timing test
  RED with the original `mem-store/start` refusal. Mutation M2a, a `format!("{}/bootstrap/…")` path,
  PASSED grok's lint, which matched only literals beginning `"bootstrap/` — the brief's own narrow
  spec. The predicate is now any non-`//` line naming `bootstrap/` (zero hits today outside the lint).

## Finding 14 — after 2a2 the codemods need a NON-RAISING stdlib type query; only `eval-with-defs!` catches the raise today

Measured 2026-09-13 on `5edca1211` (probes: `bootstrap/era/probe-R/{defect-*.wat,vpo.wat,vpo2.wat}`).
- **The door answers only what a program ADDS.** Stdlib answers come from each codemod's `try-type-of`
  (`match-arm-to-bracket-map-pattern.wat:162`, `positional-ctor-to-map.wat:166`): `eval-with-defs!`,
  whose `FormOutcome` is the only catch in wat. No try/catch intrinsic exists.
- **`type-of` raises on every name `is-type?` admits without a `TypeDef`**: 35 of `is_builtin_primitive`'s
  37 names (all but `Option` and `Result`, which are `defenum`s), plus derive-marker parents
  (`src/types.rs:631`, `:644`, `:956`; `src/runtime.rs:9995`). `:wat::core::Vector`: `is-type?` true,
  `type-of` raises `unknown type`, and the process still exits 0. `(type-of :wat::core::i64)` as a
  literal is refused at check (Doctrine 1).
- **`variant-parent-of` never raises, but answers only the `.` spelling.** `Option::Some`, `Result::Ok`,
  `RecvOutcome::Message` (literal or `from-string`) → `None`; the `.` forms → their enum. Steps 23 and
  26 run before the dot flip (27), so they see `::`. Its own `@example` (`src/reflect/verbs.rs:1718`)
  claims `Option::Some` → `:wat::core::Option`, and nothing checks it:
  `verify_examples_reports_no_failures` is `#[ignore]` (the doctest runner raises before collecting).
- **Blast radius of making `type-of` total** (a kind for a known name with no declared structure):
  `wat/runtime-typeinfo.wat`, `src/reflect/verbs.rs` (`type-of`'s `None` arm, `type_kind_value`,
  `type_body_value`), one exhaustive match (`tests/reflection/probe_arc296_type_of_six_kinds.wat:32-37`).
  Both codemods fall through with `_`.
- Routed: the 2a2 stdlib route is the builder's ruling (four questions in the main chat, 2026-09-13).
- **RULED C; closed by 2a1b** (grok `deaeeb131`, verified by the orchestrator): one `TypeEnv`
  classifier (Declared / Builtin / Marker / Unknown) that `is_known_type`, `type-of` and `subtype?` ask.
  `bootstrap/era/probe-R/kinds.wat`: `Vector`, a literal `i64`, a computed `HashMap` → Builtin; a derive
  marker → Marker with its children; `subtype?` on it → true; an unknown name raises as before. The
  agreement wall (`type_of_answers_every_is_type_name`, enumerated from the stores) went RED under the
  orchestrator's own mutation of the Marker arm (grok's was the Builtin arm). The door is unchanged
  (corpus 1457/32; one-step identity 0/0/0). The two builtin stores differ 14 one way and 12 the other
  (SCORE-2a1b's table); routed.
- **The exclusion loop for 2a2** (`bootstrap/era/probe-R/door-exclude.wat`, the 32 refused step-24
  files): all reach `Ok` after dropping 39 refused forms in 981 ms; 6 negative defservice fixtures
  recover 15–48 types each; stdlib files exclude only the declarations that differ from HEAD.

## Finding 8 — a NESTED program is never checked, so its defects are invisible on main

- A child program inside `(:wat::core::forms …)` (spawned by `spawn-peer`, `spawn-program`, …) is
  checked only when it STARTS. `--check` and `every_wat_scripts_file_loads` check the parent.
- Two instances, both on main's binary (`bootstrap/era/probe-H/`):
  - `wat-scripts/probes/arc-170/probe-m1-ann-erase.wat`: parent `--check` rc 0. Run: the child dies
    at startup, `variant arm head :probe::CMsg::Setup is not namespaced` (`CMsg` is declared only in
    the child, so it is off finding 7's census). The run still exits 0 and prints the failure as a
    value. Its `EXPECT (green): "echo:z"` is false.
  - `…/probe-m1-ann-erase2.wat` (finding 5): the child's `serve` signature still names
    `:probe::CMsg` (line 47). erase2 is erase with `PMsg`/`CMsg` → `PoolMsg`, and that one site was
    missed. Its two arms carry the leaked `{:deps}`/`{:pair}`; the parent's sends use the right
    `{:addr}`/`{:s}`.
- This is what hid finding 5, and it hides finding 7 in every child program. The fix is on the
  extirpare ladder: a gate that checks every nested program literal, not a patch to two probes.
- Size (a TEXT count of `(:wat::core::forms`, indicative only; the real census `--check`s each child):
  164 occurrences in 105 of main's `.wat` files (18 `wat-scripts/probes`, 11 `tests/services`, 9
  `tests/comms`, …). grok-rete modifies or adds 13 of them.
  - ⚠ Superseded by the substrate census below: 141 literals, not 164. The text count also counted
    mentions inside comments and strings.

### The census, asked of the substrate (`bootstrap/era/probe-N/`, HEAD `4ff274b0a`)

- **The instrument.** `extract-forms.wat` reads each file with `read-string`, collects every
  `(:wat::core::forms …)` node at any depth, and writes that literal's children, cut by source span,
  as a standalone program for `wat --check`.
- **It discriminates.** On `probe-m1-ann-erase{,2}.wat` it fails exactly the pre-fix children
  (`f2e0ac26b^`: `:probe::CMsg::Setup is not namespaced`, `unknown type :probe::CMsg`) and passes
  grok's fixed ones.
- **Over all 1932 tracked `.wat` outside `wat-scripts/fixes/`: 141 literals** (none unreadable).
  110 pass `--check`: 107 look like programs, 3 like templates. 31 fail: 29 look like programs, 2
  like templates.
- **Most of the 29 are NOT defects.** One site of each kind was READ:

  | kind | read | verdict |
  |---|---|---|
  | UselessMain (4) | `tests/kernel/wat_run_sandboxed.wat:34`, a `main` that is `nil` | **the instrument is stricter than production.** `validate_user_main_not_useless` has one caller, `startup_from_source` (`src/freeze.rs:947`). The child path, `startup_from_forms_with_inherit` (`src/process/verbs.rs:431`), never runs it, and `tests/kernel/wat_run_sandboxed.rs:67` asserts this child closes `"closed"` |
  | a literal used as DATA | `tests/macros/probe_resolver_quote_awareness_forms_data.wat:3`, inside a `do`, never spawned | not a program |
  | a deliberate negative child (6, `wat-tests/core/core-{arithmetic,equality}.wat`) | `core-arithmetic.wat:265`, whose test expects the failure | pinned by its own test |
  | a child that relies on its parent | `probe-child-inherits-defns.wat:17` | the failure is the probe's answer |
  | "empty input" (2) | `wat/service.wat:2490`, `` `(:wat::core::forms ~child-main-form)`` | a TEMPLATE the detector missed: an unquote's span covers only the `~` |

  The rest (≈14 in `wat-scripts/probes/arc-170/`, plus `tests/process/arc112_scheme_probe.wat`,
  `tests/wat_lang/wat_core_forms.wat`, `probe-s1-named.wat`) are NOT yet classified. Their shape
  matches the two real defects `erase`/`erase2` were.

- **Where the literals sit** (`extract-forms2.wat`, which records each literal's enclosing head and
  whether it is inside a quasiquote; still 141): plain `:wat::test::spawn-peer` 121 (102 pass, 19 fail) ·
  `:wat::spawn::Locus/launch` 1 (pass) · `:wat::core::concat` 7 (fragments of an assembled program) ·
  `defn` 2 · `length` 1 · `do` 1 (data) · quasiquote templates 5.
- **The 19 `spawn-peer` children that fail, every one classified by reading:**

  | n | what | a defect? |
  |---|---|---|
  | 4 | UselessMain (`--check` only; production's child path never runs that wall) | no, an artifact of the instrument |
  | 6 | `wat-tests/core/core-{arithmetic,equality}.wat` `…-rejected` tests: each asserts its child FAILS | no, deliberate negatives |
  | 1 | `probe-child-inherits-defns.wat`: the child calls a parent defn by name | no, its failure is the answer |
  | **7** | `wat-scripts/probes/arc-170/probe-m1-{cf-norevoke,dial-runner,fix-norevoke,fix-revoke,grant-admits,worker-setup}.wat`, `probe-bracket-process-runner.wat` | **YES.** Each child predates the RecvOutcome/SendOutcome walls: `recv` and `Echo/echo` now return `(RecvOutcome :- [T])` where the child expects `T`, and `SendOutcome` gained `Stopped`, which its matches lack. These are the same two walls grok met in `erase{,2}` (SCORE-2b § D3). |
  | 1 | `tests/process/arc112_scheme_probe.wat:12` (1 unresolved reference) | not yet classified |

  **The blind spot hid at least 9 broken probes on main** (these 7, plus `erase{,2}` which 2b fixed). The
  floor was green over every one of them.

- **What a GATE must be, from this census:**
  - it keys on a literal in a PROGRAM-STARTING position (spawned, sandboxed, …), not on every `forms`;
  - it runs each child on the CHILD's real path, `startup_from_forms_with_inherit` with an
    `InMemoryLoader` (`src/process/verbs.rs:429-433`), never `--check`, which is stricter;
  - a test that expects a child's startup to fail pins that expectation itself, and the gate honours it;
  - a template is not a program until it is expanded.

## `convert.sh` does not run `one-param-spec` the way this check did

`scripts/replay/convert.sh` gives `one-param-spec` the converted file as its WHOLE context
(`printf '["%s"]\n["%s"]\n' "$WORK" "$WORK"`). `run.sh` gave it every file in scope. The tool learns
user-type arity from its context by design ("a type declared in file A resolves for a call site in
file B", its header), so at replay time a grok file using a parametric type declared in ANOTHER file
is left unconverted. Main's param-spec wall then refuses it (STOP-2, loud). The composition result
therefore does not describe `convert.sh` for this step.

Measured (`bootstrap/era/probe-M/`, grok-rete tip `37528f6e0`, 1641 `.wat` outside `wat-scripts/fixes/`):
- the raw corpus as context fails in 0.3 s, `lex error … angle-bracket type parameters are illegal`.
  The file is `docs/arc/2026/05/130-cache-services-pair-by-index/complected-2026-05-02/substrate.wat`
  (byte 2629), and its sibling `test.wat` fails the same way (byte 2263). Main DELETED both in
  `c5f1ee487` WALL(278) ("every tracked *.wat must READ"); grok-rete modifies both.
- without those two (1639 files): rc 0 in **2.6 s**, against 0.28 s with the single file. A corpus
  context is affordable on every `convert.sh` call, provided unreadable files are excluded and each
  one is REPORTED, never silently dropped.

## grok-rete `.wat` files OUTSIDE the 1489 checked here (11 of its 194 modified)

- 3 that main DELETED (`c5f1ee487` WALL(278); `a3f24a5c3`): the two `130-…/complected-2026-05-02/{substrate,test}.wat`,
  and `wat-scripts/scratch-pad/probe-f64-comparator-bogus-head.wat`. grok-rete modifies them, so the
  replay meets modify/delete. Main owns the deletion unless grok's change carries rete behaviour.
- 8 that are grok-rete's own CODEMODS (`wat-scripts/fixes/rete-*`, `to-faithful-clojure-{net,rete}`,
  `type-query-to-defquery`), excluded because a tool is never its own input. The replay needs a stated
  way to bring a codemod's own source to main's syntax without the chain rewriting the string
  literals its rules match on.
- Plus grok-rete's 161 NEW `.wat` files: they pass through the chain with no main counterpart, so this
  check cannot see them.

## Operational notes (all paid for today)

- Long runs are launched `setsid nohup`: the harness's memory guard kills tracked background tasks.
  Twice today it fired on a transient spike, and the only kernel OOM victim was an unrelated 29 GB
  `python3`.
- `pgrep -f '<script path>'` matches your OWN shell's text. Match the runner's argv prefix.
  - Twice more on 2026-09-13:
    - a guard `! pgrep -f 'positional-ctor-to-map.wat'` matched the Bash call carrying it;
    - a wait loop matched a leftover harness `bash -c` whose command line held the whole launching
      heredoc.
  - **Do not decide on `pgrep -f` at all.** Wait on a pid you hold (`$!`, `wait`), or on a DONE line
    the job itself writes.
- A codemod piped EMPTY stdin dies with `"disconnected"`. That is never-ran, not "0 unresolved".
