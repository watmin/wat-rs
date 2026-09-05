# RESUME — the registry onslaught, and the three named decisions it is blocked on

> **Builder, 2026-09-04:** *"make sure we know how to resume the registry work we're blocked on
> now... this doc-comment clean up is likely to take several compactions to resolve... so.. make
> sure our resumption state is trivial to restart."*

**Read this file to restart the registry work. It is not blocked on effort. It is blocked on three
decisions, each measured, each with its evidence on disk.** Nothing here needs re-deriving; every
number carries the command that produced it.

## GROUND — re-derive before trusting a single line below

```bash
./target/release/wat wat-scripts/scratch-pad/255-registry-census.wat     # rows · aliases · axes
./target/release/wat wat-scripts/scratch-pad/255-b0-what-actually-gates-the-rete-rows.wat
./target/release/wat wat-scripts/scratch-pad/255-b0-name-and-totality.wat
./target/release/wat wat-scripts/scratch-pad/255-b0-rows-without-handlers.wat
```

As of `1b372d76b` (2026-09-04): **571 rows · 85 SpecialForm · 52 alias · 74 RETE_OPS rows, 52
registered, 22 not.** Floor 5139/5139 + doctests green; clippy 0.

## ⛔ WHAT IS BLOCKED — 22 rete rows, in three populations. NONE is a labour problem.

### (1) TWENTY `OpClass::Fallback` rows — an alias is the WRONG MECHANISM, permanently

```
:wat::rete::i64::{+ - * / mod rem quot}  ·  :wat::rete::f64::{+ - * /}
:wat::rete::vector::get · :wat::rete::vec::get · :wat::rete::linkedlist::get
:wat::rete::string::subs · the */first trio · (re-derive: RETE_OPS rows with class OpClass::Fallback)
```

`src/intrinsic/special/rete_alias.rs`'s own header, first 36 lines, ★★★ paragraph: a `Fallback`
row **may never be aliased** — the alias check in `dispatch_keyword_head` fires BEFORE
`dispatch_keyword_head_value`'s `RETE_PREFIX` gate, so a 2-arg alias under a `Fallback` name makes
the 4-arg `:undefined` form unreachable and raises `ArityMismatch { expected: 2, got: 4 }`.

**This is not a theory and not a first discovery.** It broke eight live rete tests when Stone 2a's
DESIGN named `:wat::rete::i64::+`; the 2026-09-04 stone REPEATED that error, and its rider
reproduced the failure deliberately before reverting.

> **THE DECISION OWED:** a `Fallback` row's registry representation is not `alias_of`. What is it?
> A new `Kind`? A row with a `fallback_of` pointer? Or does the RULING's "every name answerable"
> accept that these answer through `RETE_OPS` rather than `registry()`? **Nothing on any list
> produces this decision.** It is the largest single block on the arc.

### (2) `:wat::core::cond` — the axes are decided, the KIND is not

Measured: registering it with `@Purity Preserving` (copied from `if`, which IS correct semantically
— `cond` is chained `if`, and `rete/purity.rs` already classifies it clause-aware that way) makes it
the only handler-less non-alias row with non-`Unevaluated` purity. `every_special_form_carries_check_
and_eval_impls` then demands `role=check` + `role=eval` impls, which a plain `defmacro` does not
have. Confirmed failing by name; confirmed passing on a byte-identical baseline without it.

> **THE DECISION OWED:** `@Purity Unevaluated` + `role=declare` is the shape every other
> handler-less row uses and it makes the gate pass — but it says something DIFFERENT about `cond`
> than `if` says about itself. Is `Unevaluated` true of a macro that expands to `if`? Decide the
> semantics; the mechanics follow.

### (3) `:wat::core::reduce` — resolution works, the CHECK-TIME witness does not survive

Measured: a `:wat::core::` name DOES dispatch through a registry `alias_of`
(`wat-scripts/scratch-pad/255-stop2-reduce-registry-alias-probe.wat`, negative-control verified).
But moving the alias off `wat/seq.wat` costs a guarantee: `probe_arc255_1c_f_reduce_2arity_retired`
refused a malformed 2-arity call at **check** time; with the registry alias the same call
type-checks and fails only at **runtime**, renamed to `:wat::core::foldl`.

> **THE DECISION OWED:** is a check-time arity refusal something the registry alias path must
> preserve, or is the negative witness re-drawn at runtime? **Do not delete the witness to make the
> move work** — a test of a retired behaviour is the retirement's evidence. Long-term the builder
> intends `foldl → reduce`, which changes this question's shape entirely.

## ✅ WHAT IS NOT BLOCKED — pick any of these and go

```
Phase 3b — check asks the registry        UNBLOCKED. 432/432 round-trip. Kills 302 of 325 duplicates.
the DEBT split                            121 = 41 wrong-shape + 60 stronger-authority + 20 owed.
the 270 both-axes grading batch           holon 91 · kernel 49 · time 41 · io 29. Clears 13 of the
                                          Unreviewed poles the rete aliases inherit. Gates nothing.
the :None codemod                         94 sites / 20 files, dry-run PROVEN with two negative
                                          controls, no design left:
                                          wat-scripts/fixes/bare-none-keyword-to-fqdn.wat
the SIX non-verb artifacts                need a RULING, not a registration — the corpus can NEVER
                                          reach 0 by registering. Phase 3a's real gate.
```

## ⛔ WHAT NOT TO DO ON RESUME — three traps this arc has already paid for

1. **Do not register a rete row without asking whether it is `Fallback`.** A census that asks
   *"is the core_name registered?"* is asking the WRONG QUESTION — that predicate passed all 20.
2. **Do not restate an axis on an alias row.** `AliasDeclaresAxis` is a real `compile_error!`
   (`crates/wat-macros/src/wat_intrinsic.rs`) — the build refuses it. An alias's axes ARE its
   target's, resolved after folding.
3. **Do not pre-compute a frozen ledger's new contents.** `REGISTRY_MEMBERSHIP_GAP_A`/`GAP_B`/
   `FROZEN_CHECKER_DEBT_LEDGER` go red naming exactly what changed. Let them name the edit.

## The paperwork, in reading order

```
RULING-the-registry-is-the-sole-authority.md               the doctrine
DESIGN-CAMPAIGN-the-registry-becomes-the-sole-authority.md 4 shapes, A picked
SEQUENCING-the-only-chain-that-gates-the-founding-target.md  ⚠ carries its own MEASURED-FALSE
                                                             correction — read that section
DESIGN-STONE-the-rete-vocabulary-enters-the-registry.md    the alias-vs-restriction resolution
WORKLIST-the-121-the-registry-cannot-vouch-for.md          re-derive, do not cite
```
