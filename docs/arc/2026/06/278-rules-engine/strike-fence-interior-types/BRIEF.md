# BRIEF — type-check the `where` fence's interior

## The work

Fill `src/rete/validate/mod.rs:282`'s empty `ReteClauseShape::Where(_) => {}` arm so a predicate
inside a `(:wat::rete::where …)` fence is type-checked exactly as the same predicate written inline
in a condition. Today `string::=` over two `i64`-bound join variables is **accepted silently** in a
fence and is a `ConstraintTypeMismatch` inline — one predicate, two answers. The probe is already
committed and its gap arm is banked; your strike ends with that `#[ignore]` coming off.

**Read `DESIGN.md` beside this file first.** It pins the one contract decision and names the two
traps in reusing the existing check verbatim. Both traps are real and both were read off the
source, not supposed.

## Read in order

| room | why you are here |
|---|---|
| `docs/arc/2026/06/278-rules-engine/strike-fence-interior-types/DESIGN.md` | the contract decision + the two traps + what is affirmatively out of scope |
| `tests/rete/probe_arc278_fence_interior_types.rs` | your acceptance criteria, already written. Three tests; two are green today and must stay green |
| `src/rete/validate/mod.rs:272-295` | `validate_when_entry` — the arm to fill is `:282`. Note `Not`/`Exists` recurse right below it |
| `src/rete/validate/mod.rs:197` and `:230` | `binds = collect_rule_bind_types(when_conds, types)` — proof the map reaching your arm is **rule-wide**, so join vars resolve. No new plumbing |
| `src/rete/validate/typing.rs:392-490` | `check_constraint_head` — the function to reuse. Its `ClauseCtx` destructure at `:400` is what you must supply |
| `src/rete/validate/mod.rs:394-402` | `ClauseCtx`'s fields, and the note that `binds` is rule-wide |
| `src/rete/validate/typing.rs:598-660` | `resolve_operand_type`. **Source 1 is trap 1**: a keyword naming no declared field is a CONSTANT with its own type — which is why empty field arrays are correct for a fence |
| `src/rete/validate/typing.rs:245-253` | `is_non_field_keyword` — **trap 2**. Read its doc comment; it answers *"has `check_operand_field_ref` already reported this?"*, and in a fence that function never runs |
| `src/rete/validate/typing.rs:552-596` | `OperandType` — the knowable-wrong vs not-knowable distinction the over-rejection guard tests |
| `src/rete/clause.rs:159` | `classify_constraint_head` — the shared classifier that says whether a list head is a constraint |
| `src/rete/clause.rs:242-290` | `expr_is_provably_boolean` — the shapes a fence interior can take (`and`/`or`/`not`/`if`/`let`/`match` + any row) |
| `src/rete/validate/error.rs:244-261` | `ConstraintTypeMismatch` + `ConstraintTypeNotComparable` — what the inline twin raises |
| `src/rete/validate/error.rs:401` | its `Display` arm — match its register exactly |

## The prior comparable result — copy its shape

**D10 is this strike on the `:then` side, and it shipped 2026-09-02.** A surface that was not
type-checked while the rest of the language was; cured by one new error variant plus one `check_*`
function called from the arm that was empty:

- `src/rete/validate/error.rs:178-193` — `RhsFieldTypeMismatch`, and read its per-field doc
  comments: each says *why* that field exists and how it renders
- `src/rete/validate/typing.rs:743` — `check_then_field_type`, the producer
- `tests/rete/probe_arc278_D10_then_field_types.rs` — the probe, and its header's section
  *"⛔ Why the not-knowable fixture is the load-bearing one"*, which is the trap your
  `legal_fences_still_compile` test exists to catch

## Sketch

```rust
// mod.rs:282 — the arm fills. The comment is REWRITTEN, not deleted: it currently
// asserts the interior is out of scope, which is the claim being retired.
ReteClauseShape::Where(expr) => {
    check_fence_interior(expr, rule_name, binds, types, errors);
}
```

```rust
// typing.rs — walk every list node; check the ones the shared classifier recognises.
// Shape-agnostic on purpose: modelling and/or/not/if/let/match here would be a second
// hand-rolled grammar to drift from clause.rs's.
pub(crate) fn check_fence_interior(expr, rule_name, binds, types, errors) {
    if let WatAST::List(items, _) = expr {
        if let Some(WatAST::Keyword(head, _)) = items.first() {
            if classify_constraint_head(head).is_some() && items.len() == 3 {
                // operands resolve through the RULE-WIDE binds, with NO fact in scope
            }
        }
        for item in items { check_fence_interior(item, ...) }
    }
}
```

## Blast radius

`src/rete/validate/{mod,typing,error}.rs`, plus removing the `#[ignore]` in
`tests/rete/probe_arc278_fence_interior_types.rs` and the `rune:lint(bad-is-banked)` line in
`tests/rete/probe_arc278_fence_interior_types_fence.wat.bad`. No new types outside `error.rs`. No
change to `collect_rule_bind_types`, to `clause.rs`, or to any `.wat` under `wat-scripts/`.

## STOP triggers

Each is a rejection criterion: ship nothing on it, report the gap, and stop.

**STOP-1 — the census is large.** Filling this arm type-checks fence interiors nobody has ever
looked at, and the floor's red list **is** the census. If it names more than a handful of `.wat`
files, STOP. Report the full list verbatim. Those are real type errors and they are worth finding,
but fixing them across the corpus is a `wat-fix` codemod strike of its own — see STOP-3.

**STOP-2 — a `?`-prefixed name can be bound by a `let` inside a fence.** The walk assumes a fence's
`?var` operands always resolve through the rule-wide bind map. Confirm from `src/rete/clause.rs:242`'s `let`
handling and the reader whether `(:wat::rete::core::let [?x …] …)` is expressible. If it is, a
locally-bound `?x` would resolve to the wrong type or to `UnboundInThisRule`, and the design owes an
answer before the arm ships. Surface it; do not guess a scoping rule.

**STOP-3 — you find yourself editing a `.wat` file by hand.** A structural rewrite across `.wat`
files goes through the self-hosted codemod (`wat/fix.wat`, pattern in `wat-scripts/fixes/*.wat`),
never hand-edits and never python/sed. If the cure needs a corpus sweep, that is a separate strike.
The two probe fixtures under `tests/rete/` are yours to touch; the corpus is not.

**STOP-4 — `legal_fences_still_compile` goes red.** That is the over-rejection guard, and its row 3
carries a computed operand precisely because a cure that refuses what it cannot type passes every
refusal row and still breaks the corpus. If it reds, the cure is refusing not-knowable operands.
That is the D10 trap; re-read `OperandType`'s variants before changing the test.

## The open question — hand it back as an answer, either way

`src/rete/validate/mod.rs:456` is the clause-level twin: `ReteClauseShape::Where(_) => {}` with the
comment *"the stone-6 STOP arm (always `None` at fire time); its interior is out of scope."*

**I have not verified that claim, and I am not asserting it.** Determine whether a clause-level
`where` can carry a predicate that is ever evaluated. If it can, it needs the same check and the
strike covers both arms. If it genuinely cannot, say so **with the evidence that shows it** — that
verified negative is worth as much here as the cure, because it is the last place this hole could
still be hiding. If I got the framing wrong, saying so is worth more than the strike.

## Working rules

- `cargo nextest run --release`, never `cargo test`. The floor is `./scripts/floor.sh`, run in the
  **FOREGROUND**; read the `Summary` line out of `.floor/latest/clean.log`, never a piped exit code.
- One cargo build at a time — `pgrep -af 'cargo|nextest'` before you start one.
- **Commit only on green.** If the floor is red: revert, report, stop. Stage explicit paths.
- Use `git commit -F -` with a quoted heredoc (`<<'MSG'`); backticks in a `-m` string get
  command-substituted.
- On any red: **do not re-run.** `scripts/floor.sh` has kept the untruncated log — copy the failing
  test's entire stdout and stderr verbatim, name the exact assertion arm, and surface it.
