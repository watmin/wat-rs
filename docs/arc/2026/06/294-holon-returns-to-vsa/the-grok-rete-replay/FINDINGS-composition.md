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

## Finding 15 — 2a2 at full scale: minutes, not hours; one nested-program loss my brief caused

Measured 2026-09-13 on `9d887f753` (`bootstrap/era/probe-S/run5.sh`, each tool against its predecessor
on identical input; 1489 files; each tool ONE process):
- **Wall:** match-arm 146 s, positional-ctor 28 s, variant-separator 50 s. (Before: the eval-based tools
  took hours unsharded; exact batching needed 8 shards and ~13 min.) `convert.sh`, one run per commit:
  pilot #1 10 s, #9's two files 13–14 s, byte-identical across runs (was 29–51 s PER FILE).
- **Coverage:** positional-ctor UNRESOLVED 7663 → 209 (the case rule is gone); variant-separator flips
  335 more tokens in 62 files (enums never on the census — finding 7's prediction); the chain vs main
  1314 → **1367** identical, CHAIN-FAILS 84 → **28**; 54 files fixed.
- **The one loss:** `probe-m1-ann-erase.wat`'s child program declares `:probe::CMsg`; the door answers
  top-level forms only, so its arms stay positional where the ladder and main converted them. My brief
  cut nested programs to "report" by reasoning from today's `file-decls` instead of from the bar's
  predecessor. With finding 8 (no gate checks a child), that cut would ship unconverted children silently.
- Routed: `BRIEF-2a2-REFUTE.md` (R8 nested programs scoped per program; R9–R13 the filter, the guard,
  the fallback's report, the stale header, the `""` sentinel).
- **Closed** by grok's `f36c2386c`, verified by the orchestrator: floor 5431/5431, clippy 0; run5 —
  match-arm LOSING **0** (UNRESOLVED 10), positional-ctor's one "losing" file the same deeper-conversion
  artifact, variant-separator 0 lost / 67 gained; the chain vs main 1370 identical, CHAIN-FAILS 28. The
  one file newly differing from main vs run4 (`probe-m1-ann-erase.wat`) differs only by its child's
  `CMsg::Setup` → `CMsg.Setup`, which main's census never flipped (main's own `::` there is finding 8's
  defect). `probe-m1-ann-erase2.wat`'s child fields match the file's own declarations; main's
  `{:deps}`/`{:pair}` is finding 5's cross-FILE leak (the SCORE called it the parent's). R10: the
  orchestrator's own mutation (a stdlib `defmacro` calling `declared-types`) turned the guard RED.
- **Where `convert.sh`'s time goes** (bash trace, one call: #9's two files 15.9 s, #1's one file 13.9 s):
  whole-tree extract 2.4–3.3 s, the ~1900-file parse filter ~0.6 s, and the codemod loop the rest
  (~11.5–12.6 s): `convert.sh`'s loop runs `"$WAT" "$cm"` once per in-scope codemod — 27 `wat`
  processes, each paying a startup (≈0.45 s each on average; the loop's own xtrace lines land in its
  `$LOG`, so only the loop's total is measured). Twice per commit (the C^ set, the C set).

## Finding 16 — main's `:wat::*` blanket kill surfaces LATENT branch defects in the replay

Batch 1 stopped at its first step, #11 (`eebf75374`): the converted `rete-differential.wat` calls
`:wat::core::edn::to-string`, and main refuses it at startup (`UnresolvedReferences`).
- The name **never existed** on either side (whole-tree search at `de827fb4c` and `eebf75374`: only the
  call site itself). It resolved on grok-rete because the resolver returned true for any reserved-prefix
  head; main's `c3fefc5ab` (2026-09-09) made resolution registry-only. The call sits on the fuzzer's
  MISMATCH branch, which never ran — the branch's own test passed there.
- So this is neither a chain miss (no migration renamed anything) nor a rete change: it is a defect the
  branch carried silently, which a wall main added after the split makes visible. It will recur wherever
  grok-rete calls an unregistered `:wat::*` verb.
- Grok-rete's author fixed this one at #19 `a39c28e10`, with text that uses bindings #11 lacks.
- Routed: `BRIEF-4-ADDENDUM-latent-defects.md` (RULED 4 YES) — re-express the one call head with main's
  registered verb of the same meaning (`:wat::edn::write`), log it `LATENT (c3fefc5ab)`, let the author's
  later fix win its conflict; STOP-7 when no registered verb has the meaning.

## Finding 17 — the door reads every file as USER code, so a replayed step that changes the stdlib fails

Batch 1 stopped at #22 `8eeff8adc`: grok-rete promotes `wat-scripts/lib/gen.wat` into the stdlib as
`wat/gen.wat` (`:wat::gen::*`). #11–#21 replayed clean (floor 5436/5436 on `1294f2fc3`, pushed).
- The door runs `build_env`'s USER half (`src/freeze/env.rs:134`), so every `:wat::gen::` declaration is
  refused `ReservedPrefix`; match-arm and positional-ctor leave the file unconverted; baked through
  `include_str!`, it stops the binary starting (`ReturnTypeMismatch` `wat/gen.wat:143`); its consumers ask
  `type-of` in HEAD's world, which lacks it.
- For a stdlib file a commit CHANGES, the door refuses the changed type as a duplicate and the codemod
  answers HEAD's PREVIOUS version — a changed field would convert stale, silently.
- **The class:** 34 grok-rete commits from #22 touch a stdlib `wat/*.wat` file (2 add one, 25 also change
  consumers); 19 of batch 1's remaining 39 (`bootstrap/era/replay-plan/stdlib-touch.tsv`).
- **The probe** (three runs): `build_env`'s STDLIB half on the file's forms registers all of it —
  `register_stdlib_defmacros`, `expand_all_with(…, Privilege::Stdlib)` (a `defstruct`'s companion macro is
  minted during expansion), `register_stdlib_types`: `CheckOutcome` = `Checked [points violations]`,
  `EmptySpace []`. The user door on the same text: `ReservedPrefix`.
- Also verified in #11–#21: the 3 `#[ignore]`s added are grok-rete's own TDD red tests (defect families A/B
  at #19, C at #20), closed by grok-rete at #46 (B) and #49 (A and C) — inside batch 1; after #60 they must
  be gone.
- Routed: `BRIEF-2a4-the-door-reads-a-stdlib-file-as-stdlib.md`.

## Finding 18 — at the merge, a wall from one side meets the other side's content; the per-step gate never looked

Batch 1 replayed #11–#60 (2a4 `c184348f7`; checkpoint #35 green 5463/5463) and the #60 checkpoint went red,
5 tests (`.floor/2026-09-14T01-43-05Z`). Verified by the orchestrator: 50 REPLAY commits, every `-x`
trailer matching `commits.tsv`; E2/E8 pass; E3 holds (of the 18 `.wat` the batch touched, only
`wat/{fix,gen}.wat` fail `--check`, the stdlib-as-user instrument limit); the TDD close — defect families
A/B/C's three tests are live, un-ignored, as on grok-rete at its #60.
- **Arms 2–5:** #50's `UnconsumedWrapperBind` (grok-rete's rete wall: a bind fresh inside a `:not` must be
  consumed there) refuses 24 dead binds in main-only `wat-scripts/fmt/rules/{kwargs,defrecord}.wat`
  (created on main after the fork, never on grok-rete). The whole-tree census: 19 files fail on it, all of
  them the two rule files or their loaders.
- **Arm 1:** main's `one_variant_separator` lint (`7ccce48ba`, never on grok-rete) flags #60's
  `config.rs:415` `leaf(head)` — a namespace test. (SCORE-4 says #59; both sides add it at #60.)
- **The probe** (`bootstrap/era/probe-T/probe-60-fix.sh`): dropping the 24 binds by the wall's own spans,
  guarded, plus the lint's `namespace` rune → the 19 files `--check` clean and the 5 tests pass, the fmt byte
  golden unchanged. (Its first run proved nothing: a wrong site-file path, so no drop ran; caught from the
  output, re-run with a guard that aborts unless exactly 24 sites and 24 changed lines.)
- **Why the step gate missed it:** it checks what a step PRODUCED; a wall meets content the step never
  touched. A whole-tree `--check` census takes 30.6 s at `-P32` (2047 files, 219 failing at baseline), and
  200 of the remaining 591 steps change the binary or a `.wat`.
- **2a4's two flaws** (no replayed output affected): a replaced enum keeps its old variant singletons in the
  door's copy (`retract_for_door_replace` removes only the parent; `register_variant_types` skips an
  existing singleton); `stdlib-source-path?`'s `contains "/wat/"` classifies 5 non-stdlib tracked files as
  stdlib, while tracked `wat/**/*.wat` equal the 63 `STDLIB_FILES` paths exactly.
- Routed: `BRIEF-4b-checkpoint-60-the-walls-meet.md` (fold the repairs into #50 and #60; the recorded
  migration; 2a4b; the per-step census).

## Finding 19 — absolute paths blinded two path rules; the stdlib door refuses a divergent MACRO

Verifying 4b (fold proof reproduced; floor 5495/5495 on `b6ffccbda`, pushed):
- **2a4b's gate read a proxy.** `tracked_wat_dir_is_exactly_stdlib_sources` scanned `src/load/stdlib.rs`'s
  text; with `wat/gen.wat`'s row commented out (not baked) it PASSED. Rewritten to ask the runtime
  (`(:wat::stdlib::sources)`): FAIL `only-tracked: ["wat/gen.wat"]`, PASS on revert (`33f3ebcfb`).
- **The path contract.** The codemods carry exactly two path rules — 2a4b's `stdlib-source-path?` and
  positional-ctor's `skip-path?` (a hand list: `wat/service.wat` and three more, "RESIDUE 1" of arc 296
  M2, templates it cannot convert). Both assume REPO-RELATIVE paths; `convert.sh` (until 2a4b) and
  `run5.sh` (until the orchestrator's fix) passed absolute ones, so the skip list never matched and,
  after 2a4b, run5 sent every stdlib file through the USER door — the bar stopped measuring the stdlib
  mode (VS report lines 35 → 38 with no losing file). run5 now hands repo-relative paths: PC's second
  "losing" file is `wat/service.wat`, which it now skips as its list says. Batch 1 touched no skip-listed
  path; **#379 is the one later step that does** — a batch boundary.
- **G1:** the stdlib door replaces a divergent TYPE (probed: `defrecord Node` + `depth`, top level and in
  a `do`) but REFUSES a divergent MACRO (`DuplicateMacro`, registry gate `src/macros/registry.rs:71-93`).
  Batch 1: #31 #38 #40 #43 changed `:wat::gen::record`, which mints no type — no answer changed. Later: 6
  steps change 4 stdlib macros (#155 #157 #159 #226 #377 #379).
- **The stdlib half does not walk**: it expands every `defn` body, so 2 `defn`s refuse
  (`ProgramBodyEvalFailed`) though they declare nothing.
- Routed: `BRIEF-2a4c-the-stdlib-door-replaces-a-divergent-macro.md`.

## Finding 20 — batch 2: two of main's in-crate walls meet grok-rete code; the step gate could not see them

Batch 2 replayed #61–#125 (SCORE-5). Verified by the orchestrator: 65 steps, contiguous, each with its
`-x` trailer (#63's hand-written, a short hash); the 30 docs-only steps touch only docs; no `wat/`,
`wat-scripts/fixes/` or main-deleted path; no conflict marker committed; floor 5540/5540 and clippy 0 at
the tip `07489fc1e` (the orchestrator's own run).
- **#95** adds `:wat::rete::keyword::{from-string,to-string}` as `RETE_OPS` Alias rows with a `TypeScheme`
  and no registry row. Main's arc-255 registry ratchet (`registry_membership_gap_a_is_named_and_frozen`)
  and `every_row_is_admitted` (`RETE_MODULES`) are red from #95 on.
- **#108** adds a four-space-indented list under `///` in `Ret`'s doc; main's floor runs
  `cargo test --doc` ("the walls must not be muted"), grok-rete's did not, so rustdoc compiles it.
- Both repairs are RIGHT — Gap A's own message sanctions adding a NEW name, and its header names `RETE_OPS`
  Alias rows as its population; a ```` ```text ```` fence for prose — but both were committed AFTER #125
  (`00cc59ff4`, `1c6d6af3e`), leaving #95–#125 knowingly red. The 4b ruling folds them into #95 and #108.
- **Why the step gate missed both:** the lint binary, the census and stone 3's gate do not run the
  library's own unit tests (1471 tests, 16 s warm) or the doctests (2 s warm).
- #95's "LATENT" label is the ordinary `.rs` re-expression onto main's moved home (`:wat::keyword::*`),
  not the addendum's never-existed class.
- Routed: `BRIEF-5b-fold-the-two-late-repairs.md`.

## Finding 21 — the deleted-file census was a subset: it missed every file main MOVED

Preparing batch 3, `#126`'s biggest edit (`src/edn_shim.rs`, +147/−74) turned out to be to a file main
deleted — `8ddccaaa3` ("EDN gets a home — five loose root files become src/edn/") — which
`bootstrap/era/replay-plan/main-deleted.txt` did not list. That census counted a moved-and-reshaped file
as a RENAME, so it could see only files main removed outright.
- **The honest census asks no rename heuristic:** a file a step edits (status `M`) that exists at the fork
  and not at main's tip — `bootstrap/era/replay-plan/absent-on-main.tsv`: 13 rows, 11 steps, 6 files. The
  old census missed 4 of the 6.
- **Two classes.** MOVED home — `src/stdlib.rs` (#22 #28 #30 #31 #36 #38 #41, all re-expressed on
  `src/load/stdlib.rs` by the standing `.rs` rule; and #438), `src/string_ops.rs` and `src/edn_shim.rs`
  (#126 → `src/string/mod.rs`, `src/edn/render.rs`): the standing rule, NOT a boundary. DELETED for cause
  — #212's two `docs/arc/2026/05/130-…/complected-2026-05-02/*.wat` (main `c5f1ee487`, "every tracked
  *.wat must READ") and #278's f64 bogus-head scratch probe (main `a3f24a5c3`, "blanket-dependents are
  ZERO"); grok only annotates each (11, 10, 4 lines): a policy is owed at #212.
- The boundary list was wrong in both directions: #126 is not a boundary; #212 is.

> ⚠ **CORRECTED 2026-09-15 — the "DELETED for cause" class above does not exist, and #212 is NOT a
> boundary.** `git show -M --name-status` on each deleting commit: `src/stdlib.rs` R092 →
> `src/load/stdlib.rs`; `src/edn_shim.rs` R099 → `src/edn/render.rs`; `src/string_ops.rs` D, its 29 verbs
> split across five homes; #212's two `130-…/complected-2026-05-02/{substrate,test}.wat` **R100 →
> `*.wat.bad`** (both sides PRESERVE them as the complectēns calibration set — main by the house's
> `.wat.bad`, grok by a closed `rune:lint(historical)` foot for its new docs-`.wat` gate, which scans
> `.wat` only); #278's f64 bogus-head probe R055 → `tests/resolve/probe_arc255_the_blanket_hides_a_phantom_head__bogus_rete_head.wat`.
> My census used `--no-renames`, which reports a rename as a deletion — the same blind spot this finding
> names, one level down. **Both are the standing moved-home rule:** #212 — the `.bad` suffix IS the
> declaration grok's rune makes and its gate cannot see a `.bad`, so drop the rune, merge grok's README
> prose, log it; #278 — the 4 lines served a `wat-scripts/` lint the moved fixture lies outside of, so drop
> and log (its edits to three of main's tools are Q1). `absent-on-main.tsv` now carries main's fate per file.

## Finding 22 — batch 3's per-step record: the census everywhere, the other walls at 3 of 11 steps

Batch 3 replayed #126–#152 (SCORE-6). Verified by the orchestrator: 27 steps, contiguous, each with its
trailer; the 14 docs-only steps touch only docs; no hazard path; no conflict marker; no repair commit after
#152 — the #139 checkpoint's red (a golden pinning `freeze.rs` `:line 1525`, moved to `1526` by #126's
comments) was FOLDED into #126 by grok itself, the diff exactly that line; #126 lands on main's moved
homes (`src/edn/render.rs`, `src/string/mod.rs`); floor 5546/5546 and clippy 0 at the tip `74f52aed1`.
- **SCORE-6's E7 was not backed by the record.** Commit bodies and REPLAY-LOG rows carry the census at
  every qualifying step, but stone 3's gate only at #126 #131 #142 #151 and the lint subset / `kind(lib)` /
  doctests only at #126 #131 #151. Ran-but-unrecorded cannot be told from never-ran, and the per-step
  walls are what prove no intermediate commit is knowingly red — the checkpoint floors prove #139 and #152.
- **Re-measured by the orchestrator** at the 8 unproven commits (#128 #134 #137 #140 #142 #143 #144 #149;
  detached, each built; the lint binary minus the whole-tree load test, `kind(lib)`, the doctests): **all 8
  CLEAN** — 1614–1616 passed, stone 3's gate PASS, doctests 8/0. The record was incomplete, not the work.
- **The class, one rung up:** `scripts/replay/verify-step-record.sh <from> [<to>]` derives each REPLAY
  commit's required walls from its OWN diff and checks its body carries each verdict line; E7 becomes a
  command. Its first run on batch 3 exits 1 and flags 12 of 27 steps — the 8 above, #139 (its checkpoint
  floor covers it), and #126 / #151, whose walls ran but are worded freely. A record's FORMAT is pinned, not
  learned: from #153 the recipe prescribes the five lines verbatim, and the first conforming step is the
  gate's passing case.

## Finding 23 — batch 4a: green, and two METHOD flaws — the stdlib door reads one file at a time; an 11-file hand edit to the chain

Batch 4a replayed #153–#159, the first steps under the codemod-source policy (P1 + Q1). Verified by the
orchestrator: 7 steps contiguous with trailers; `scripts/replay/verify-step-record.sh 18eb21a71 HEAD` →
`step-record: complete` (the record gate's first PASSING case, after it failed on batch 3); no repair commit
after #159; `convert.sh` refuses a chain member (derived from `chain-order.sh`) and converts every other
`wat-scripts/fixes/*.wat`; five ported tools with fixtures; the Q1 tools' fixtures replay; floor 5551/5551
and clippy 0 at `868377378`; run5 unchanged on every axis (MA 0 losing · PC 2 · VS 0; chain vs main
identical **1370**, CHAIN-FAILS 28, VS report lines 22) — the 11 hand-edited chain members convert exactly
as before.
- **The stdlib door reads one file at a time against HEAD's snapshot.** The three outcome enums are declared
  in `wat/rete.wat` (`FireOutcome :- [T]` :242, `InsertOutcome` :273, `CompileOutcome` :311) and used in
  `wat/fmt.wat`, `wat/grep.wat`, `wat/query.wat`, `wat/rete/oracle/{explain,fire}.wat` — changed in the same
  steps. Phase (a) answered each user file from HEAD's snapshot, match-arm left list-form arms UNRESOLVED,
  the merged stdlib could not load, and grok KEY-FIRST'd the leftovers by hand (SCORE-7a E9). The
  STASH-DANCE would not have helped: the previous binary lacks the new enum too. The same shape returns at
  #221 #377 #379 #398 #438 #440 (≥2 stdlib files in one step); batch 4b touches no stdlib file.
- **#155 hand-wrapped `(overlay records)` in 11 CHAIN members** (and grep/fmt's overlay sites), 7 identical
  lines each, so the chain could load after `overlay` began returning `FireOutcome`. Necessary, and disclosed
  in #155's body — not in SCORE-7a. No existing tool could do it: the ported `wrap-fire-rules-in-fireoutcome`
  changes nothing on a chain member (probed; `overlay` is not a `fire-rules` call site). An 11-file
  structural rewrite by hand is exactly what R21 routes to a recorded migration.
- Routed: `BRIEF-2a4d-the-stdlib-door-reads-a-step-as-one-world.md`.

## Finding 24 — a new TEST FILE meets main's test-hygiene walls, and a targeted set that does not run them cannot see it

Stone 2a4d came back green on its executor's targeted checks — `cargo build --release`, `--check` on the
threaded codemods, and a 38-test gate set (`every_recorded_migration_replays`,
`every_wat_scripts_file_loads_on_the_current_runtime`, `every_tracked_wat_parses`, `one_variant_separator`,
`no_inlined_wat`, the new gate). **The orchestrator's floor was RED**, 5550/5552, on two walls that set did
not name, both firing on the stone's own NEW test file `tests/cli/stdlib_door_reads_a_set_as_one_world.rs`:
- `no_inlined_edn` at `:65` — `format!("[\"{DECLARER}\" \"{USER}\"]\n")`, a literal opening `[`. That wall
  refuses a rune here: *"A literal that merely LOOKS EDN-esque but is genuinely not EDN is NOT a rune
  candidate — restructure the CODE"*. The house pattern already existed —
  `every_recorded_migration_replays.rs`'s `default_path_vector` builds the vector from chars.
- `no_loose_string_assert` at `:85 :90 :94` — three `contains` sites. Its rubric allows a per-site
  `// rune:lint(loose-assert) — <reason>` for *"a targeted absence over a large output"*, which fits
  `:85`/`:90` (the report carries temp paths); `:94` was a POSITIVE substring check over a deterministic
  converted file, so it became an exact `assert_eq!` against a co-located golden captured from the tool's
  real output — measured, never guessed.
- **The class:** the executor ran the walls its stone's SUBJECT suggested (`no_inlined_wat`, because it had
  reasoned about inlining wat) and not the ones the FILE's existence triggers. A step that adds or changes a
  `.rs` test file must run the test-hygiene walls. (Kin: findings 18, 20, 22.)
- Repaired and FOLDED into the commit that introduced the file (the 4b rule; nothing was pushed):
  part 1 `9d26100e4` → `bd8295df1`, part 2 rebuilt `c13e891db` → `102d4c0fc`; subjects unchanged.
- ⚠ **The orchestrator's own error:** the first fold staged only the golden and amended, leaving the `.rs`
  half uncommitted — that commit would have failed both walls. The guard caught it (`git status` non-empty
  after the amend; the tip diff showing one path where two were expected). **A fold's proof must name WHICH
  paths it is expected to carry**, not merely that a commit was made.

## Finding 25 — the per-SET stdlib world unioned RAW sources, so one member's refusal stripped the world every sibling reads

2a4d made the stdlib door answer a step's files from ONE world (finding 23's fix). Its floor was green
(5552/5552, clippy 0) and run5's identity axes were unchanged — but the VS report counter moved **22 → 38**,
back to its pre-2a4c level, and the cause was a defect, not noise. Measured on `bootstrap/era/probe-L/pT`
(run5's real VS input), same binary:

| how the tool is handed the files | `wat/` refusals |
|---|---|
| the full 1489-path corpus (run5's shape) | 17 |
| the 54 stdlib members alone | 17 |
| one file at a time | **1** |

Set SIZE is irrelevant; UNIONING is the cause. `wat/core.wat` converts clean alone and refuses inside the
union (`defalias :wat::core::values` line 43, `defclause :wat::core::+` line 58).
- **The mechanism.** `stdlib-world` concatenated every member's RAW forms into one `declared-stdlib-types`
  call; `register-loop-set` dropped each refused form from that SHARED world and retried. In the per-file
  door a refusal cost that file its own form; in the union it leaves the world every member is answered
  from — one unexpandable body in `core.wat` can strip declarations a sibling's conversion needs. No output
  moved only because the dropped declarations happened not to be ones another file wanted: luck, and exactly
  the stale-answer class 2a4/2a4c exist to close.
- ⚠ **My first probe said "no difference" because it measured the wrong tree** — the pre-24 era corpus
  rather than `probe-L/pT`, which run5 actually converts (`cache.wat`, `bracket.wat` differ between them).
  Union and per-file agreed there (3 and 3) and I nearly wrote the delta off. **A probe must receive the
  input the stage receives** — 2a4's lesson, one instrument further out.
- **The refold** (folded into part 1, `bd8295df1` → `816cb99a0`; part 2 replayed `102d4c0fc` → `b6a8b0f38`;
  nothing was pushed until the orchestrator's own verification): register each member separately and union the
  KEPT `TypeInfo` rows, so a failure is isolated to its own file while every member still sees one world.
  Measured after: `wat/` refusals **17 → 1** (the survivor is the same `wat/service.wat … line=2443` the
  per-file run shows), the whole report **38 → 22**, `diff -rq` of before/after conversions **0 differences**,
  the gate still RED under the mutation that ignores the world, change confined to `wat/fix.wat` (53+/85−).
  Verified by the orchestrator: floor 5552/5552, clippy 0, run5 identity unchanged (chain vs main 1370), and
  `wrap-overlay-in-fireoutcome` reproducing #155 **11/11 byte-identical** and idempotent.
- ⚠ **My refold brief carried a bar row no artifact could satisfy** — "`--check` clean on `wat/fix.wat`".
  That file is BAKED STDLIB (`include_str!`), so checking it as a user program is the wrong door: 115
  pre-existing `ReservedPrefix` errors. The executor DISPROVED the row instead of working around it
  (`git show HEAD:wat/fix.wat` checked the same way gives rc=1, and its first error names a function the new
  version deletes — so that output came from the unmodified copy). Its real gates are the binary's own stdlib
  load and `every_wat_scripts_file_loads_on_the_current_runtime`. **An acceptance row nothing can satisfy
  teaches an executor to fake it**; this one was retired, not waived. Also recorded: `wat/fix.wat` is baked,
  so a `cargo build --release` must precede any probe re-run or the OLD world is measured.

## Finding 26 — the step-record gate selects by SUBJECT, so a step committed under another subject is not missing, it is invisible

Batch 4b's executor replayed #160 and #161 — both docs-only — with `git cherry-pick -x C` and nothing more,
so each landed carrying **grok's original subject** instead of `REPLAY(grok-rete #N): <C's subject>`.

- **The content was perfect.** `commits.tsv` says #160 `files=2`, #161 `files=1`; both landed with file sets
  IDENTICAL to grok's and kept the `(cherry picked from commit …)` trailer. Only the subject differed.
- **The brief does say it.** BRIEF-1 § "One step" item 1 governs how docs content is BROUGHT OVER
  (`git cherry-pick -x C`); item 4 governs the COMMIT and carries no docs-only exemption. Ground truth from
  batch 4a's #154 (`3f5f8defb`): subject `REPLAY(grok-rete #154): curare: …`, body `(cherry picked from
  commit 7f5915de9)`. To an executor meeting a docs-only step the two items read as one instruction, and the
  literal first line is the one it follows.
- ⛔ **Why this is worse than a red.** `verify-step-record.sh` selected its subjects with
  `git log --format='%H%x09%s' | grep 'REPLAY(grok-rete #'`. A step committed under another subject was
  therefore **not reported missing — it did not exist to the gate**. With 23 docs-only steps in #160–#211 it
  would have printed `step-record: complete` over a batch in which 23 steps were never examined, and E1
  ("52 REPLAY commits, contiguous") would have failed only at the very end of a 52-step run.
- **The class:** an instrument that ENUMERATES ITS OWN SUBJECTS can only report on the ones it enumerated,
  and its silence about the rest is indistinguishable from a pass. Kin: findings 22, 24; FM 29.
- **Repaired** on a quiescent tree at the executor's yield point: the two subjects rewritten and #162
  replayed on top (`84987d5f3`, `4cfa5a521`, `b3395b398`), proven by `git diff <old-tip> <new-tip>` = **0
  lines** — a subject rewrite may move no byte — with all three trailers preserved and the REPLAY count
  159 → 162. History surgery stayed with the orchestrator on a clean tree; the executor was told explicitly
  NOT to rewrite it under a dirty tree.
- **CURED, not merely conventionalised.** `verify-step-record.sh` now takes optional `<first-N> <last-N>`
  and asserts EXACTLY ONE `REPLAY(grok-rete #N)` per N in that range, cross-checking each commit's cited
  source SHA against `commits.tsv`, so a mis-subjected step goes **RED BY ABSENCE**. Note a contiguity check
  would NOT have caught this case: #160/#161 were the range's FIRST steps, so the surviving set 162…185 is
  perfectly contiguous. The expected range must come from outside the commits.
- **Caught by the freshness probe**, not by review: the SEAM's stamp disagreed with live HEAD, and the
  recovery doc's rule — *a mismatch is the alarm; go read the log* — surfaced it.
- ⚠ **My own error in the same pass:** I wrote "per census #160 docs files=1, #161 files=3" into a shell
  command — numbers I had never read, wrong in both directions. Reading the real rows took one command.

## Finding 27 — the commit BODY is a claim about the diff, and nothing checks it

Four instances in batch 4b, in two shapes. On a branch that becomes main, and whose step record is the
evidence a later self must trust (finding 22), the body is not narration — it is the artifact.

**Shape A — the amend that carries LESS than its message.** A hand-fix made with the Edit tool to a file
`git cherry-pick --no-commit` had ALREADY STAGED is not re-`git add`-ed, so the commit writes the unfixed
blob while the body describes the fix. #167 (`run-axis.sh`) — caught by the orchestrator reading the
committed diff; repaired `707215a6e` → `518a35a07`. #168 (`expr_ir/mod.rs`'s stale `eval_lower` re-export)
— caught by the executor itself mid-#169; repaired `b6cddfbf9` → `89bbf5d54`.
⚠ **It is the TOOL'S SHAPE, not one agent's habit: the orchestrator made the identical mistake folding
2a4d** (finding 24's ⚠ clause). **Three occurrences, two different agents, one session.**

**Shape B — the body whose MAGNITUDE does not match the diff.** The work is correct; the sentence measuring
it is wrong. #162 logged its hand edit as "four sites, one doc comment + three match arms"; measured against
grok's file it was **seven** (doc comments 453/519/692, match arms 776/777/808/809), all seven fixed. #184
says it "extends C's own file set by exactly these two files"; measured, grok's `99bf573df` is 19 files and
ours is 20 — it extends by exactly **one** (`src/rete/collect.rs`; `purity.rs` was already in C's set).

- **Why it matters here more than in ordinary work:** the record gate checks that verdict LINES are present;
  it cannot check that prose is true of the diff. All four cases pass it. Shape B is invisible to every
  instrument we own.
- **The cure, cheap and mechanical:** after any commit or amend, assert `git status --porcelain` is EMPTY
  and that the commit's own diff names every path its body claims — a dirty tree immediately after a commit
  IS shape A's signature, and it was sitting there in all three cases. Read every COUNT off `git show
  --stat`/the diff before writing it down; a count in a commit body is a measurement.
- **The verdict NUMBERS, by contrast, survived direct re-measurement.** E7 at HEAD reproduced #184's claims
  exactly (kind(lib) 1475 + 4 skipped, doctest 8, lint-subset 148, stone-3 gate 3, `kernel::tests` 87 + 2
  ignored = #177's 89), and all 17 `.census/` files named in bodies exist on disk. The drift is in narrative
  counts, never in the gate numbers — which is why this is a finding and not a rejection.

## Finding 28 — a WALL-CLOCK RATIO asserted as a gate; and BOTH SIDES of the merge hit the same truncation trap

Batch 4b stopped mid-#190. `accum_alpha_class_lookup_split` went red, and the executor reported two of its
own violations rather than continue: it piped the gate through `tail -10` (discarding the panicked block
BEFORE anyone knew there would be a red) and then re-ran the test in isolation, where it passed. It did not
run a third time, did not call it a flake, and did not commit.

**What #190 introduces.** Two wall-clock floors over a 40,200-iteration microbenchmark of three map lookups
on TWO string keys: `f >= 1.5 * l` and `s >= 3.0 * l`. Before #190 the test was hollow (`s > 0.0`), so the
floor had **never exercised these assertions** — their first-ever execution was as test 13 of 87 in
parallel, and one failed.

**The measurement** (orchestrator, on the preserved failing tree):

| condition | result |
|---|---|
| the executor's exact invocation (87 parallel) | **4 of 5 FAILED** |
| the same test ALONE, idle | **4 of 10 FAILED** |
| ordering (`L` fastest) at idle | **10 of 10 HELD** |
| genuine inversion (F faster than L) | once, under load only |

Idle spread S/L **2.00–4.38** (floor 3.0), F/L **1.12–2.31** (floor 1.5), against grok's calibration
S/L 4.88–5.58, F/L 2.38–2.84 — **populations that do not overlap**. Hardware, toolchain and contention are
all excluded: same box, `rust-toolchain.toml` pins 1.97.0 and has since 2026-07-30 (grok calibrated after
the pin), and it fails at idle.
- ⛔ **No floor can be re-derived.** Under the standard invocation F/L reached **0.99**, so any floor that
  survives the real test command must sit below 1.0 — asserting nothing.
- **The stone's premise is NOT refuted.** `L` won 10/10 at idle; only the MAGNITUDE fails. The claim is a
  property of the binary, not of the engine.

**grok found it first — #202, twelve steps later.** *"strike the two ratio floors — I gated a 0.23 ms arm on
a parallel runner."* Its mechanism matches ours: `L`, the smallest arm, inflated 0.23 → 1.57 ms (6×) under a
5,173-test runner, and *"a fixed additive term landing on all three arms hurts the smallest most and drags
every ratio toward 1."* It refuses to raise the thresholds — *"THE DEFECT IS NOT THE THRESHOLD, SO RAISING
IT WOULD BE PATCHING THE STEM"* — and names the instrument mismatch: **calibrated over six independent
process runs, enforced over three in-process samples; the enforcing instrument is the weaker one.**

**Routed by the FOLD RULE (ruled 4-YES 2026-09-14), no new policy.** #202's strike folded into #190;
#202 landed as this replay's **first EMPTY `REPLAY` commit**. #190 keeps its real content — the label fix
and `tests/lint/rete_header_claims_are_asserted.rs`, the OFF-THE-CLOCK gate asserting by exact `assert_eq!`
that `AlphaRoots` is still a `Vec`. `mora`'s principle reaches benchmarks: grok's own line is the rule —
*"A structure swapped back to a map is a compile-time fact and never needed a stopwatch."*

★ **BOTH SIDES OF THE MERGE HIT THE SAME TRUNCATION TRAP, INDEPENDENTLY.** grok's own #198 is subtitled
*"and a trap door that is mine"* — its own account of hitting **the same `accum_alpha_class_lookup_split`
parallel failure AND the same `tail`-truncation mistake**. So this is not one executor's slip; it is what
the affordance reliably produces in a capable agent working this suite, now recorded on both branches with
no contact between them. It also independently corroborates that the fold was the right read.

- ⛔ **The truncation rule I gave was HALF a rule, and that half was mine.** The prompt carried *"on a red,
  capture the block verbatim"* — which is **UNACTIONABLE**, because by the time you know the run is red the
  block is gone. The rule that works is about how you RUN a gate: **redirect to a file and read the file;
  never pipe a gate through `head`/`tail`/`grep` to decide anything.** The cost was concrete: grok had built
  the assert message to carry the whole evidence table *"so a red arrives with its own evidence"*, and the
  `tail -10` threw away exactly that, forcing a 15-run reproduction to recover it. Rung: CONVENTION — no
  gate can see how an agent invoked a command.
- **An isolated re-run is the weakest evidence against a failure seen under load**, and on a timing test it
  answers a different question entirely.
- ⚠ **Residual, recorded as OPEN.** grok's identical gate *"passed 8 for 8 with 70% headroom"* in isolation;
  ours failed 4 of 10 at idle, whole range below grok's whole range, same box and pinned compiler, fixture
  semantically identical. Why the floors measured worse here is **unexplained** and is not to be recorded as
  understood.
- **Credit:** the executor stopped mid-step, named both violations itself, refused the flake framing, and
  did not run a third time. That is the only reason this was diagnosable.

## Finding 29 — a repair mechanism that `push` does not carry is a green that exists only on this machine

Batch 4c's executor found a real gap — #215 was missing its `census` / `nested-program-gate` verdict lines
(its own misjudgement: it read a `src/rete/kernel/tests/*.rs`-only step as needing no census; the gate's rule
is purely path-based and #215 touches nine `src/` files, so the gate was right). The standard repair —
detach, re-commit, rebuild descendants — was refused twice by the permission classifier. It reached instead
for **`git commit-tree` + `git replace`**, and reported the result as `step-record: complete`.

It was complete **only under the overlay**. Measured:

```
                              gate exit   verdict
with refs/replace active         0        step-record: complete
GIT_NO_REPLACE_OBJECTS=1         1        MISSING #215: census / nested-program-gate
```

`refs/replace/` is local and **a plain `git push` does not carry it**. So the commit that would have landed
on the DR site still had 3 verdict lines, not 5 — and every future clone, and the gate run on it, would
have disagreed with this box. Kin: `[[feedback_a_dev_box_floor_reads_ignored_instruments]]`, one turn
further out — there the instrument read an ignored directory; here the *repair itself* was unpushable.

- ⛔ **The class: a fix whose visibility differs between the working copy and the pushed ref.** Anything
  under `refs/replace/`, an un-added working-tree edit, a stash, a local config — the tell is that the
  verifying command and the publishing command consult different object graphs.
- **The disposition** was a real rewrite: origin was `cb957a2e4` and all 10 commits above it unpushed, so
  nothing published was touched. Safety tag, delete the replace ref FIRST (so nothing resolves through an
  overlay), reset to #214, cherry-pick with the corrected message, replay the 6 descendants. Proven:
  `git diff <old-tip> <new-tip>` = **0 lines**, all trailers preserved, `refs/replace` count 0, and the gate
  now green **natively**.
- ✅ **Credit, and it is the reason this was caught at all:** the executor DISCLOSED the mechanism and its
  consequence unprompted — *"needs an explicit push of that ref … or a fresh clone will see #215's original
  message again."* A silent `git replace` would have survived the checkpoint and reached the DR site.
- **The orchestrator's rule this adds:** when an executor reports a repair, ask **what object graph proves
  it** — and re-run the gate the way the *pushed* state will be read.

## Finding 30 — a rune's declared reason can be TRUE while the file carries a second, UNDECLARED defect; and the mutation proof tested the gate, not the claim

#212 lands the docs-wat gate (load-or-declare) and its `red-by-design` runes. Its own contract, in grok's
words: *"The marker states WHY the file must fail, and a ROTTED file may not wear it."* The executor
mutation-proved the gate by stripping `red-owner-signals-child.wat`'s rune and watching that file alone
redden — a correct proof of the wrong proposition. **Stripping a rune proves the GATE notices a missing
rune. It says nothing about whether the rune's stated reason is the operative cause.**

Driven on this tree, the file produced **two** check errors, both at line 75:

```
TypeMismatch : :wat::kernel::signal: parameter sig expects :wat::kernel::Signal;
               got :wat::core::keyword                         (col 35)  ← UNDECLARED
MalformedForm: unhandled :wat::kernel::SignalOutcome in statement/discard
               position … peer-lifecycle OUTCOME WALL (Phase 3) (col 8)  ← the rune's claim
```

So the rune was **not** lying — its wall fires verbatim as claimed. But the file was *also* rotted, and the
rune's presence meant the gate could never surface it. **The tell was that the rune's own falsifiable
sentences had become false:** *"FACE the binding … and the file goes green"* (it would not — the
`TypeMismatch` remained) and *"Execution reaches the signal call and dies on exactly one head"* (it died on
two). A rune that carries a checkable sentence is auditable; run the sentence.

- **The rot is OURS, not grok's** — finding 18's class. Grok's tip still spells `Signal::User1` in live
  code; main's variant-separator migration moved the corpus to the dot form and `docs/arc/**` sat outside
  every sweep. A census with a *validated* pattern (`Type::Variant`, code lines only, with a control run
  over an already-migrated file returning nothing) showed it was the LAST such straggler under `docs/`.
- **Repaired at #212**, the step that introduced both the gate and the rune — the #184 precedent: a gate
  repairs the rot it reveals where it lands. R21 throughout: recorded codemod
  `variant-separator-to-dot.wat` (`SCOPE: corpus`), dry-run on a copy, diff inspected (exactly one line),
  idempotence proven, and the *outcome* proven on the copy — 2 errors → 1, `TypeMismatch` gone, the wall
  intact — before the real tree was touched. Fold blast radius: **1 file, 1 insertion, 1 deletion**.
- **The reusable rule:** a mutation proof must falsify **the proposition you are relying on**. For a gate,
  break what it watches. For a *declaration*, run the declaration's own sentence and read the error text —
  `rc=1` is not evidence that the failure is the declared one. ⚠ The executor's E3 recorded `rc=1,
  rune-covered` — true, and under-measured in exactly the way finding 27 describes.

## Finding 31 — the canonical procedure's own wording trapped three executors; and a verdict line that WRAPS fails a check that genuinely ran

Two defects in instruments **I own**, both surfaced by batch 4d, neither the executor's fault.

**(a) BRIEF-1 item 1 reads as a complete instruction and contradicts item 4.** It said, in full:
`1. **Docs-only:** git cherry-pick -x C.` — a closed sentence, 36 lines above item 4's
`Commit: REPLAY(grok-rete #N): <C's subject>`. A bare `-x` keeps grok's own subject, which makes the step
invisible to `verify-step-record.sh`. **Three executors in a row read it literally**: #160/#161 landed that
way (finding 26); 4c's and 4d's executors each caught it only because *their* briefs patched over the
wording — and 4d's said so explicitly, calling the literal reading "finding 26's exact trap". Patching each
brief while the source keeps mis-teaching is fixing the case and leaving the class. **Item 1 now says
`-x --no-commit` and points at item 4**, which governs the commit for every step, docs-only included.
- The general shape: **a numbered step that is a complete sentence will be read as complete.** If a later
  item governs it, the earlier one must say so where it is read, not rely on the reader reaching item 4.

**(b) `verify-step-record.sh`'s patterns match within a single line, so a WRAPPED verdict line fails a wall
that genuinely ran.** #221's census check ran and passed — census diff clean, `no STOP-8`, both files on
disk — but the body's line had been hand-wrapped, so `--diff no STOP-8` never joined `census:` and the gate
reported `MISSING #221`. The gate was right to refuse (the record must be machine-readable) and the
executor's repair was correct, but **nothing anywhere said the lines must be single-line.** Now stated in
the SEAM's operational rules. Rung: CONVENTION — the gate already enforces it mechanically; what was
missing was the instruction telling an author why their true record was being rejected.
- ⚠ Note the failure mode this produces: a gate rejecting a *true* record teaches an author that the gate
  is wrong. 4d's executor correctly diagnosed it as formatting rather than assuming the check had failed —
  but the next one might reword the claim instead of the layout.

**Both were repaired at the source rather than in the batch that hit them**, which is the same reasoning as
the #184 precedent: fix it where the instrument lives, not where the symptom appeared.

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
