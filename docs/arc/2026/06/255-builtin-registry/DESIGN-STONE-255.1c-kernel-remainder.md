# DESIGN STONE — 255.1c-kernel-remainder · HOME #8: the tier's last thirteen, and the blanket-accept gets a NAME

## The population — everything left under `:wat::kernel::`

```
runtime.rs:4321  serve-dispatch-op      4321 + 5640, TWO arms
runtime.rs:5662  retag-op
runtime.rs:6741  call-site              :6742 macro-call-site
runtime.rs:6773  assertion-failed!      :6777 raise!      :6779 here      :6783 fn-forms
runtime.rs:6810  peer-process           :6816 peer-wire?  :6819 address-wire?
runtime.rs:6821  require-wire-address   :6832 peer-pid
```

After this the `:wat::kernel::` literal dispatch is **empty** and the tier is wholly registered.

## ★★ THE HEADLINE — `peer-pid` IS INVISIBLE TO THE TYPE CHECKER, AND IT HAS 18 CALL SITES

Measured, with the pattern positive-controlled after four narrow-pattern errors the same day:

```
peer-pid   corpus call sites: 18      mentions in src/check.rs: 0
```

No registered `TypeScheme`, no bespoke `infer_list` arm — **nothing**. It falls through to
`check.rs:5561`:

> `// HARVEST (236.2): silent-by-intent — no scheme found for multi-arg form; accept and pass.`
> `return … CheckResult::ok(fresh.fresh())`

Its arguments are unchecked, its arity is unchecked, and its return type is a **fresh type variable**.
An eighteen-site production verb that the checker knows nothing about.

**This is task #110 — arc 255's FOUNDING HOLE — with a name attached for the first time.** The
blanket-accept has been described as a structural risk; here it is as a specific live verb. And 255's
own road says `255.1b-iv` must precede the flip precisely because *an unregistered `:wat::` head
type-checks clean*, so a mass rename would ship half-broken and silent.

Registering `peer-pid` does not close #110 — the door stays open for everything else — but it takes
one verb out of the blanket's shadow and **proves the hole is not theoretical.**

## ★ Three rulings the taxonomy explicitly DEFERRED TO THE CARVE — this is the carve

1. **`serve-dispatch-op`.** `255.1c-taxonomy` recorded: *"The ward found no clean single-axis fit
   (dispatch + a crash-sentinel broadcast). Recommend `:ControlFlow` with the broadcast noted as
   defensive plumbing — the CARVE rules it, not this stone."* ⚠ It has **TWO arms** (`4321` and
   `5640`); both must go.
2. **`:ControlFlow`'s prose for `raise!`/`assertion-failed!`.** Same stone: *"They never return; they
   abandon evaluation rather than direct it. The ward accepted the fit and asked the prose be
   strengthened to say so. That is a one-line prose edit the CARVE makes when it files them."*
3. **`require-wire-address` is `:CheckGate`'s only member — and its prose currently LIES.**
   `intueri` found this and it was independently verified: the variant's text asserts *"One member
   today"* naming this verb, which **is not registered and carries no `@Category`**, so actual
   membership is **zero**. A claim shipping as `///` API documentation that the disk contradicts.
   **Carving it makes the sentence true for the first time.**

## The axis prediction — the rider RE-DERIVES all thirteen

| verb | predicted Category | note |
|---|---|---|
| `raise!` `assertion-failed!` | ControlFlow | + the prose strengthening above |
| `here` `call-site` `macro-call-site` `fn-forms` | Reflection | *the program interrogating ITSELF* |
| `require-wire-address` | CheckGate | its first real member |
| `peer-wire?` `address-wire?` | **Probe?** | ⚠ would be the FIRST tenants of a zero-tenant variant |
| `peer-pid` `peer-process` | **Projection?** | a component of a peer the caller holds |
| `serve-dispatch-op` `retag-op` | **UNRULED** | derive from the bodies; see above |

★ **`:Probe` and `:Projection` are the interesting rows.** `:Probe` has **zero tenants** — it was
minted with no members, and `:Probe`'s prose warns off the trap that killed `:Predicate`:
*"NOT 'returns a bool': `length` returns an i64 and belongs here. Sorting by return type is the
axis-mix that sank the proposed `:Predicate`."* So `peer-wire?` must NOT be filed as `:Probe` merely
because it ends in `?`. Derive from what it DOES.

And the `peer-pid`/`peer-process` pair tests `:Projection` against a **handle** rather than a record —
does "returns a component that was already there" hold when the whole is an opaque peer?

## ★ The classification-failure hunt continues — the builder's standing method

> *"we continue with the names we have as seek failures to classify as we move forward."*

Thirteen bodies, and two of them (`serve-dispatch-op`, `retag-op`) the taxonomy already declined to
rule. **A verb that will not classify is the deliverable**, exactly as in home #7 — where the strain
report refuted two of the orchestrator's own four candidates from the bodies.

## The adoption audit, done at draw time — and it found ONE candidate, correctly rejected

```
assertion-failed! 1266 · fn-forms 42 · call-site 24 · peer-pid 18 · retag-op 6
peer-process 5 · serve-dispatch-op 4 · here 2 · macro-call-site 2
require-wire-address 2 · peer-wire? 1 · raise! 1 · address-wire? 0
```

**Twelve of thirteen are adopted.** `address-wire?` is the only zero — and it **fails the `drop`
test**: `Address` has a codec in `capability/registry.rs`, arriving through the capability waist, and
its sibling `require-wire-address` is live in `wat/bracket.wat:903,986`. So an Address genuinely flows
through wat programs: `address-wire?` is **exercisable and unexercised** — the `close` case, not the
`drop` case. **Inventory entry, NOT a retirement.** `[[feedback_no_consumers_does_not_mean_dead]]`

The discriminator has now earned its keep twice in a row: it killed `drop` and it stopped this one.

## Blast radius

```
NEW   src/intrinsic/kernel_remainder.rs   (name the rider's call if a better one fits)
EDIT  src/intrinsic/mod.rs                one `mod` line
EDIT  src/runtime.rs                      delete 14 arms (serve-dispatch-op has TWO)
EDIT  wat/runtime-meta.wat                :ControlFlow's prose + :CheckGate's false membership claim
```

No `check.rs`. **No stub schemes** — the standing rejection since home #5.

## ⚠ Standing orchestrator step — the goldens, NOW EIGHT NOT FIVE

The census I used for four consecutive stones was short by three. **The real census is
runtime.rs ×5, check.rs ×2, freeze.rs ×1** — and this stone edits `runtime.rs` only, so the five
should fire and the other three should not. **That is a prediction, and the floor falsifies it.**

Procedure: `git diff --numstat`, confirm which hunks precede each pinned site (home #7 proved the
delta is NOT always uniform — one of its three hunks sat below the pinned line and applying the net
would have been wrong), confirm `:col` unchanged, bump, verify by floor.

## Progress meter

86 → 99 registered forms, and `:wat::kernel::` literal dispatch reaches **zero**. The tier that
opened as *"not a family — a TIER braiding seven concerns across 49 arms"* closes as eight homes.

---

## ⊘ AMENDED MID-STRIKE — `kernel_remainder.rs` IS A BAD NAME, AND THE NAME WAS THE SYMPTOM

**Builder:** *"uhm.... this is an awful name..."* — of `src/intrinsic/kernel_remainder.rs`, which
**this design supplied as the default** and the rider took.

It names the file by **what was left over**. That is the `utils`/`helpers`/`misc` mumble `intueri`
names outright: *"The file tree should mirror the domain. When you `ls src/`, you should see the
architecture — not `utils`, `helpers`, `common`, `misc`."* Written by the orchestrator, in a brief,
the same day that ward was cast.

**And the name was diagnosing the grouping, not just labelling it.** The rider's thirteen rows landed
on **SIX** categories:

```
Reflection 4 · ControlFlow 3 · Probe 2 · Projection 2 · CheckGate 1 · Transform 1
```

Home #6 ruled that a HOME is a code-organization unit and a CATEGORY is a per-row label, so a home may
honestly hold two. **Six is not a home — it is a bucket**, and the only thing its members share is
"not yet carved". The bad name was the accurate one.

### THE SPLIT — four homes, names ruled by the builder

```
kernel_abort.rs     raise! · assertion-failed!                        verbs that NEVER RETURN
kernel_source.rs    here · call-site · macro-call-site · fn-forms     the program reading its own
                                                                      source position and form
kernel_identity.rs  peer-pid · peer-process · peer-wire? ·            asking a peer/address WHAT IT IS
                    address-wire? · require-wire-address              — projected, probed, and gated
kernel_serve.rs     serve-dispatch-op · retag-op                      service-dispatch plumbing
```

⚠ `kernel_reflect.rs` was considered and rejected — it collides with the existing
`src/intrinsic/reflect.rs`, and these four are narrower than reflection anyway: source position and
form, not arbitrary self-interrogation.

★ `kernel_identity.rs` is the interesting one and it VINDICATES home #6's distinction: one subject
(*what is this peer or address*) across three categories — `:Projection` projects the pid,
`:Probe` asks whether it is a wire, `:CheckGate` refuses a call site that lacks a wire address. A
home named for its subject; categories per row.

**Builder's ruling, verbatim:** *"those names are fine,, 109 owes a massive renaming, we'll address
any misnames then, those are fine for now"* — so these ship, and **arc 109 owns the sweep** if any
of them reads wrong later.

### Sequencing

The rider's thirteen rows are CORRECT and fully reusable — only the partition changes, not a single
`@Category`, `@Purity` or body. It finishes; the split is a `git mv`-shaped follow-on.

**Progress meter, corrected:** the tier closes as **eleven homes**, not eight —
stdio · time · ambient · message · error · resource · abort · source · identity · serve · (bytes,
reflect, witness predate the carve).
