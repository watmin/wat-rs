# SCORE — STONE: the FORM is the authority for `->` (amended: delete Slot)

No commit. Floor and clippy left to the orchestrator. Mid-strike AMEND applied: STOP-1 lifted; Slot deleted **after** the lexical glue carried the load.

## Sequence — not inverted

```
1  land the lexical `->` glue
2  PROVE generic-fn  -> :wat::core::i64  BOTH TOKENS, SAME LINE
3  THEN delete Slot
4  PROVE row 2 again, plus foldl-bare / defmacro / defn
```

Ret-specs held **before** the deletion (previous segment) and **after** it. Nothing was restored.

## Row 2 — generic `fn`, after Slot is gone

`run-all.wat` on `generic-fn.wat`: `FORMS=1 COMMENTS=0 IDEMPOTENT=true`

```
(:wat::core::fn :- [:wat::core::i64]
  [acc <- :wat::core::i64 x <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::i64::+ acc x))
```

`-> :wat::core::i64` — both tokens, one line. The previous stone's defect (`->` then the type on the next line) is gone.

## Rows 3 / 4 / 5 — the three former Slot clients, after deletion

| fixture | driver | ret-spec |
|---|---|---|
| `foldl-bare.wat` | `run-all.wat` | `-> :wat::core::i64` same line, `IDEMPOTENT=true` |
| `defmacro-ret.wat` (new) | `run-all.wat` | `-> :wat::WatAST` same line, `IDEMPOTENT=true` |
| `defn-multi.wat` | `run.wat` (R1) | `-> :wat::core::i64` same line, `IDEMPOTENT=true` |

Removing Slot cost nothing. The form already knew where `->` was.

## Row 6 — Slot is GONE

```
grep -c Slot wat/fmt.wat              →  0
grep -rc Slot wat-scripts/fmt         →  0 everywhere
```

Deleted:

- the `:wat::fmt::Slot` record
- `q-slot`
- the builder (`slots-from-registry` / `slot-of-syntax` / `slot-of-form` / `find-arrow` / `any-variadic` / `variadic?` / `row-has-syntax?`)
- the map that folded Slots into `format-source`'s rete records
- R11's Slot join (already replaced by the lexical withhold before this AMEND)
- `glue-type-args-symbol` / `glue-type-args-keyword` (they existed only to assert Slot `{head, glued: 2}`)
- `run-slots.wat` (a driver that existed only to print it)

**Not deleted:** `[[NOTE-the-registry-already-knows-the-slots]]` and the three scratch-pad probes
(`277-does-the-registry-know-slots.wat`, `277-can-wat-read-its-own-grammar.wat`,
`277-locate-the-slot-in-a-grammar.wat`). The knowledge stays; the unused machinery is gone.

## The glue

R11 withholds a Break when the previous sibling is Named `"->"` **or** Named `":-"` (kind-checked via `Named`, so the string `":-"` does not match). The form is the authority. No index, no registry, no Slot.

`:-` **emitter** untouched (STOP-3): declaration is still a layout leaf; constructor still `force-leaf`s child 2 and explodes values.

## Row 7 — defclause

No fmt fixture. Nested `[-> :T]` is a **vector element**; the enclosing form's sibling-index cannot reach it. Inside that vector both children are atoms, so they ride even if R11 fires on the vector.

## Ruled shapes, after deletion

All `IDEMPOTENT=true`. Declaration one line (`type-nested`). Constructor glues type-args and explodes values (`type-ctor`). `let-two` / `half-broken` / `all-four` / `claim-demo` / `assoc-ride` / `let-complex` / `unruled-*` hold.

## Walls

Disagreeing-kind sabotage still raises `fmt: conflicting Breaks for node 11 — block vs align`. Deleted after. `ClaimedUnder` 0. `col` 0 in every rule file.

## Commands

| command | result |
|---|---|
| `cargo build --release` | clean |
| `generic-fn.wat` **after Slot gone** | `-> :wat::core::i64` both tokens same line, **IDEMPOTENT=true** |
| `foldl-bare` / `defmacro-ret` / `defn-multi` | same, no regression |
| remaining fixtures | ruled + idempotent |
| `run.wat` on `wat/io.wat` | **COMMENTS=28** |
| `grep -c Slot wat/fmt.wat wat-scripts/fmt/**` | **0** |
| kind-conflict sabotage | **raises**, then deleted |
| `every_wat_scripts_file_loads` (after deletion) | **1 passed** |

`git diff --stat`: `wat/fmt.wat` −105, `siblings.wat` ±, `run-slots.wat` deleted. `defmacro-ret.wat` new. No Rust.

---

## ORCHESTRATOR VERDICT — 2026-09-05

**ACCEPTED.** ⛔ **The floor went RED first — and the gate was right.**

| what | result |
|---|---|
| ★★★ row 2 — a GENERIC `fn`'s ret-spec | **`-> :wat::core::i64`, BOTH TOKENS, SAME LINE** |
| rows 3/4/5 — the three former `Slot` clients | `foldl-bare`, `defmacro-ret`, `defn-multi` — all one line, all idempotent |
| ★ row 6 — `Slot` is GONE | `grep -c Slot wat/fmt.wat` **0**; across `wat-scripts/fmt/` **0 files** |
| floor, **after the fix** | **5179 run, 5179 passed, 0 FAILED, 18 skipped** · clippy **0** |
| net | `wat/fmt.wat` **−105 lines**; 179 deletions against 55 insertions |

```
(:wat::core::fn :- [:wat::core::i64]              ← param-spec rides the head line
  [acc <- :wat::core::i64 x <- :wat::core::i64]
  -> :wat::core::i64                              ← THE RULING
  (:wat::i64::+ acc x))
```

**The sequence was honoured**: glue landed, row 2 proved, THEN `Slot` deleted, then proved again.
**Removing `Slot` cost nothing — the form already knew where `->` was.**

## ⛔ THE RED, CAPTURED — and it is a gate doing its job

```
FAIL ( 110/5179) wat::lint every_tracked_wat_parses::every_tracked_wat_file_parses

panicked at tests/lint/every_tracked_wat_parses.rs:52:5:
1 tracked *.wat file(s) do not parse. A .wat the reader cannot read is invisible to every corpus
tool that walks the tree — which is how two of them survived months. Fix the file, or rename it
`.wat.bad` if being unreadable is the point.
  wat-scripts/fmt/run-slots.wat — could not read: No such file or directory (os error 2)
```

**Not a formatter defect. A half-finished deletion**: `run-slots.wat` was removed from disk but left
TRACKED in git, and the gate enumerates *git-tracked* `.wat` files. Staged the deletion; the floor
went green.

★ **This is the third time this session a gate caught something a targeted run could not**, and the
first time it caught a *process* error rather than a code one. A deletion is not done when the file
leaves the disk; it is done when it leaves the index.

⚠ **On re-running after a red:** the doctrine forbids re-running to make a red disappear, because
that destroys the evidence. Here the ARM was captured whole *before* anything moved, the cause is
named in the failure text itself, and the re-run followed a FIX. That is the licensed order.

## Not disputed

The `:-` withhold added to R11 is **within the AMEND**, which said gluing `:- [T]` to `fn`'s head
line is correct — and it is what makes row 2's param-spec ride. The `:-` emitter path was untouched
(STOP-3): declarations stay layout leaves, constructors still explode their values. Row 7 answered
honestly — `defclause`'s `[-> :T]` is a vector ELEMENT, so a sibling-index test cannot reach it, and
both children are atoms so they ride regardless. The three walls stand. `wat/io.wat` **COMMENTS=28**.

## ⬜ THE NEXT GAP, visible in row 2's own output

```
  [acc <- :wat::core::i64 x <- :wat::core::i64]      ⛔ both args on ONE line
```

The builder's `foldl` ruling puts **one argument per line**:

```
    [acc :- wat.type/i64
     x   :- wat.type/i64]
```

`defn-args.wat` breaks the arg-spec vector for a `defn`; **nothing does it for an `fn`.** That is the
next rule file — and under the ownership discipline it is exactly one: a rule dispatching on `fn`'s
arg-spec vector, claiming it, breaking its children one triple per line.
