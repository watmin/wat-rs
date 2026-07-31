# DESIGN-STONE — `insert` joins the dual-impl: the wat form becomes the oracle, the native becomes the user path

> **The builder's ruling (2026-07-31):** *"we build correct but slow first, then we build the correct
> and fast against it — the oracles must be a beacon of correctness then optimize against them.
> Real wat-rete users should only use the rust native flavor; the wat interpreted path is just a
> demonstration of correctness."*
>
> `insert` never got its dual. It is the last hot verb where the interpreted form IS the user path.

## The measurement that grounds this

`wat-scripts/scratch-pad/probe-insert-cost-split.wat` (`157346ab`), three arms folding over the same
range and constructing the same record the same number of times — which is what makes the
subtractions valid:

```
n=20000        total-ms   µs/fact
baseline         39.02      1.95    fold + construct + read a field   (the interpreted harness FLOOR)
conj             34.95      1.75    floor + PersistentVector/conj
insert          270.81     13.54    floor + :wat::rete::insert

insert - conj = 235.8ms = 11.79 µs/fact  ->  87% of the per-fact cost is insert itself
insert / conj = 7.75x
```

R24's do-not says these seed harnesses are interpreted theatre, so the fold+construct floor could
have been most of the 15 µs — it is not. The floor is 1.95 µs and is if anything OVERSTATED (the
baseline arm runs first and eats the warmup; at n=5000 it reads 3.14 µs/fact against conj's 1.70).
The container is free. **`insert` is the cost.**

Why: `wat/rete.wat:833-844` is interpreted wat performing **7 `Session` accessors + a 7-field
`Session` reconstruction per fact** — and its own header says the reconstruction exists because
`Record/assoc` returns the base `:wat::core::Record` type and the checker needs the concrete one.
**A type-system workaround, paid 20,000 times.**

Seeding is 74% of a real `accum` workload (306ms of 412ms at `[100 200]`) and ~66k facts/sec, which
is the gate on line rate (R25 `MACHINA CHAOS DOMAT`).

## The shape — an exact mirror of the `fire-rules` trio, not a new invention

```
fire-rules-spec   the wat ORACLE      wat/rete.wat:1819   pure wat, correct-but-slow
fire-rules'       the native kernel   runtime.rs:4706     dispatched to eval_fire_rules_native
fire-rules        THE PUBLIC VERB     wat/rete.wat:1838   one-line delegate to the prime
```

`runtime.rs:4706` states the convention as law: *"rete dual-impl: unprimed is the wat ORACLE, primed
the native kernel; never collapsed."* So:

| | today | after |
|---|---|---|
| the wat form | `insert` (the user path) | **`insert-spec`** — the oracle, semi-hidden, unchanged logic |
| the native | — | **`insert'`** — Rust, dispatched in `runtime.rs` |
| the user path | `insert` | **`insert`** — a one-line wat delegate to `insert'` |

**Call-site churn: zero.** `insert` keeps its name, arity and signature; every existing caller is
already calling the public verb. The rename is `insert` → `insert-spec` for the *body*, and the
public `insert` is re-authored as the delegate.

## ★ THE ONE CONTRACT DECISION

**`insert'` resolves the `facts` field BY NAME, never by positional index.**

A `Session` record value is `class_fqdn` + positional fields, with the names living in the
`RecordDef` in the TypeEnv (`runtime.rs:5700-5730`). Today `facts` happens to be index 5 of 7.
Hardcoding 5 would make a future field reorder silently write the wrong slot — a wrong answer, not a
compile error, and the kind of thing the differential might not catch if the reordered field has a
compatible type. Resolve `facts` from the record's field names and fail loudly if it is absent.

Everything else is a structural clone: the other six fields carry through untouched, and `:facts`
becomes the conj'd `PersistentVector`. `insert` performs **zero activation** (`rete.wat:828-830` —
the WM stays open until `fire-rules`), so no memory is touched and no network is walked.

## The RED gate — a differential, per the ruling

The oracle is the beacon; the native is checked against it. The gate asserts:

1. **Single insert:** `insert'` applied to a compiled Session and a fact yields a Session
   structurally equal to `insert-spec`'s on the same inputs.
2. **Repeated insert:** folding N facts through `insert'` equals folding the same N through
   `insert-spec` — same `:facts` vector, same order, all six other fields untouched.
3. **The public verb delegates:** `insert` and `insert'` agree (so the delegate is real, not a
   second implementation that could drift).

Today this is RED at the runtime — `insert'` does not exist, so the call raises `UnknownFunction`.
(Per `[[reference_check_is_not_a_complete_red_arbiter]]`, `--check` alone would NOT catch an unknown
callee; the runtime is the arbiter for this gap, so the gate must RUN, not merely type-check.)

**Perf is measured, not gated.** `probe-insert-cost-split.wat` is the instrument, run before and
after. A wall-clock gate here would be flaky and — per 2026-07-30 — can pass for reasons unrelated
to the mechanism. The reachable target is the floor the probe established: **~1.75–1.95 µs/fact**,
i.e. **~7×** on seed, taking `[100 200]` seeding from 306ms to roughly 45ms and insertion from 74%
of that workload to about 25%.

## Blast radius

- `wat/rete.wat` — rename the body to `insert-spec`; re-author `insert` as the delegate.
- `src/runtime.rs` — one dispatch arm for `:wat::rete::insert'`.
- `src/rete/` — the native `eval_insert_native`.
- A differential test.

No corpus migration, no codemod, no call-site churn.

## Out of scope = REJECTED (affirmative cuts)

- **A bulk `insert-all` verb.** Once each insert is near the floor, a bulk loop is *already* near the
  floor — bulk would only save the last few Session rebuilds. It is a second-order win with its own
  API surface, and it does not help the streaming arrival pattern the chaos engine actually has
  (facts one at a time). If a measurement later demands it, that is its own stone with its own number.
- **Touching activation.** `insert` stages; `fire-rules` activates. That split is the design
  (`rete.wat:828-830`) and this stone does not move it.
- **Retiring the wat form.** Explicitly forbidden by the ruling — it is the beacon.
- **The `accum` residual fire term.** The last `:clara` point, a separate diagnosis.

## Sequencing

1. Land the RED differential gate (fails at the runtime: `insert'` unknown).
2. `insert-spec` rename + `insert` delegate + native `insert'` + dispatch arm.
3. Weigh: the differential, the full `--release` floor, clippy — by my own re-run.
4. Re-run `probe-insert-cost-split.wat` and the `accum` phase split; record before/after in the seam.
