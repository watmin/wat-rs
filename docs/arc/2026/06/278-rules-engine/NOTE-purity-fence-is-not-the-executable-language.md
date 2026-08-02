# NOTE — the purity fence's accepted language ≠ the `where` evaluator's executable language, and they differ in BOTH directions

**Found 2026-08-01 by the `where-control` corpus family; both instances reproduced by my own hand,
not relayed.** These are the first two STOP-1s the corpus has produced across eight families and 88
rows — which is exactly what R62 predicted: the rejection column is the half a peer-oracle cannot
bound, and it stayed empty until a family went looking at the language's control forms.

The two instances look alike under the brief's single "STOP-1" label. They are different animals, and
the difference is the finding.

---

## Instance 1 — `cond`: the fence says YES and the engine says NO. A HIDDEN FAILURE.

```clojure
(:wat::rete::where (:wat::core::cond ((:wat::core::= ?a 0) true) (:else false)))
```

Reproduced (`wat-scripts/scratch-pad/probe-where-cond-fence-execution-split.wat`):

| gate | verdict |
|---|---|
| `--check` | **CLEAN** |
| the same logic spelled with `if` (control) | fires, derives `n=5` |
| `:wat::rete::compile` + the purity fence | **PASSES** |
| `fire-rules` | **RAISES** `#wat.runtime/UnknownFunction {:path ":wat::core::cond"}` |

**Root** (as reported, and consistent with the reproduction): `cond` is a `defmacro`, not a runtime
primitive like `if`/`let`/`match` which dispatch in `eval_inner`. A `where` is captured as DATA by
`quasiquote` and evaluated later by `eval-test`, which never macro-expands. `classify_expr` has a
clause-aware `cond` arm that structurally approves it — so the fence reasons about a form the
evaluator cannot execute.

**Why this is the serious one.** A capability boundary is honest: the checker refuses the form, at the
site, before you ship. This is the inverse — every static gate passes and the failure arrives at
*first fire*, in whatever process happened to run the rule. That is the silhouette arc 278's law
forbids (R55 `REVOLVTIONE, NVLLA LARVA`), and it is R57 recurring: *a law is completed by USE, not by
declaration.* The fence was declared complete; a real consumer walked into the gap.

It is also `NON MVRVS SED VITIVM` (R24) inverted — there a "wall" turned out to be a flaw; here a
**pass** turns out to be one.

The control row matters: the identical branching logic spelled with `if` compiles and fires correctly.
The wall is `cond`'s macro-ness, not branching.

---

## Instance 2 — `Some`/`None`/`Ok`/`Err`: the fence says NO to something genuinely pure. CAPABILITY LOST.

```clojure
(:wat::rete::where (:wat::core::= 1 (:wat::core::Option/unwrap-or (:wat::core::Some ?k) 0)))
```

Reproduced: `--check` clean, then `:wat::rete::compile` panics — **located, at compile time, honestly**:

```
compile-condition: where expr must be pure and deterministic   (wat/rete.wat:566)
```

**Root** (reported, plausible and consistent with the reproduction — the Rust walk is NOT re-read by
me): `constructor_meta` derives purity from the frozen `TypeEnv`, which covers every user
`defrecord`/`defenum`; but `Option`/`Result` are checker-special-cased builtins never registered
there, so they fall through to `intrinsic_meta`, which does not list them either.

**Why this one is NOT a hidden failure.** The wall fires, at compile time, with a location. It is
working — it is just *wrong on the merits*: `Some`/`None`/`Ok`/`Err` are total, non-raising, pure
constructors. This is an over-rejection from a registry gap, not a considered design boundary.

Note the asymmetry the rider isolated: **reading and `match`-ing** a bound `Option`/`Result` field is
completely fine (`where-control` row 7 and `where-record` row 11 both land on it). Only *constructing*
one anywhere reachable from a `where` is rejected.

---

## ★ The finding, which is neither instance alone

**The purity fence's accepted language and the `where` evaluator's executable language are two
different sets, and they differ in BOTH directions:**

- the fence **admits** what the evaluator cannot run (`cond`, and presumably every other macro) —
  a hidden failure;
- the fence **refuses** what the evaluator could run perfectly well (`Some`) — a lost capability.

### The consequence for #49a, and it is the reason to record this now

The obvious way to specify a compiled-`where` executor is *"it must handle whatever the purity fence
accepts."* **That specification is wrong in both directions.** It is too large (it includes `cond`,
which no evaluator can run today) and too small (it excludes total pure constructors). A compiler
built against the fence would inherit both errors.

This is R62's own thesis arriving as a concrete bill: the corpus's *green* rows tell you what agrees
with Clara; the corpus's *rejections* tell you where our own boundary actually is — and the boundary
turns out not to be where the fence draws it.

### The dispositions differ, and should not be bundled

- **`cond` (and macros generally)** — either the fence learns to REJECT a macro head in a `where`
  at compile time, located; or the `where` capture macro-expands so `cond` genuinely works. Either
  closes it. The RED gate is on disk
  (`probe-where-cond-fence-execution-split.wat`) and is expected to raise until one of them lands.
  **Do not close it by deleting the probe.**
- **`Option`/`Result` constructors** — a registry gap: register their purity metadata where every
  other constructor's lives. Arc **255** (`pure?`/`deterministic?` metadata) is the plausible home;
  this note does not rule that, and the Rust walk (`constructor_meta` / `intrinsic_meta`) has NOT been
  re-read by me — ground it before drawing the stone.

---

## ⚠ ESCALATION (same day, `where-collection`): it is NOT just macros — and the fence does not check TYPES or TOTALITY either

The section above asked whether `cond` was one verb or a class. **It is a class, and a wider one than
"macros."** Three further instances, all reported with reproductions:

| form | fence | first fire |
|---|---|---|
| `(length (map f ?coll))` | PASSES (both verbs individually pure) | **raises `TypeMismatch`** — `length` does not take a `Stream` |
| `(> (first ?t) 0)` where some `?t` is `[]` | PASSES (`first` unconditionally classified pure) | **raises `MalformedForm`, kills the fire** |
| 2-arity `reduce` (no seed) | PASSES | identical hazard — `wat/seq.wat:207` bottoms out in `(first coll)` |

So the fence is a **syntactic purity walk**. It does not check:

- **macro-vs-primitive** (`cond` — the original instance),
- **types in composition** (each verb pure, the pipeline ill-typed),
- **totality** (`first` is partial; the fence treats it as total).

### The `first`-on-empty case is the worst of all of them, and it is a different kind of bad

The others fail on the first fire, every time, for everyone — loud and immediate. This one is
**DATA-DEPENDENT**: the rule compiles, fires correctly, and keeps firing correctly until a fact
carrying an empty collection arrives — and then it kills the whole fire. A predicate that has worked
in production for a month can be detonated by one empty vector.

### This is the missing `total?` axis, and the record designed it a month ago

`NOTE-overlay-read-path-and-distributed-horizon.md` Part 5 (2026-06-20) already ruled the fix, in
detail: **`total?` as a THIRD axis on the 6a fence** (`pure? ∧ deterministic? ∧ total?`), the same
structural walk, rejecting at registration what it cannot prove terminates — eBPF's road, explicitly
chosen over runtime fuel. It was designed and never built.

It was argued there as a **multi-tenant resource-safety** requirement, for running strangers' code.
That framing understated it. `first`-on-empty shows the missing axis is not only a defence against a
hostile tenant — it is a live correctness hole for **our own first-party rules**, reachable today, in
the form the corpus says users will write.

### And the over-rejection direction widened too

`nth` and `filter`-then-`into` are also refused (both bottom out in deliberately-unclassified verbs —
`Option/expect` and `rest`). Meanwhile `:wat::core::every?` and `:wat::core::some?` **do not exist at
all** — not a fence rejection, an absence; the corpus emulates them with `foldl`.

**The one genuinely good news:** a real higher-order verb IS admitted. `foldl`/`reduce` at 3-arity
(explicit seed) closing over a user `fn`, applied to a bound collection, composes through the fence
and runs correctly — including safely on empty input, because the seed makes it total. Four corpus
rows exercise it. That is the shape that works, and it works *because* it is total.

### The count now

Five instances, in two directions, from two families that went looking. The specification for #49a
cannot be "whatever the purity fence accepts" — the fence is neither sound (it admits the
unrunnable) nor complete (it refuses the pure), and the unsoundness includes a data-dependent case
that no amount of green testing will reliably surface.
