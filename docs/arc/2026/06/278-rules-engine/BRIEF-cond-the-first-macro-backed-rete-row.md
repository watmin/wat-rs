# BRIEF — `cond` reaches the rete surface, and `do` is cut

**Two rulings, builder, 2026-08-05.**

> *"having a solution to kill if chains would be great (i know they expand into them at macro time…
> but giving that us is great)"* — mint `cond`.
>
> *"i think do can go… i almost only ever use do for a stdout write or something similar."* — cut `do`.
>
> *"we'll swap it later… arc 109's purpose is dealing with renames — make it basically in parity with
> core's cond now — a new name comes later."* — **ship the paren form, in parity with core.**

Anchor `/home/watmin/work/holon/wat-rs/`; verify with `pwd`. Floor at draw time
**`4356 / 4356 / 0 / 262`**, clippy clean, gate `9 pair(s), 98 rows`.

⚠ **Row count depends on `string::subs` landing first** — that stone is in flight and also edits
`RETE_OPS`. Read the live count; do not assume.

---

## ★★ THE LOAD-BEARING FINDING — `cond` is the FIRST macro-backed rete row

Every existing `Form` row is a **runtime special form**. `cond` is the inverse. Measured:

| | `defmacro` in `wat/core.wat` | runtime arm in `runtime.rs` |
|---|---|---|
| `and` `or` `not` `if` `let` `match` `fn` | **0** | 1–5 each |
| **`cond`** | **1** (`wat/core.wat:1237`) | **0** |

`dispatch_rete_op`'s `Alias \| Form \| Redispatch` arm re-invokes `dispatch_keyword_head_value(op.core_name, …)`
**at runtime**. A macro is *gone* by runtime — it expanded at expand time. So **the existing `Form`
mechanism cannot reach `cond`**, and a row alone will not work.

And the reason is exact: macro lookup is by **exact name** —
`src/macros/registry.rs:48`, `self.macros.get(name)`. `:wat::rete::core::cond` is simply not that
key, so it never expands and then never dispatches.

**This is the same shape as the `nth` STOP**: the first row of a new *kind*, where the existing
machinery silently does not reach it.

### The likely path — GROUND IT, do not assume it

Register the `:wat::core::cond` `MacroDef` under the rete name as well, **driven by `RETE_OPS`**
(for each `Form` row whose `core_name` names a macro, alias it) — so the table stays the one door
and no name is hand-written twice. `MacroRegistry`'s own `register` already guards redefinition
(`registry.rs:58`), so read that before adding a second key.

**⛔ If a table-driven alias is not reachable from where macros are registered, STOP and report.**
Do not hand-register `cond` at a call site, and do not special-case it in `expand.rs` — either is a
second door around the one table, and the naming-rule tests exist precisely to keep that from
happening.

---

## PART A — the `cond` row

```rust
type_params: &[],
rete_name:   ":wat::rete::core::cond",
core_name:   ":wat::core::cond",
class:       OpClass::Form,
params:      &[],            // Form rows carry no TypeScheme
ret:         <the Form convention — copy `core::match`'s row exactly>
meta:        OpMeta { pure: true, deterministic: true, total: true },
```

**`total: true` is earned, not asserted**: `cond` expands to nested `:wat::core::if`, which is
already `total` in `purity.rs`'s list, and the expansion introduces nothing else. A `cond` with no
terminal `:else` is a **macro-expansion error** (`macro-error "cond: non-exhaustive"`), not a runtime
domain hole — it cannot reach the fence at all. Confirm both claims rather than taking them here.

### Syntax — PARITY WITH CORE, paren clauses, ruled

```clojure
(:wat::rete::core::cond
  ((:wat::rete::core::keyword::= tier :gold)   0.5)
  ((:wat::rete::core::keyword::= tier :silver) 0.7)
  (:else                                        0.9))
```

Clauses are **Lists** `(test body)`, terminal `(:else body)` — verified by run against core's macro,
which rejects vector clauses. **Do NOT implement bracket clauses.**
`docs/arc/2026/04/109-kill-std/NOTE-match-cond-clause-brackets.md` records the bracket flip AND a
ruling that the bracketed form must not be called `cond` — that is arc 109's rename, explicitly
deferred by the builder this session. Parity now; the swap later.

### The structural guard

`cond` is a **structural guard**, not a head-table entry — `src/rete/purity.rs:946` already matches
`:wat::core::cond` in `classify_expr`. It needs widening to recognise the rete name. **`match` is the
shipped exemplar for this exact widening** (arc 278 #56 phase 2 used `resolve_core_name` as the
single discriminator so the guard body is not duplicated) — copy that recipe; do not invent one.

Per the where-stone's STOP: **a structural-guard widening and a head-table entry are different
edits. Do not conflate them.**

---

## PART B — `do` is CUT, and the derivation is the point

**Do not mint `:wat::rete::core::do`.** Record it as an affirmative cut in `RETE_OPS`'s module doc,
*with the reason*, so no future round re-opens it:

> `do` evaluates every non-final form and **discards its value**
> (`eval_do_tail`, `let _ = eval_inner(arg, …)`), returning the last. In a `where` — which the fence
> guarantees pure ∧ deterministic ∧ total — a discarded pure value cannot affect anything, so
> `(do a b)` ≡ `b`, always. It is not *unused*; it is **incapable of meaning**. Its other role
> (def-position splicing, `register_runtime_defs_form`) needs a definition context, and a `where` is
> an expression. Builder, 2026-08-05: *"i almost only ever use do for a stdout write"* — the effectful
> use, which the fence forbids. Cut on derivation, **not** on corpus demand (R60).

---

## ⛔ STOPs

- **⛔ STOP-1 — if the macro-alias path is not table-driven, STOP and report.** No hand-registration,
  no `expand.rs` special case.
- **⛔ STOP-2 — paren clauses only.** No bracket form. That is arc 109's, with its own naming ruling.
- **⛔ STOP-3 — do NOT touch `wat/core.wat`'s `cond` macro.** The rete name reaches the same macro;
  core's definition is untouched.
- **⛔ STOP-4 — do NOT mint `do`.** Record the cut.
- **⛔ STOP-5 — do NOT conflate the structural-guard widening with a head-table entry.**
- **⛔** No `_` wildcard arm on an enum scrutinee.
- **⛔** Do not commit, stash, push, or touch git.

## Verify — FOREGROUND, block, SOLO

```
cargo build --release
cargo nextest run --release
cargo clippy --release --all-targets
./wat-scripts/perf/grid/check-where-shapes.sh
```

Read the **Summary line**, never a piped exit code.

## EXPECTATIONS

| # | what | expected |
|---|---|---|
| 1 | row count | live count **+1** |
| 2 | ★★ **the rete spelling EXPANDS and RUNS** | the tier ladder above evaluates to `0.7` for `tier = :silver` — the whole stone; if the macro alias is missing this fails |
| 3 | ★★ **first-match wins, not last** | reorder so two tests are true; the FIRST body is taken |
| 4 | ★ `:else` terminal fires | all tests false → the `:else` body |
| 5 | ★ **non-exhaustive is a located error** | a `cond` with no `:else` → `macro-error "cond: non-exhaustive"`, located, not a silent nil |
| 6 | ★ **it composes in a real `where`** | a `defrule` whose `where` uses the ladder against a bound field; `fire-rules` selects correctly |
| 7 | ★ core's `cond` unchanged | `(:wat::core::cond …)` still works identically |
| 8 | ★ naming-rule tests pass | all four; the row should need no exception |
| 9 | ★ every other family unregressed | one `Alias`, one `Fallback`, one other `Form` (`if`), one `Redispatch` |
| 10 | ★ floor / gate / clippy | floor ≥ live baseline · `9 pair(s), 98 rows` · clean |

Rows 2, 3, 5, 6, 9 re-run by the orchestrator by hand.

**Runtime prediction: 50–80 minutes** — the macro-alias mechanism is the unknown. Time-box 160.

**Trap doors:**
1. **Adding the row and stopping.** The `Form` arm is runtime; the macro is expand-time. Row 2 is what
   catches this.
2. **Hand-registering the alias.** STOP-1 — the table is the one door.
3. **Implementing bracket clauses** because they read better. They do, and it is arc 109's call.
4. **Widening only the guard, or only the head table.** Different edits (STOP-5).
5. **Asserting `total` from the row rather than from `if`'s classification.** Check it.

## Scratch

Scratch `.wat` in `wat-scripts/scratch-pad/` — never a tmp dir; it is parsed and type-checked on
every build. Real program, real `:user::main`. **Read core's `cond` macro (`wat/core.wat:1237`) before
writing any clause** — the orchestrator got the clause syntax wrong three times today by writing from
memory instead of reading the definition.
