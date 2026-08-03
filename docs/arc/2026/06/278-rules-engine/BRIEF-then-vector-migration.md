# BRIEF — Stone A: `:then` becomes a vector of bare fact forms

Spec: `DESIGN-STONE-then-is-a-vector-of-singular-facts.md` § "Stone A". Read it first — it carries
the ruling, the reason the wrapper drops, and the bootstrap order this brief executes.

## You are a rider, not the orchestrator

**Ending your turn ENDS you.** Nothing wakes you; no notification is coming. Run every command in
the FOREGROUND and block on it. Your turn ends when the numbers are in your hands.

## The work, in one paragraph

Change `defrule`'s RHS surface from spliced varargs to a **vector**, and drop the per-entry
`(:wat::rete::insert …)` wrapper — the vector position now carries "insert this":

```clojure
;; BEFORE                                   ;; AFTER
:then                                       :then [(:wmv::Hit ?k)]
  (:wat::rete::insert (:wmv::Hit ?k))
```

**This changes NO semantics** — same facts, same order. Surface only. If you find yourself changing
what a rule *does*, you have left the stone.

## ★ THE BOOTSTRAP ORDER — read this before touching anything

**2 of the 54 files are STDLIB: `wat/core.wat` and `wat/rete.wat`.** Change the macro and its
stdlib callers break, so the stdlib fails to load, so the runtime cannot start, so the codemod
cannot run. Ordering is therefore forced:

1. **Change the `defrule` macro** (`wat/rete.wat:2150`) to read a vector, AND **hand-migrate the
   `defrule` call sites in `wat/core.wat` and `wat/rete.wat` in the same step.** This is the
   bootstrap `wat/fix.wat`'s STASH-DANCE header covers — hand-editing exactly these two is correct
   and is NOT the "no hand-edits" rule being broken (that rule governs the multi-site sweep).
2. **Confirm the runtime loads** — `cargo build --release` and a trivial `./target/release/wat` run.
   If it does not, STOP-1.
3. **Write and run the codemod** over the remaining 52 files.
4. Weigh. The tree is RED between steps 1 and 3 and that is expected — the commit is atomic.

**Fallback, only if step 2 fails:** widen the macro to accept both shapes, codemod, then narrow.
Costs a temporary second spelling and needs an AST-kind test the F5 pure-total allow-list may not
permit. Report before taking it.

## The macro change — it gets SMALLER

Today (`rete.wat:2150-2174`) the RHS is spliced varargs:

```clojure
then-forms (:wat::core::rest (:wat::core::rest (:wat::core::rest rest)))
…
(:wat::core::quote [~@then-forms])
```

carrying a long comment about `drop` having gone lazy (arc 118.2a) and `to-vec`/`into` not being on
the F5 pure-total allow-list. The new form is symmetric with `when-vec`:

```clojure
when-vec (:wat::core::get rest 1)
then-vec (:wat::core::get rest 3)
…
(:wat::core::quote ~when-vec)
(:wat::core::quote ~then-vec)
```

**Delete the gymnastics and their comment** — a comment explaining a workaround that no longer
exists is a lie waiting to be believed.

## The Rust side

`build_insert_fact` (`src/rete/matcher.rs`, ~`:567`) validates that each RHS entry is a List whose
head is the keyword `:wat::rete::insert`, then reads `items[1]` as the fact form. **After this
stone each entry IS the fact form** — so that head validation and the unwrap both go. Ground the
production-pass caller in `kernel.rs` too; it calls `build_insert_fact` directly with the form.

**Do NOT touch the session-level `:wat::rete::insert` defclause (`rete.wat:1004`)** — that is the
other meaning of the name and it survives. This stone retires only the RHS marker meaning.

## The codemod

**This is a `wat-fix` codemod** (CLAUDE.md item 1) — not hand-edits, not python, not sed.
`wat-scripts/fixes/` has twelve recorded exemplars; `drop-deftest-prelude.wat` is the closest in
shape (it deletes a span from inside a macro call form and was validated by dry-run + diff, which
caught a comment-eating bug).

Per entry the rewrite is: take the span of each `(:wat::rete::insert <fact>)` form after `:then`,
slice out `<fact>`'s own text span, and emit them joined inside `[ … ]`. Span-faithful — only what
the rule changes changes.

**Mandatory before applying to the corpus:** dry-run onto a `/tmp` copy and `diff` it. Then prove
idempotency — a second run must report **0 changes**. A non-idempotent recorded codemod is a bad
durable example (that lesson cost a prior strike).

Apply with every path listed:
`printf '["pathA" "pathB" …]\n' | cargo wat ./wat-scripts/fixes/<name>.wat`

**Scope, measured:** 197 `defrule` sites across 54 files — `tests/rete` 19, `wat-scripts/scratch-pad`
11, `wat-scripts/perf/grid` 11, `wat-scripts/fixes` 9, `wat/` 2 (hand, step 1), `tests/services` 2.
**Enumerate the file list yourself and report the count** — do not trust this number blind; a
hollow grep has produced a false scope in this arc more than once. Remember the non-`.wat`
extensions exist (`.wat.bad`, `.wat.disabled`, `.wat.expr`) — check whether any carry a `defrule`
before concluding the glob is complete.

## STOP triggers — rejection criteria. Ship nothing, report the gap.

1. **STOP-1 — the runtime will not load after step 1.** Report what broke; do not start the sweep
   against a broken stdlib.
2. **STOP-2 — a `:then` entry is not a plain `(:wat::rete::insert <fact>)`.** If any site holds
   something else (a bare form, a nested call, anything), halt and report the sites — the ruling is
   that every member is a singular fact, and a surprise here means the corpus disagrees with the
   stone.
3. **STOP-3 — the codemod is not idempotent.** Second run must be 0 changes.
4. **STOP-4 — semantics move.** Any `:derived` output changing, any rule firing differently. This
   stone is surface-only.
5. **STOP-5 — scope.** Do NOT add expression support to `:then` (that is Stone B, and it does not
   ship without the fence). Do NOT touch the session-level `insert` defclause. Do NOT arm any fence.
6. **STOP-6 — the `_` wildcard on an enum scrutinee is doctrine-illegal.**

## Gates — foreground, report every result line

```
cargo build --release --all-targets            # exit 0, ZERO warnings
cargo clippy --release --all-targets           # likewise
cargo test --release --test rete               # 244/0/9 at HEAD 6c57dc9f
cargo test --release --test lint               # 66/0 — includes every_wat_scripts_file_loads,
                                               #   which is your real corpus gate here
./wat-scripts/perf/grid/check-where-shapes.sh  # 9 pairs, 98 rows agreeing
```

**Do NOT run `cargo nextest run`** — the orchestrator weighs the whole floor centrally once your
tree is quiescent.

### The gate that decides whether this shipped

**`check-where-shapes.sh` must still report 9 pairs / 98 rows agreeing.** Those axes are
`defrule`-driven and compared against Clara — if the migration changed a rule's meaning, the
differential says so. That is the strongest non-vacuous signal available for a surface change, and
it is why STOP-4 is checkable rather than aspirational.

Two lint traps that have bitten repeatedly in this arc: a doc comment or assert message that
**parses as a wat list** trips `no_inlined_wat_in_tests`; a `contains(...)` on a rendered error
trips `no_loose_string_assert`. Fix at the root — **no `rune:lint`**.

## Do not

Do not commit, push, stash, or revert anything you did not write. Do not add `#[allow(dead_code)]`
to silence a signal — if something loses its last reader (the `insert`-head validation may), say so
in your report rather than muting it.
