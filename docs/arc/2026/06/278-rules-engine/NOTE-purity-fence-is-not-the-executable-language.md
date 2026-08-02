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

### Not yet checked, and it is the obvious next question

`cond` is one macro. **The corpus has not established whether every other macro reachable from a
`where` fails the same way** — the reproduction covers `cond` specifically. If the class is "any
macro head," the hidden-failure surface is much larger than one verb, and enumerating it is the
cheap next probe.
