# BRIEF — STONE 1c-b-iii: build `is_type_equatable`, gate `infer_equality`, land the held two

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-1c-b-iii-equality-narrows-its-domain.md`

## The work, in one paragraph

`:wat::core::<` is `@Totality Total` because `infer_ordering` gates on `is_type_orderable`, which
narrows its declared domain to exactly what the runtime can compare. `:wat::core::=` is `Partial`
because `infer_equality` has **no such gate** — it declares `∀T`, `Fn` is inside that domain, and
`values_equal` returns `None` there, which `eval_eq` raises. **Build the sibling predicate, gate
equality on it, then grade and land `=`/`not=` with whatever the gate actually makes true.**

## Read in order

1. **`src/check.rs:12868`, `is_type_orderable`** — the shape you are mirroring. Read every arm,
   including the parametric recursion (`Vector`/`Option`/`PersistentMap`), the `Tuple` arm's
   non-empty guard, `TypeExpr::Fn { .. } => false`, and especially
   `TypeExpr::Var(_) => true, // unresolved — defer to runtime`.
2. **`src/runtime.rs:5302`, `values_equal`** — the real domain. Your allow-list mirrors **what
   this function actually has arms for**, not what seems reasonable. Its `_ => None` catch-all is
   the boundary. Note the deliberate absences its comments name (cross-numeric arms hard-cut at
   Stone 237.8c; `Value::Function` never handled).
3. **`src/check.rs:12817`, `infer_equality`'s `types_compatible`** — the insertion point. That
   test asks whether two types RELATE (unify / subtype / both-record / both-numeric). Yours asks
   a different question: is this type equatable **at all**. Both must hold.
4. **`src/rete/purity.rs`**, `intrinsic_meta`'s `total` fallback — `matches!(head,
   ":wat::core::reduce" | ":wat::core::=" | ":wat::core::not=")`. Its header states a homed name
   must leave it.
5. **`docs/arc/2026/06/255-builtin-registry/NOTE-equality-is-argued-proven-partial-and-held.md`** —
   the two held rows' doc blocks, kept verbatim. **Lift them; do not re-derive them.** Their
   `@Totality` line is the one thing you may change, per your own measurement.

## ⛔ THE LOAD-BEARING RISK — measure it FIRST, before writing the gate

```
wat/test.wat:61
(:wat::core::defn :wat::test::assert-eq :- [T] [actual <- :T expected <- :T] -> :wat::core::nil
```

**The test framework's own assertion primitive compares two bare type variables with
`:wat::core::=`.** A domain gate that rejects an unresolved type var does not fail one file — it
fails `assert-eq` and every test that uses it.

`is_type_orderable` already ruled this case (`Var(_) => true`, deferring to the runtime). Decide
the same case for equality **deliberately, and say why**. Then answer the question that ruling
raises, and **report the answer rather than assuming it**:

> If a type variable is admitted, every CONCRETE call site is gated and cannot raise — but a
> GENERIC body still defers to the runtime backstop. **Does that leave `:wat::core::=` genuinely
> `@Totality Total`, or `Total` only at concrete call sites with the hole surviving inside generic
> bodies?**

Ground the answer: construct a generic wat fn that compares two `:T` values, instantiate it at a
type `values_equal` has no arm for, and see whether `--check` refuses it at the call site or lets
it through to a runtime raise. **That experiment is this stone's most important output.** Either
result is shippable and honest; only an unmeasured claim is not.

## Then, per what you measured

- Grade `=`/`not=` and land the two held rows, lifting their doc blocks from the NOTE.
- If `Total`: `intrinsic_meta`'s by-name placeholder retires — its claim becomes **true** rather
  than assumed — and the four rete/sift fixtures must pass **unedited**.
- If `Partial`: the rows stay held, the NOTE is updated with what you learned, and the gate ships
  on its own merits (it still turns `(= <fn> <fn>)` from a silent runtime raise into a compile
  error, which is worth shipping alone).

## Blast radius

`src/check.rs` (the new predicate + the gate in `infer_equality`) · `src/runtime.rs` (the two
wrappers, only if you land them) · `src/intrinsic/mod.rs` and `src/rete/purity.rs` (ledgers and
the placeholder, only if you land them) · a `wat-scripts/scratch-pad/` probe for the generic
experiment. **No handler body changes. No fixture edits.**

## STOP triggers — halt and report, do not improvise

- **STOP-1.** `wat/test.wat`'s `assert-eq` stops compiling. **Do not narrow the test framework to
  fit the gate.** Report; the gate's type-var ruling is wrong.
- **STOP-2.** Any of the four rete/sift fixtures needs editing to pass
  (`probe_arc278_foreign_pred_purity`, `probe_arc278_sift_logs` ×2, `probe_arc278_sift_arena`).
  They compare a String field to a literal and genuinely cannot raise; if the gate refuses them,
  **the gate answered the wrong question** — report rather than edit a fixture.
- **STOP-3.** Your allow-list would admit a type `values_equal` has no arm for, or refuse one it
  does. The predicate must mirror the runtime, not approximate it. Report the mismatch.
- **STOP-4.** The corpus loses files to the new gate beyond the `Fn` case. Name every file and
  what it compares — that is a finding about the corpus, not a reason to loosen the gate.
- **STOP-5.** A test outside the ledger ratchets goes red. Copy its entire stdout and stderr block
  verbatim from `.floor/latest/raw.log`, name the exact assertion that fired, and report — before
  re-running anything.

## Verification, in this order

```bash
./target/release/wat --check wat-scripts/scratch-pad/probe-core-eq-is-partial.wat   # must now REJECT
cargo build --release 2>&1 | tail -20
./scripts/floor.sh > /dev/null 2>&1; echo "EXIT=$?"
grep -E "^\s+Summary" .floor/latest/raw.log | tail -2
cargo clippy --all-targets -- -D warnings 2>&1 | tail -20
```

★ `probe-core-eq-is-partial.wat` is committed and currently `--check`s clean while raising at
runtime. **After this stone it must fail `--check`.** That flip is the gate's proof it fires.

## Acceptance — derived, and deliberately conditional

```
is_type_equatable exists, mirroring values_equal's REAL arm set
infer_equality gates on it
probe-core-eq-is-partial.wat        --check exit 0  →  --check REJECTS      ⬅ the gate's proof
wat/test.wat assert-eq              still compiles                          ⬅ load-bearing
the four rete/sift fixtures         pass UNEDITED (if you land the rows)
=/not= grade                        PER MEASUREMENT — not predicted here
floor                               5128/5128
clippy                              0
```

⚠ `=`'s grade is deliberately not predicted. Two acceptance tables in this campaign were wrong
because they stated an expectation instead of deriving a bar. **Measure, then grade.**

## Working rules

Everything foreground. You may not spawn sub-agents. Do not background the floor run. No
worktrees, no `git stash`, no `git revert`, no commit, no push — leave the tree dirty and report;
the orchestrator commits. Report the generic-instantiation experiment in full — what you built,
what `--check` did, what running it did — because that result decides `=`'s grade and it is the
thing I cannot take on trust.
