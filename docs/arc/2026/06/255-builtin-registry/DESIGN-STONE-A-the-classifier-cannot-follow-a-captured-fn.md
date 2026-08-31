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

⛔ **THE UNMEASURED PART, and it is load-bearing:** `sort` and `sort-by` are wat **`defclause`s**,
  not registered intrinsics. A requirement on `sort$native`'s argument must **propagate outward**
  through the defclause to *its* callers, because inside `core.wat` the argument is the free
  variable `keyfn`. **Contract propagation through a wat defclause is not measured and may be the
  whole stone.** Nobody should cost this shape until that is probed.

### A-2 — give the classifier an environment; resolve the capture at RUNTIME

Add an `Option<&Environment>` to `classify_expr`/`head_ok`. On an unknown head, look it up in the
supplied environment (and in a `Function`'s `closed_env`) and recurse into its body if it resolves
to a `Function`. Absent an environment, behaviour is exactly today's — default-deny.

The check runs **once per call, at the door, on the fixed comparator** — never per comparison — so
the cost is one classifier walk per `sort`, not O(n log n) of them. A runtime refusal is later than a
check-time one; that is the real price.

⚠ The classifier stops being purely static. That is a genuine change in what the thing IS, and it
  is the reason this is the builder's call and not mine.

### A-3 — impose nothing; let the registry record the truth and the fence refuse

Declare the HOFs honestly (`Effectful`, or `Unreviewed`), accept that they stay out of the four-axis
`where` fence, and close the `effectful_by_prefix` question separately.

It claims nothing it cannot verify — but it concedes the ground. Five verbs stay unhomeable-as-pure,
the fence stays shut to them, and a user can still make `sort` effectful with no diagnostic
anywhere. It is the status quo with better paperwork.

## What this stone must NOT do

- ⛔ **Do not widen `:wat::core::` in `effectful_by_prefix`.** It would make the guess vacuous for
  the largest namespace in the language (that NOTE's option 1, and it is still wrong).
- ⛔ **Do not declare a HOF `Preserving` to sidestep this.** `Preserving` is documented in
  `wat/runtime-meta.wat:44` as *"a special form that PRESERVES the purity of its sub-forms"*, and its
  only two users are the two special forms. For `if`, the fence walks into the branches; for a HOF
  the argument is a runtime value nothing can inspect. It would assert a conditional purity **nothing
  can verify** — the hole intact, wearing a label that sounds checked.

## THE FOUR QUESTIONS — flat YES/NO, every option

**Obvious + Simple + Honest must ALL hold before Good UX is weighed.** A `—` means the question was
never reached, because the option was already disqualified.

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **A-1** declare the contract; enforce statically at the call site | YES | **NO** | **NO** | — | ⛔ **DISQUALIFIED** |
| **A-2** the classifier may hold an environment; resolve the capture | YES | YES | YES | YES | ✅ **ADMITTED** |
| **A-3** impose nothing; record honestly, fence stays shut | YES | YES | YES | **NO** | ⛔ **DISQUALIFIED** |

**Why each NO — the answer is the finding, not an opinion:**

- **A-1 Simple? NO.** Three things, not one: a new per-parameter metadata key, new per-call-site
  enforcement, and a hand-declaration at *every link* in every HOF chain. `sort-by` must declare an
  obligation `sort$native` incurred.
- **A-1 Honest? NO.** The declaration asserts an obligation **nothing verifies against the body**.
  Forget one and the hole reopens with no diagnostic — a list that cannot tell *"deliberately
  omitted"* from *"never added."* This is the defect class the whole arc exists to kill.
- **A-3 Good UX? NO.** It passes the first three by claiming nothing — and serves no caller. A user
  can still make `sort` effectful with no diagnostic anywhere. ★ This is precisely what *"UX is the
  tiebreaker, not the load-bearing test"* is for: A-3 is **honest but useless**, and that is a real
  disqualification, not a quibble.

★ **A-2 is the only option that answers YES four times**, and it does so on measurements, not
preference.

## ★ THE PROBE RAN — 2026-08-30. THE FORK HAS COLLAPSED, AND NOT AS PREDICTED.

This section previously said: *"if contract propagation through a wat defclause turns out to be
cheap, A-1 wins on Good UX."* **That framing was wrong. Propagation is neither cheap nor expensive
— it DOES NOT EXIST**, and nothing in the substrate would derive it.

### What A-1 would actually cost — measured

| | measured | source |
|---|---|---|
| a fn TYPE carrying purity | ⛔ **none.** `TypeExpr::Fn { args, ret }` — two fields, no effect row | `src/types.rs` |
| a metadata-map on a `defclause` | ✅ **parsed and enforced** — two tests cover it | `src/check.rs:21772, :21803` |
| any defclause in the corpus using one | ⛔ **zero.** *"corpus carries no `:restricted-to` defclause today"* | `src/check.rs:21723` |
| `binding_metadata` granularity | ⛔ **per-BINDING**, and `:restricted-to` constrains the caller's FQDN prefix — a capability wall, not a property of a parameter | `src/check.rs:633, :1383` |

So the carrier exists, but the **key is new** (per-parameter, not per-binding), the **enforcement is
new**, and — decisively — **nothing derives the obligation from the body.** `sort-by` would
hand-declare `{:pure-params [keyfn]}` because a human noticed it closes over `keyfn` into
`sort$native`'s pure slot. Nothing checks that declaration against what the body actually does.

⛔ **That is the CONVENTION rung, and it is the exact failure this arc exists to kill:** a
hand-declaration that cannot tell *"deliberately omitted"* from *"never added"*, so a forgotten one
reopens the hole silently. It is `is_pure_total`'s 174-verb gap in a new costume.

### What A-2 actually costs — measured, and smaller than this document first said

| | measured |
|---|---|
| `classify_expr` call sites | **19 — ALL inside `src/rete/purity.rs`.** The signature change does not cross a module boundary |
| the capture is reachable | ✅ `Function.closed_env: Option<Environment>` (`src/value/environment.rs:4,46`) |
| the lookup exists | ✅ `Environment::lookup(&self, name, head_span) -> Option<TrackedValue>` (`:200`) |
| both consumers already hold what they need | ✅ `freeze.rs` has the `Function`; `sort$native`'s door has `env` **and** the comparator value |

### ⚠ A-2's ONE REAL HAZARD — name it in the stone, it is not blocking

`classify_fn` guards recursion with `seen: HashSet<String>` **keyed on the FQDN**, resolving through
`sym.get(fqdn)`. **An anonymous closure has `name: None` and is not in `sym`** — so it has no key,
and a naive "follow the capture" recursion has no back-edge guard. A-2 must supply its own: identity
on the `Arc<Function>`, or a depth bound. Cheap, but it must be designed, not discovered.

### Where that leaves the fork

**A-2 is contained to one file and closes the hole; A-1 is a new declaration surface whose
correctness rests on each author remembering to declare.** A-3 remains the honest concession.
The Good-UX argument that favoured A-1 (check-time beats runtime) survives — but it is now paid for
with a hand-declaration per link in every HOF chain, and this arc has spent the whole day proving
what that costs. **The ruling is the builder's; the measurement no longer supports A-1 being the
cheap one.**
