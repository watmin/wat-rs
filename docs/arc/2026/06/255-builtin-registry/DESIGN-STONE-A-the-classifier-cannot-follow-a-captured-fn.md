# DESIGN — STONE A: the axis classifier cannot follow a captured fn value

> **Builder, 2026-08-30:** *"do C now… and draw A as the next stone."*
>
> **Drawn, not scoped.** The mechanism is measured and the candidate shapes are laid out with the
> four questions on each. **The fork is the builder's** — this document does not pick one, because
> the three differ on what the language *promises*, not on cost.

## The mechanism — measured, not argued

`wat-scripts/scratch-pad/255-probe-the-classifier-cannot-see-through-a-closure.wat`, run against the
substrate's own `:wat::rete::pure?` (the same `classify_expr` walk `src/freeze.rs:803` uses to impose
pure ∧ deterministic ∧ total on sigma fns):

```
(fn [a b] (< a b))                       -> true     sort/1's comparator
(fn [a b] (< (keyfn a) (keyfn b)))       -> FALSE    sort-by's comparator
(fn [a b] (do (println …) (< a b)))      -> false    an effectful comparator
```

Row 2 is the finding. `wat/core.wat:1537,1546` build `sort-by`'s comparator around the **free
variable** `keyfn`, bound to a caller-supplied fn **value** at runtime.

```rust
fn classify_expr(ast: &WatAST, axes: &[Axis], sym: &SymbolTable, seen: &mut HashSet<String>)
```

`head_ok` resolves a head as: data constructor → `sym.functions` (top-level, recursive) →
`intrinsic_meta` → **default-deny** (`src/rete/purity.rs:920`). A local binding holding a closure is
in none of those, so it default-denies. **The classifier is AST-structural and has no environment.**

★ **`src/freeze.rs:803` carries the identical blind spot.** It has never bitten only because sigma
fns are closed arithmetic. This — not `effectful_by_prefix` — is the real reason the W7 HOF family
is hard. `NOTE-the-prefix-guess-does-not-scale-to-a-mixed-namespace.md` named the symptom; this is
the mechanism.

**What it blocks:** `map · mapv · filter · foldl · sort$native` — every verb that runs code it did
not write. Five verbs, one cause.

## Two facts that bound the design

1. **The information EXISTS at runtime.** `Function` (`src/value/environment.rs:48`) carries
   `closed_env`: *"`fn` values have `name = None` and carry their `closed_env` from the creation
   site."* A captured `keyfn` is reachable from the comparator value.
2. **A per-arg declaration surface already exists.** `@yields <argname> <desc>` (arc 255 Stone P5-b)
   names *which argument is a callback*, carries a SUBJECT, is repeatable, and is already declared
   on five HOF intrinsics (`resource.rs:339,340,394`; `hologram.rs:54`; `witness.rs:66`).

## The fork

### A-1 — declare the contract; enforce it STATICALLY at the caller's call site

The HOF declares a purity requirement on its callback arg (a new directive, or an axis clause on
`@yields`); the registry carries it as an entry field; the checker classifies the **argument's AST
at the call site**, where a literal or a named fn is visible.

- **Obvious? YES** — the error lands on the user's own `keyfn`, naming their function.
- **Simple? YES** at the intrinsic's own door; the classifier is unchanged.
- **Honest? YES** where it applies — a static refusal on a form the walker can actually see.
- **Good UX? YES** — a check-time error, not a runtime one.
- ⛔ **THE UNMEASURED PART, and it is load-bearing:** `sort` and `sort-by` are wat **`defclause`s**,
  not registered intrinsics. A requirement on `sort$native`'s argument must **propagate outward**
  through the defclause to *its* callers, because inside `core.wat` the argument is the free
  variable `keyfn`. **Contract propagation through a wat defclause is not measured and may be the
  whole stone.** Nobody should cost this shape until that is probed.

### A-2 — give the classifier an environment; resolve the capture at RUNTIME

Add an `Option<&Environment>` to `classify_expr`/`head_ok`. On an unknown head, look it up in the
supplied environment (and in a `Function`'s `closed_env`) and recurse into its body if it resolves
to a `Function`. Absent an environment, behaviour is exactly today's — default-deny.

- **Obvious? YES** — "follow the value you actually hold" is the direct reading of the defect.
- **Simple? YES**, mechanically: `classify_expr` has **19 callers**, `head_ok` 2,
  `find_axis_violation` 2 — an `Option` parameter keeps every existing caller's behaviour identical.
- **Honest? YES** — and it is the only shape that closes the hole for the case that motivated this:
  a user's impure `keyfn` smuggled in through `sort-by`. It also fixes `freeze.rs`'s identical blind
  spot in the same motion.
- **Good UX? YES**, with one caveat to design in: the check runs **once per call, at the door, on
  the fixed comparator** — never per comparison — so the cost is one classifier walk per `sort`, not
  O(n log n) of them. A runtime refusal is later than a check-time one, which is the real price.
- ⚠ The classifier stops being purely static. That is a genuine change in what the thing IS, and it
  is the reason this is the builder's call and not mine.

### A-3 — impose nothing; let the registry record the truth and the fence refuse

Declare the HOFs honestly (`Effectful`, or `Unreviewed`), accept that they stay out of the four-axis
`where` fence, and close the `effectful_by_prefix` question separately.

- **Obvious? YES · Simple? YES · Honest? YES** — it claims nothing it cannot verify.
- ⛔ **Good UX? NO** — it concedes the ground. Five verbs stay unhomeable-as-pure, the fence stays
  shut to them, and a user can still make `sort` effectful with no diagnostic anywhere. It is the
  status quo with better paperwork.

## What this stone must NOT do

- ⛔ **Do not widen `:wat::core::` in `effectful_by_prefix`.** It would make the guess vacuous for
  the largest namespace in the language (that NOTE's option 1, and it is still wrong).
- ⛔ **Do not declare a HOF `Preserving` to sidestep this.** `Preserving` is documented in
  `wat/runtime-meta.wat:44` as *"a special form that PRESERVES the purity of its sub-forms"*, and its
  only two users are the two special forms. For `if`, the fence walks into the branches; for a HOF
  the argument is a runtime value nothing can inspect. It would assert a conditional purity **nothing
  can verify** — the hole intact, wearing a label that sounds checked.

## The cheap probe whoever draws this should run FIRST

Before designing: take A-1's unmeasured half on its own. Declare a purity requirement on ONE
intrinsic's callback arg and ask whether a `defclause` between the caller and that intrinsic can
carry it. If contract propagation through a wat defclause turns out to be cheap, A-1 wins on
Good UX (check-time beats runtime). **If it does not, A-2 is the only shape that closes the hole**,
and the fork collapses to a ruling about whether the classifier may hold an environment.
