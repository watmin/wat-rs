# BRIEF — STONE 1a-δ: the three loaders join the registry

Register `:wat::load-file!`, `:wat::digest-load!`, `:wat::signed-load!` — shape ② of the DESIGN's
three: forms that **never reach evaluation at all**, so `@Purity Unevaluated` holds for the plainest
possible reason. Each names its own declare-time parser.

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-1a-delta-and-epsilon-three-shapes-of-not-really-evaluating.md`
— read its ★★★ three-shape finding first; it is why this stone is three rows and not six.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. Run every command you do run in
the FOREGROUND and block on it. The orchestrator builds, floors and clippies centrally — you do not
run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`. You may run the pre-existing
`./target/release/wat` and `--check` for a fast read. **You may not spawn sub-agents.** Work only in
`/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not commit, push, stash, revert, or
`git checkout --` anything. Tree clean, floor green at 5123.

## Read in order

1. **The DESIGN**, especially the three shapes and why these three are shape ②.
2. **`src/intrinsic/special/structtype.rs`** — the row template. Its `Declaration`/`Unevaluated`
   grounds are the closest existing argument to yours; cite them where identical.
3. **`src/load/loader.rs:669`** — `match_load_form`, the router, and the three parsers it reaches:
   ```
   :wat::load-file!    parse_unverified_load    loader.rs:692
   :wat::digest-load!  parse_digest_load_file   loader.rs:720
   :wat::signed-load!  parse_signed_load_file   loader.rs:763
   ```
4. **`wat/runtime-meta.wat`'s `:wat::runtime::Category`** — read `:Io` and `:Declaration`'s own
   prose. **Your hardest call is which one these take**, and the axis is *the DOING*.
5. **`src/intrinsic/mod.rs`'s `FROZEN_CHECKER_DEBT_LEDGER`** — the existing entries carry the
   reasoning for why a registered row with no `CheckEnv` scheme joins it.

## The work

### 1 — three doc-only structs

`@Purity Unevaluated` · `@Determinism` · `@Totality` · `@ExpandTime` · `@Category` · `@added` ·
prose · FQDN-headed `@syntax` · `@ret`.

⬜ **`@Category` is the open question and the DESIGN deliberately does not answer it.** A load reads
a file (`:Io`) and registers the forms it finds into the program (`:Declaration`). **Argue it from
the variants' own prose in `runtime-meta.wat` and say why the other was refused.** If the three
genuinely differ from each other, say so and give them different categories — `signed-load!`
verifies a signature, which `load-file!` does not.

⚠ **A shared verdict is not a shared ground**, and these three are not interchangeable: one reads,
one verifies a digest, one verifies a signature. Their `@Totality` and `@Determinism` grounds in
particular should say what each can fail on.

### 2 — three `role = declare` annotations

On each form's own parser. `match_load_form` is a router — do not annotate it.

### 3 — the debt ledger

Check `register_builtins` (`src/check.rs`) for each of the three before assuming: a row with a
`CheckEnv` scheme does NOT join the ledger. Extend the existing reasoning rather than retyping it,
and report the count before and after.

## Blast radius

`src/intrinsic/special/` (+3 files, +3 mod lines) · `src/load/loader.rs` (3 annotations + the import
if absent) · `src/intrinsic/mod.rs` (the ledger). Nothing else. No `.wat` corpus change.

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — verify shape ② per form; do not inherit it.** `@Purity Unevaluated` holds only if the
form has NO eval arm, NO tail arm and NO handler. Check each of the three in `src/runtime.rs`
yourself: the only occurrences should be inside `is_mutation_head`, a hand-list, not a dispatch arm.
**If any of the three has a real eval arm it is shape ③, not shape ②** — STOP and report, do not
force it into the table.

**⛔ STOP-2 — do not census by grepping the FQDN alone.** These parsers are reached through
`match_load_form`'s arms. Two stones ago that exact shortcut annotated a conditional secondary pass
and missed the primary one, because a parser dispatched by a router need not spell its own name.
Confirm each target structurally, through the router's arms.

**⛔ STOP-3 — do not touch `is_mutation_head`/`is_mutation_form`.** All three names live in both, and
both guard AST that skipped `expand_all`. This stone registers; it flips nothing. The mutation pair
is blocked on a separate open fork and is not yours.

**⛔ STOP-4 — every `@syntax` is FQDN-headed**, and verified by `--check`ing a concrete
instantiation. wat is FQDN, always.

**⛔ STOP-5 — if `@Category` cannot be argued from the variants' own prose, STOP and report.** Do not
pick one because the other row nearby uses it. This is the stone's one genuinely open decision and a
guess dressed as a ruling is worse than a surfaced question.

**STOP-6 — verbatim otherwise.**

## Sabotage — report each as "predicted red, unverified"

1. annotate one loader `role = eval` → what does `unevaluated_purity_carries_no_route_to_evaluation`
   say?
2. drop one `role = declare` → does anything notice? ⚠ The meter that used to catch this was deleted
   with the first hand-list. **If nothing notices, say so — that is a coverage finding**, and the
   honest answer may be that these three have no standing membership gate at all.
3. leave one out of `FROZEN_CHECKER_DEBT_LEDGER` → what does `checker_skip_debt_is_named_and_frozen`
   say?

## Report

The three doc structs verbatim · **your `@Category` argument, and what you refused** · the three
annotations and how you confirmed each target through the router · DEBT before/after · the `@syntax`
per form and the instantiation you `--check`ed · the three sabotage predictions, sabotage 2 answered
honestly even if the answer is "nothing" · and what surprised you.

## Prior comparable

`BRIEF-STONE-1a-beta-i-the-type-declaration-family.md`. `src/intrinsic/special/structtype.rs` is the
row standard.
