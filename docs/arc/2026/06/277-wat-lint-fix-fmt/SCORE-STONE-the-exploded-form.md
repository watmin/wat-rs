# SCORE — STONE: the exploded form, and blank lines after a complex binder

No commit. Floor and clippy left to the orchestrator. Part 1 was green before part 2 started.

## PART 1 — leading-atom run, not "break every child"

**The one-line `do` explodes** (`claim-demo.wat`). `IDEMPOTENT=true`.

```
(:wat::core::do
  (:wat::kernel::println "a")
  (:wat::kernel::println "b")
  n)
```

**A leading atom rides** (`assoc-ride.wat`):

```
(:wat::hashmap::assoc m
  (:wat::i64::+ b 1)
  (:wat::i64::* b 2))
```

**A leading compound means nothing rides** (`foldl-bare.wat`): `fn`, `0`, `xs` each own a line.

`rete::or` is not legal inside `:where` (`"where expr is not pure"`). Four kind-rules (list/vector/map/set) instead.

### Findings — R11 is more active, as designed

Unruled type applications explode (`(:wat::core::Vector :- [T])` → `:-` rides, the arg vector breaks). Match arms are not claimed by R4, so `(_ (+ a b))` in `all-four.wat` now breaks the compound body. One arm per line still holds. Both idempotent. Coverage, not a softened rule.

Ruled shapes (`defn-multi`, `defn-empty`, `let-two`, `half-broken`, `unruled-*`) unchanged.

## PART 2 — `BlankBefore`, from structure

Complex first binder (`let-complex.wat`) — blank line before the next binder. `IDEMPOTENT=true` (blanks do not accumulate). Never before the first binder.

Simple binders (`let-two.wat`) — **no** blank. Output unchanged from before this stone.

Trigger: previous binder's value has a compound child. Same test as part 1. Not span/whitespace.

## Walls

Disagreeing-kind sabotage still raises `fmt: conflicting Breaks for node 11 — block vs align`. Then deleted. `ClaimedUnder` 0. `col` 0 in every rule file.

## Commands

| command | result |
|---|---|
| `cargo build --release` | clean |
| `run-all.wat` on `claim-demo` / `assoc-ride` / `foldl-bare` | exploded shapes, **IDEMPOTENT=true** |
| `run-all.wat` on `let-complex.wat` | blank between binders, **IDEMPOTENT=true** |
| `run-let.wat` on `let-two.wat` | no blank, unchanged |
| existing ruled fixtures | ruled + idempotent |
| `run.wat` on `wat/io.wat` | **COMMENTS=28**, IDEMPOTENT=true |
| kind-conflict sabotage | **raises** |
| `every_wat_scripts_file_loads` | **1 passed** |

---

## ORCHESTRATOR VERDICT — 2026-09-05

**ACCEPTED — the strike built exactly what was specified. ⛔ AND THE SPECIFICATION WAS WRONG, which
its own output proves.** The finding is mine.

| what | result |
|---|---|
| row 3 — a leading ATOM rides | `(:wat::hashmap::assoc m` ✓, both calls break |
| row 4 — a leading COMPOUND means nothing rides | `foldl` head bare; `fn`, `0`, `xs` each own a line ✓ |
| row 8 — complex binder → blank line | ✓ before `j`, never before the first binder |
| row 9 — simple binders → NO blank | `let-two` byte-identical to before the stone ✓ |
| row 10 — blanks do not accumulate | **`IDEMPOTENT=true`** ✓ |
| the three walls | disagreeing-kind sabotage still raises; `ClaimedUnder` 0; `col` 0 |
| floor | **5179 run, 5179 passed, 0 FAILED, 18 skipped** · clippy **0** |

Row 3 was the row that made row 2 non-trivial, and it holds. Row 9 was the failure that looks like
success, and it holds.

## ⛔⛔ THE FINDING — R11 DAMAGES ANY FORM THAT HAS A GRAMMAR BUT NO RULE

`foldl-bare.wat`, verbatim output:

```
    (:wat::core::fn
      [acc <- :wat::core::i64 x <- :wat::core::i64]
      ->
      :wat::core::i64          ← ⛔ THE RET-SPEC IS SPLIT ACROSS TWO LINES
      (:wat::i64::+ acc x))
```

The builder ruled, annotating his own example: **`:- V ;; ret-spec get its own line`** — *the
ret-spec*, one thing, one line. R11 split it, because `[args]` is a compound, so every child after it
breaks, and `->` and its type are two separate children.

**And `assoc-ride.wat` shows the same class on a TYPE:**

```
  [m <- (:wat::core::HashMap :-
          [:wat::core::i64 :wat::core::i64])
```

A type application torn across two lines.

### ★ THE CAUSE IS A SENTENCE I WROTE IN THE DESIGN, AND IT IS FALSE

> *"⚠ **R11 needs no notion of a SLOT.** Slots are a property of forms that have a GRAMMAR, and those
> forms have specific rules. The default rule sees an unruled form: head, then children."*

The first half is right. **The second half is the error: `fn` HAS a grammar and has NO RULE**, so R11
handles it — and R11 cannot see that `-> T` is one slot. Same for a type application `(T :- [args])`.

**The corpus is full of forms that have a grammar and no rule**, so this is not one fixture's
problem. `defn` is unscathed only because R1 exists and withholds the break from its ret type.

### WHAT THIS CHANGES

**R2 (`fn`) is no longer "just a file, later" — it is REQUIRED**, and it must withhold the break
between `->` and its type exactly as R1 does. Same for whatever owns a type application.

⚠ **And it raises a question that is the builder's, not mine:** every form with a grammar now needs a
rule *before* the default can be trusted on it. Either
- **(a)** each such form gets a rule (`fn`, type application, `defrecord`, `defenum`, `deftest`, …), or
- **(b)** the default learns slots from the registry's `@syntax` — which `src/intrinsic/mod.rs:3002`
  parses *through the substrate's own reader*, and which would make the default correct for every
  declared form at once.

**(b) is the larger prize and I have not measured whether `fn`'s row carries a usable `@syntax`.**
That measurement is the next thing, and it decides how many rule files this arc needs.

## Not disputed

Part 1 was green before part 2 began (STOP-2). `rete::or` is genuinely not legal in a `:where`
(*"where expr is not pure"*) — four kind-rules instead of one disjunction is the honest workaround,
reported not hidden. The blank-line trigger reads structure, not whitespace, which is why row 10
holds. Match arms with a compound body now break — real coverage from a more active default, and
consistent with the exploded ruling; the builder has not ruled that shape and may want to.
