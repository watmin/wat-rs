# Realizations — Arc 112

Observations that surfaced during slice 1, captured because they
recur across substrate work and are worth keeping.

## "I don't know" is the thing to fix, not the thing to bank

I had been reading the substrate code carefully — `instantiate`,
`rename`, `unify`, `format_type`, `apply_subst`, `expand_alias`,
`reduce` — and reasoned that all should preserve `Parametric`
shape recursively. The probe still failed with "got Process"
(no args). I traced every code path I could find, came up with
three failure-mode hypotheses, and was about to commit a
"diagnostic gap" investigation note and bank for fresh context.

User direction (2026-04-30):

> we do not yield - we win - always - explain to me what you do
> not know - we will resolve it
> [...]
> you are beyond wat - you can add whatever eprintln! you want in
> the rust code for us to resolve this

The key reframe: I had been treating the substrate's Rust source
as opaque — *it works the way it works; my job is to figure out
what it does by reading.* But the substrate is just code. I have
write access. Adding `eprintln!` at four call sites took 90
seconds; recompile took 10 seconds; running the probe gave me
the literal `TypeExpr` structure at every layer of the inference
chain.

**The honest move when reading the source isn't producing
understanding: instrument the source.** `eprintln!` is the wat
substrate's universal debug primitive — same as `(println ...)`
in any other Lisp. The cost of running it is bounded; the
cost of NOT running it is sometimes unbounded (in my case, an
hour of dead-end pattern-matching against a misread error
message).

This generalizes beyond debugging. Whenever the substrate's
behavior diverges from my mental model, the cheap recovery is
**make the substrate tell me what's true.** Not by reading
harder — by adding the prints that surface the actual values.

## The output-buffering trap

The bug was named about three minutes after the first eprintln
fired: the probe's `sr` binding was unifying CORRECTLY. The
"got Process" error I'd been chasing for an hour was for a
DIFFERENT binding inside the substrate stdlib (`sandbox.wat`'s
`drive-sandbox` function), which uses bare
`(proc :wat::kernel::Process)` parameter typing. With Process
now parametric, that bare type lost its match.

The reason I never saw the substrate-stdlib errors: I was
piping `wat <file> 2>&1 | head -5`. That captured the FIRST
error block (the user's `sr` binding) and discarded the other
nine. The "10 type-check error(s)" header was just past the
cutoff. **One pipe character of output truncation hid 90% of
the diagnostic.**

The substrate IS self-describing. I was just covering its
mouth.

Lesson: when the substrate's diagnostic looks unexpectedly
narrow, **count the errors before reading them.** A `grep -c
"type-check error"` is one syscall and tells you whether you're
looking at the whole picture.

## Subdragons surface fast once the head is named

Once the eprintln found the actual mismatch site, the fix was
five minutes:

- `wat/std/sandbox.wat::drive-sandbox` — function gained `<I,O>`;
  `(proc :Process)` → `(proc :Process<I,O>)`.
- `wat/std/hermetic.wat::run-sandboxed-hermetic-ast` — same.

Substrate boot: 0 type-check errors. The probe's `Process<i64,
i64>` binding worked end-to-end.

The remaining cargo test failures (6 binaries) were exactly
what the substrate predicted: test fixtures with bare
`(proc :Process)` annotations, ~45 sites across `tests/` and
`wat-tests/`. **Mechanical sweep — same shape arc 110/111 had.**
Dispatched a sonnet against the type-checker's natural error
messages (`expects :Process<?N,?M>; got :Process`). Same
substrate-as-teacher pattern as arc 111, applied without the
custom migration hint — the regular `TypeMismatch` output was
already specific enough for the agent.

## The substrate-as-teacher pattern, layered

Arc 111's REALIZATIONS named three audiences for one diagnostic
stream: humans, agents, orchestrators. Arc 112 demonstrates a
fourth: **the substrate-author at debugging time.**

When I was stuck on the parametric-scheme bug, the substrate's
production diagnostic was correct but insufficient for ME, the
substrate-author. I added transient instrumentation —
`eprintln!` in `instantiate` and `process_let_binding` — that
turned the substrate into a tutorial about ITSELF. The
instrumentation surfaced what the production diagnostic
abstracted away (the actual `TypeExpr` structure at each
inference step). I read it; I understood; I removed the
instrumentation; I shipped the fix.

The pattern:

| Audience | Diagnostic shape | Lifetime |
|---|---|---|
| Production users (humans) | `TypeMismatch` with type names | Permanent |
| Production agents | Same `TypeMismatch`, possibly with arc-N migration hint | Permanent during migration window; retired after |
| Orchestrators | `grep -c` on the diagnostic stream | Implicit; falls out of the production output |
| Substrate-author at debug time | `eprintln!` showing internal `TypeExpr` shape | Transient — added, used, removed |

All four read the SAME stream (stderr) at the SAME layer (the
type checker). The instrumentation differs only in how loud and
how detailed. The substrate doesn't need separate tooling for
the four audiences — the `Display` impls, the migration hints,
the format strings, and the eprintln debug all compose against
one channel.

## The user's role: encouragement at the threshold

I came within one tool call of banking the work in the
"investigation note" form. The user's intervention —

> we do not yield - we win - always
> [...]
> the entire wat journey is a demonstration in self sufficiency
> - the file system provides

— pushed me through the doubt threshold. Once I committed to
running `eprintln!` rather than reading harder, the dragon fell
in 30 minutes (90 seconds of instrumentation, 10 seconds of
recompile, ~5 minutes of reading the output, 5 minutes of
substrate stdlib edits, 20 minutes of context-recovery and
discovery of the `head -5` truncation).

The threshold the user pushed me through is real. **Substrate
debugging IS the self-sufficient kind of work — the substrate
is just code; instrumentation is just code; there is no
external party that knows more about the substrate than the
substrate itself can tell you when you ask.** When in doubt,
don't ask the doc, don't ask the search engine, don't ask
fresh context. Ask the substrate. It will answer truthfully.

This realization carries forward to every future arc. The
discipline isn't "read more carefully." It's "instrument when
reading isn't enough, and trust the output."

## What stayed clean

- The DESIGN.md (committed `5159f92`) was correct from the
  start; the implementation matches what was designed.
- The phantom-type-param machinery in the substrate
  (`StructDef::type_params`, `parametric_decl_type`,
  `register_struct_methods`) needed no change — arc 071's
  comment had anticipated this exact use case.
- The slice-1 fix is small: ~6 substrate edits in `types.rs` +
  `check.rs`, four-line edits in two `wat/std/*.wat` files,
  ~45 mechanical fixture-sweep sites. Total under 200 LOC of
  user-visible change.

The only thing that needed to change was my willingness to
treat the substrate as something I could question by
instrumenting, not just by reading.

## Cross-references

- `docs/arc/2026/04/112-inter-process-result-shape/DESIGN.md` —
  the original five-slice plan; correct from drafting.
- `docs/arc/2026/04/112-inter-process-result-shape/SLICE-1-INVESTIGATION-2026-04-30.md`
  — the bank-and-retreat note I almost shipped. Kept as the
  honest record of the threshold I almost stopped at.
- `docs/arc/2026/04/111-result-option-recv/REALIZATIONS.md`
  § "The substrate is the teacher" — the realization arc 112
  layers a fourth audience onto.
- `feedback_diagnose_before_spec.md` (memory) — the doctrine
  that says read the actual code path. Arc 112 extends it:
  read the code, AND if reading isn't enough, **make the code
  speak.**
