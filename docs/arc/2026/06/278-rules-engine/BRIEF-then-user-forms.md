# BRIEF — `:then` admits user forms (Stone B)

Spec: `DESIGN-STONE-then-is-a-vector-of-singular-facts.md` § "Stone B", under the DSL-closure ruling in
`DESIGN-STONE-where-admits-only-rete-ops.md`.

**There is no new design here and none is owed.** The builder's sentence is the whole specification:

> *"the `:then` forms must produce a fact to be inserted — each item in the `:then` vec must be a fact.
> The user forms must be composed only from `:wat::rete::<namespace>::<fn-name>` — arbitrary complexity,
> rete-allowed primitives only."*

Same fence as `:when`, same three axes, same composition door. One addition: an item's value must be a
fact.

**Stone B's two stated `Open` items are both CLOSED** — do not re-derive them. *"Is `resolve_operand`'s
no-eval also a performance contract?"* is moot: `:then` expressions **compile**, they are not eval'd
per fact (`compile_rhs` already runs once per rule at setup, `kernel.rs:2237-2251`). *"Interaction with
#49a's compiled RHS"* is the same answer — one IR, two sites.

## You are a rider, not the orchestrator

**Ending your turn ENDS you.** Nothing wakes you; no notification is coming. Run every command in the
FOREGROUND and block on it. Your turn ends when the numbers are in your hands.

## The work — TWO widenings, one fence

```clojure
;; today — an item must be a record-constructor form, operands must be ?var / :field / literal
:then [(:usr::Rate :count ?c :window ?w)]

;; after — an item may be a CALL returning a fact, and an operand may be an EXPRESSION
:then [(:usr::make-rate ?c ?w)
       (:usr::Rate :count (:wat::rete::i64::+ ?c 1 :undefined 0) :window ?w)]
```

**(a) THE ITEM HEAD.** `validate_and_reorder_then` (`src/rete/validate.rs:624`) requires
`fact_items[0]` to be a `Keyword` that `lookup_fields` resolves to a known fact type, else
`UnknownFactType`. Widen: the head may instead be a **fn whose declared return type is a fact type**.
Note the branch — the kwargs *reorder-to-declaration-order* logic below that check applies to a
constructor, **not** to a fn call, which has its own argument convention.

**(b) THE OPERANDS.** `resolve_operand` (`src/rete/matcher.rs:516`) accepts `?var` / `:field` /
literal and returns `None` for anything else; `compile_rhs` (`src/rete/compiled_rhs.rs:78`) mirrors
that with `_ => return None`, falling the whole form back to the interpreter. Widen both to a
**compiled expression** — a third `RhsOp`, not an eval.

Both are governed by the identical fence: **pure ∧ deterministic ∧ total, every head
`:wat::rete::<module>::<op>`**, with the constructor / accessor / `sym.functions` doors intact. The
composition door is the point — a user fn of arbitrary depth is admissible exactly when it bottoms out
in rete primitives.

## ★ THE ONE CONTRACT DECISION — the fence hangs on the WAT side

`validate_rete_rules` (`src/rete/validate.rs:302`) and everything under it carry **`types: &TypeEnv`
and no `SymbolTable`**. The fence needs `sym` (that is where `sym.functions` and the fn return types
live). Threading `sym` through that whole Rust path is a param cascade, and this arc has a standing
rule against exactly that.

**So the fence goes where the `where` fence already is: in wat, at rule-compile time.**

- `wat/rete.wat:658` — `compile-condition`'s `where` branch: `(:wat::rete::pure? expr)` /
  `(:wat::rete::deterministic? expr)`, raising via `Option/expect` with
  `axis-violation-message` naming the offending head and axis. **This is the shape to mirror.**
- `wat/rete.wat:776-792` — the **accumulator** fence, and it is the closer precedent for widening (a):
  it cannot fence a bare head, so it **constructs a synthetic call** `(<acc-hd> __acc__)` and runs
  `pure?`/`deterministic?` on *that*. Same trick applies to a `:then` item head.

Consequently **`validate_and_reorder_then` RELAXES rather than tightens**: it stops requiring a
constructor head (it cannot resolve a fn's return type without `sym`), and the wat fence takes over
enforcing head-legality, the three axes, and returns-a-fact.

`:wat::runtime::return-type-of` exists (`check.rs:3254`, `runtime.rs:11608`) — a fn value in, its
declared return type as a String out. That is how "this item produces a fact" is checked.

## ⛔ Freeze-time, never fire-time

The fence is a **static walk at rule-compile**, exactly like `where`'s. It must not run per derived
fact. If you find yourself adding a check inside `exec_compiled_rhs` or `build_insert_fact`'s per-fact
path, you have put it in the wrong place.

## The gap this closes on the way past

`validate_and_reorder_then` validates the item's **shape** — head, fact type, field names, arity — and
**never inspects the value-position operands**. So today an unbound `?var` in a `:then` passes
`--check` clean and only surfaces at fire time (`compiled_rhs.rs` documents this). Once the fence reads
those operands, it becomes a compile error. Say so in your report if you confirm it.

## Read in order

1. `wat/rete.wat:640-680` — `compile-condition`'s `where` fence. The exemplar.
2. `wat/rete.wat:770-800` — the accumulator fence's synthetic-call trick. The exemplar for (a).
3. `src/rete/validate.rs:624` `validate_and_reorder_then` — what relaxes.
4. `src/rete/matcher.rs:516` `resolve_operand` and `:572` `build_insert_fact` — the interpreted path.
5. `src/rete/compiled_rhs.rs` whole file — `RhsOp`, `compile_rhs`, `exec_compiled_rhs`. Small.
6. `src/rete/kernel.rs:2237-2251` — where `compile_rhs` is called: **once per rule, at setup**, cached
   by rule name. This is why the fence and the compile both cost nothing per fact.
7. `src/rete/purity.rs` `head_ok` / `classify_expr` — the fence's Rust side; read it to understand what
   `pure?`/`deterministic?` do, do not modify it.

## STOP triggers — rejection criteria. Ship nothing, report the gap.

1. **STOP-1 — threading `sym` into `validate_rete_rules` turns into a param cascade.** That is the
   signal the check belongs in wat, not Rust. Report and re-aim; do not thread it.
2. **STOP-2 — an item head that is neither a fact-type keyword nor a fn returning a fact type.**
   Report the form. Do not widen "produces a fact" to accommodate it.
3. **STOP-3 — a `:then` expression that the fence admits but `compile_rhs` cannot represent.** Report
   it rather than falling the whole form back to the interpreter silently; a fenced form that cannot
   compile is a finding about the IR, and it is #49a's information.
4. **STOP-4 — the `_` wildcard on an enum scrutinee is doctrine-illegal.** Name every variant.
5. **STOP-5 — scope.** Do NOT arm the `where` fence or change `compile-condition`'s existing gate (that
   is #57). Do NOT mint vocabulary. Do NOT touch `:when`.

## Gates — foreground, report every result line

```
cargo build --release --all-targets            # exit 0, ZERO warnings
cargo clippy --release --all-targets           # likewise
cargo test --release --test rete
cargo test --release --test lint               # incl. every_wat_scripts_file_loads
./wat-scripts/perf/grid/check-where-shapes.sh  # 9 pairs, 98 rows agreeing
```

**`check-where-shapes.sh` is the gate that decides whether this shipped.** Those axes are
`defrule`-driven and differential against Clara: if widening `:then` changed what any existing rule
derives, it says so. This stone must be **observationally inert** on every rule that exists today —
new capability, zero behaviour change.

**Do NOT run `cargo nextest run`** — the orchestrator weighs the whole floor centrally.

Two lint traps that recur in this arc: a doc comment or assert message that **parses as a wat list**
trips `no_inlined_wat_in_tests`; a `contains(...)` on a rendered error trips `no_loose_string_assert`.
Fix at the root — **no `rune:lint`**.

## Prove it both directions

A RED gate is required, not optional: a `:then` item calling a user fn composed of **core**-namespaced
ops must be **refused at compile** with the offending head and axis named; the same fn written in
`:wat::rete::` ops must compile and derive the fact. Without the refusing half, the fence is
unmeasured.

## Do not

Do not commit, push, stash, or revert anything you did not write. Do not add `#[allow(dead_code)]` or a
`rune:lint` to silence a signal. Do not eval a `:then` expression per fact — compile it.
