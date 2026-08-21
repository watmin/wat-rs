# SCORE — the thirteen schemes: the Persistent family is under the type checker

Rider: 3.3 min, one flight, no STOP triggered. Every row re-run by the orchestrator's own hand.

| # | what | result |
|---|---|---|
| 1 | ★★ **the gate** — every persistent verb, 7 args | ✅ **14 reject, 0 blanket-accepted** (was 0/13) |
| 2 | ★★ **a declared V is ENFORCED** | ✅ `(PersistentMap [String i64])` used as `String` → **REJECTED**. Was accepted an hour ago. |
| 3 | control: `V=String` used as `String` | ✅ still accepted |
| 4 | control: `HashMap` V=i64 | ✅ still rejected, unchanged |
| 5 | ★ ADDITIVE — bare annotations survive | ✅ bare `PersistentMap` and `PersistentVector` → `/length` both check |
| 6 | blast radius | ✅ `src/check.rs` only; no runtime, no `.wat` from the rider |
| 7 | floor | ✅ **4818/4818, 69.8s** (after one fix — see below) |
| 8 | clippy | ✅ 0 |
| 9 | rustfmt | ✅ HEAD=0, now=0 |
| 10 | goldens | ✅ **unmoved** — both `check.rs` insertions (~19858, ~20112) sit BELOW the pins at 13567/13585, and a hunk below a pin shifts nothing |

Row 1 is 14, not 13: `PersistentVector/concat` already had a scheme, which is why the rider anchored
its new block there.

## ★★ THE FIRST HERETIC SCREAMED — and it is the exact case 109 filed in July

The floor came back **4817/4818**, and the one red was not a golden:

```
:wat::core::PersistentMap/assoc: parameter #3 expects :wat::rete::RootJoinNode;
                                 got :wat::rete::ProductionNode
    tests/rete/probe_arc278_1a_data_model.wat:10 and :20
```

`109/NOTE-typed-literal-constructors.md` (2026-07-18) names **these two node types** as its worked
example, and records the workaround that made the site possible:

> *"`assoc`/`conj` into a **bare-empty** collection stays value-unconstrained and accepts
> heterogeneous elements — the engine's own idiom (`rete.wat:420` grows `Session.network` exactly
> this way)."*

**The schemes killed that workaround, which is precisely what they were for.** The site was legal only
because `PersistentMap/assoc` had no scheme; with one, `V` unifies to the first value's type and the
second is refused. This is the wall working, not a regression.

★ **And the cure was already on disk — shipped this morning by ①b.** Measured, with a control:

```
(PersistentMap [:wat::core::i64 :wat::core::Record])  then assoc A, assoc B  →  2      ✓
(PersistentMap)                     bare-empty, same  →  "expects :user::A; got :user::B"
```

Declaring the supertype is exactly the capability `109/NOTE-typed-literal-constructors.md` said had no
working form. The note that predicted this failure and the stone that fixes it are the same arc, four
weeks apart.

Fixed at the one failing site (single-site, so a hand edit; R21 governs MULTI-site structural
rewrites). Floor green.

## ⚠ 200+ LATENT SITES — recorded, not fixed

The bare-empty idiom appears **200+ times across 32 files**, concentrated in `wat/rete.wat` (110) and
`tests/rete/**` (~70):

```
wat/rete.wat 110 · probe_arc278_8i_accumulator_folds 34 · 4c_retraction 10 · 6b_eval_test 7
1a_data_model 6 · 4b_cascade 6 · 4a_production_fire 5 · 0c_persistent_parity 4 · sift_* 12 · …
```

**Only ONE broke**, because the rest happen to be homogeneous — every value at that site is currently
the same type. They are not safe; they are **lucky**. Any one of them gains a second type and it goes
red, now with a proper diagnostic instead of silence.

⚠ That is the honest reading and it should not be dressed up: this stone did not fix 200 sites, it
made 200 sites *capable of failing correctly*. The 238-bare-annotation strike (`②a`) is where they get
declared types; this stone is what makes those declarations mean something.

## Honest deltas

- **The rider reused `pv_of` instead of minting `persistentvector_of`.** An identical closure already
  existed at `check.rs:20099`, serving `PersistentVector/concat`. It found it, used it, and said so —
  the brief asked it to look rather than mint, and it looked.
- **All 13 template line numbers were correct.** Confirmed by matching surrounding code, not trusted.
- **It read all 13 runtime implementations against their twins** before transcribing — identical
  arities, argument roles and return shapes throughout. **No divergence, so no finding.** That is the
  STOP-2 check actually being performed rather than assumed.
- ⚠ **One citation of mine had drifted:** the NOTE cites `check.rs:5561` for the blanket-accept; it is
  now at **`5588`** (27 lines). The rider caught it and wrote the corrected line into its comment
  "at time of writing". `[[feedback_a_blocker_note_is_a_claim_with_a_date_on_it]]`
