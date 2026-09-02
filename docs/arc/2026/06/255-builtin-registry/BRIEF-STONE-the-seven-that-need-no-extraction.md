# BRIEF — STONE: the seven that need no extraction

Register seven verbs that already have named eval handlers. DESIGN:
`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-seven-that-need-no-extraction.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`.
**You may not spawn sub-agents.** Work only in `/home/john/work/holon/wat-rs`; verify with `pwd`
first. Do not commit, push, stash, revert, or `git checkout --` anything. Tree clean, floor green at
5118, HEAD `eaf24fcfa`.

## Read in order

1. The DESIGN, whole — especially § "The axis that must not be guessed" and § "Expect THREE ledgers
   to move".
2. **An existing `#[wat_intrinsic]` with a full directive block** — `src/intrinsic/collection.rs`'s
   first few. That is the shape: the doc comment carries `@arg`/`@ret`/`@added`/`@Category` and the
   five axes.
3. **`src/macros/eval.rs`'s `is_expand_time_legal`** — its fallback `matches!` residue list. You
   must check every one of the seven against it *before* writing `@ExpandTime`. See STOP-1.
4. `src/rete/purity.rs`'s `KNOWN_UNREVIEWED` — `:wat::core::show` is on it.

## The work

### 1 — annotate the seven handlers

Each already has a named eval handler and a `register_builtins` TypeScheme. Add
`#[wat_intrinsic(":wat::core::<name>")]` to the handler, with a full directive block:

```
:wat::core::bool::to-string   eval_bool_to_string                          (src/runtime.rs)
:wat::core::i64/to-f64        crate::numeric::convert::eval_i64_to_f64
:wat::core::i64/to-string     crate::numeric::convert::eval_i64_to_string
:wat::core::u8                crate::numeric::convert::eval_u8_cast
:wat::core::record?           crate::record::access::eval_record_q
:wat::core::not               eval_not                                     (src/runtime.rs)
:wat::core::show              eval_show                                    (src/runtime.rs)
```

⚠ **Verify each pairing against its dispatch arm before annotating.** The table came from reading
arms, but a previous brief's pairing was wrong and the rider caught it by checking. **The arm is the
truth; the brief is a claim.**

⚠ If a handler's Rust signature is not the shape `#[wat_intrinsic]` expects (it sniffs arity from
fixed args), STOP and report — do not reshape the function. STOP-5.

### 2 — the directive blocks

`@added`, `@Category`, `@arg`×N, `@ret`, at least one `@example`, and all five axes. Ground each
axis in one sentence from the verb's own body, the way `src/intrinsic/special/control_flow.rs`'s
`If` grounds its four. Do not copy a block between verbs — `i64/to-f64` and `i64/to-string` look
alike and are not the same function.

★ `@arg` types must match the verb's registered `TypeScheme`, because
`doc_arg_ret_types_match_checker_scheme` will compare them the moment the row exists. Read the
scheme in `register_builtins` (`src/check.rs`) and write the `@arg`/`@ret` types from it.

### 3 — move the ratchets

- `REGISTRY_MEMBERSHIP_GAP_A`: delete all seven (94 → **87**).
- `REGISTRY_MEMBERSHIP_GAP_B`: delete `:wat::core::bool::to-string` only (119 → **118**) — it is the
  only one of the seven in the fixed 121-name corpus census. ⚠ Deleting any other name from Gap B
  trips its FOREIGN check; deleting none trips STALE.
- `KNOWN_UNREVIEWED` (`src/rete/purity.rs`): delete `:wat::core::show`. **The DESIGN predicts the
  completeness gate will demand exactly this.** If you believe another of the seven is also on that
  list, check and report — do not delete a line the gate has not earned.

## Blast radius

`src/runtime.rs` · `src/numeric/convert.rs` · `src/record/access.rs` (attributes only, no bodies) ·
`src/intrinsic/mod.rs` (two frozen lists shrink) · `src/rete/purity.rs` (one line) · whatever the
compiler names. No `.wat` corpus change. **No verb changes behaviour.**

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — `@ExpandTime` MUST BE CHECKED AGAINST THE RESIDUE, NOT GUESSED.** `macros/eval.rs`'s
`is_expand_time_legal` consults the registry FIRST and falls back to a hand-list only for
unregistered names. **The moment you register one of the seven, its declared `@ExpandTime` REPLACES
its residue entry.** So: if a verb is on that residue list today, it is legal inside macro bodies
today, and declaring `Unreviewed` for it SILENTLY MAKES IT ILLEGAL — a real behaviour change. Last
stone this exact trap would have made `(fn ...)` illegal in every macro body; the rider caught it.
Check each of the seven, report which are on the residue, and declare accordingly.

**⛔ STOP-2 — `(:wat::holon::Bogus 1 2)` MUST STILL TYPE-CHECK CLEAN.** Unchanged from last stone: an
acceptance row, not an oversight.

**⛔ STOP-3 — DO NOT TOUCH `is_reserved_prefix`.** `grep -c "if is_reserved_prefix(head)"
src/resolve/walk.rs` must still be **1**. Flipping it fails 578 of 599 corpus files — measured.

**⛔ STOP-4 — NO EXTRACTION.** Eleven Gap A names have INLINE eval arms (`Vector`, `HashMap`,
`HashSet`, `Tuple`, `filter`, `foldl`, `find-last-index`, `stream->vec`, `stream->pvec`,
`rete::insert$native`, `stdlib::sources`). They are visibly next and are NOT this stone. Registering
one requires extracting its arm into a named function first — a different stone shape.

**⛔ STOP-5 — DO NOT RESHAPE A HANDLER TO FIT THE MACRO.** `#[wat_intrinsic]` sniffs arity off the
Rust signature. If a handler takes `&[WatAST]` it registers Variadic; if that is wrong for the verb,
that is a finding to report, not a signature to change. Bodies and signatures stay verbatim.

**⛔ STOP-6 — `subtype?` and `conforms?` ARE NOT YOURS.** They sit beside these seven, look identical
in the dispatch match, and are CUT by the DESIGN: inline check arms rather than schemes, and
`KNOWN_UNREVIEWED` purity. Registering them means ruling them.

**STOP-7 — you cannot run the gates.** State per ledger what you changed and what you expect the gate
to say; report it as **unverified reasoning**, explicitly. The orchestrator runs them. Naming the
limit is the discipline — the last two riders did this correctly.

## Report

Per-file diff summary; the seven directive blocks verbatim **with the grounding sentence per axis**;
**which of the seven are on `is_expand_time_legal`'s residue list and what you declared for each**;
the dispatch arm you verified each pairing against; the `TypeScheme` you read each `@arg`/`@ret` from;
the three ledger edits with expected gate outcomes (unverified); confirmation of STOP-3's grep. Then:
**what surprised you** — an axis you could not ground, a scheme that disagreed with the verb's doc, a
handler signature the macro would misread, or a name you expected on the residue list and did not
find.
