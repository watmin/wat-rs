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

## `convert.sh` does not run `one-param-spec` the way this check did

`scripts/replay/convert.sh` gives `one-param-spec` the converted file as its WHOLE context
(`printf '["%s"]\n["%s"]\n' "$WORK" "$WORK"`). `run.sh` gave it every file in scope. The tool learns
user-type arity from its context by design ("a type declared in file A resolves for a call site in
file B", its header), so at replay time a grok file using a parametric type declared in ANOTHER file
is left unconverted. Main's param-spec wall then refuses it (STOP-2, loud). The composition result
therefore does not describe `convert.sh` for this step.

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
- A codemod piped EMPTY stdin dies with `"disconnected"`. That is never-ran, not "0 unresolved".
