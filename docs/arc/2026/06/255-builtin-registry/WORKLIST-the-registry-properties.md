# WORKLIST — the properties the registry still has to grow. As of 2026-08-30.

> **Builder, 2026-08-30:** *"what properties does the registry need to grow for this? … get these
> noted in our build list — do not forget them."*
>
> Companion to `WORKLIST-the-44-unhomed.md`. That one tracks which **verbs** have no home; this one
> tracks which **properties** have no home. Both retire the same way: a row goes when the thing it
> names exists. **Replaced in place, never appended.**

## The LOCKED RECORD MODEL promised EIGHT Layer-1 fields. Three were never built.

`DESIGN.md:399`, locked 2026-06-21:

```
name · arity · kind · pure · deterministic · expand_time_legal · defined_in · layer
                                             ^^^^^^^^^^^^^^^^^   ^^^^^^^^^^   ^^^^^
```

Measured against `IntrinsicEntry` (`src/intrinsic/mod.rs`) on 2026-08-30:

| field | wat `defenum` | Rust type | on the entry | cost |
|---|---|---|---|---|
| `expand_time_legal` | ✗ | ✗ | ✗ | mint + declare + 431 answers |
| `defined_in` | **✓ exists** | ✗ | ✗ | generate + carry; **auto-derived** |
| `layer` | **✓ exists** | ✗ | ✗ | generate + carry; **auto-derived** |

---

## ⬜ 1. `defined_in` · `layer` — cheap, but ⛔ **DO NOT BUILD YET: it would be a CONSTANT**

**`:wat::runtime::DefinedIn` (`:Wat | :Rust`) and `:wat::runtime::Layer`
(`:Substrate | :Userland`) ALREADY EXIST in `wat/runtime-meta.wat`** — minted for
`metadata-of`'s reflection surface and never promoted to entry fields.

★ **These are NOT 431 declarations to write.** The locked model says so explicitly:

> *"Provenance (`defined_in`/`layer`) **AUTO-DERIVED at the registration site** — a wat form can't
> claim `:rust`; **the tag can't lie**."*

So the honesty is structural, not declared: the macro knows whether it is expanding a `#[wat_intrinsic]`
(⇒ `Rust`/`Substrate`) or a wat form, and computes the value. No verb answers anything.

**What it needs:** two `wat_enum_from!` generations, two entry fields, two derivations in the macro.
No corpus migration, no per-verb judgement.

## ⛔ AND WHY IT MUST WAIT — measured 2026-08-30, correcting this file's own first draft

Everything that can enter the registry today arrives through `#[wat_intrinsic]` (429) or
`#[wat_special_form]` (4). **Both are Rust attributes on Rust functions.** So every one of the 433
entries would carry `Rust` / `Substrate`:

```
#[wat_intrinsic]     429  ->  Rust / Substrate
#[wat_special_form]    4  ->  Rust / Substrate
wat-defined            0
```

**A field with one value across the entire population discriminates nothing, cannot be wrong, and
teaches no consumer anything.** Building it now is justified only by *"we will need it when wat
forms register"* — which is future-vapor by this project's own standard, and precisely what
`exigere` exists to drive out. **Cheap to build is not the same as worth building.**

★ **The unblocking condition is explicit:** build these when a *second kind* can enter the registry
— when a wat `defn`/`defclause` registers, so `DefinedIn` has a `Wat` to discriminate. Until then
the crate-split question ("what is substrate, what is userland?") has a constant answer and does not
need a field to hold it.

⚠ This entry originally read *"the cheapest thing on the board"* and recommended doing it first.
That was the orchestrator's, and it was wrong on the axis that matters: cost, not value.

---

## ✅ 2. `expand_time_legal` — GROWN 2026-08-30 (expand-T1 → T4b). Left below as the record.

`src/macros/eval.rs`'s `is_pure_total` — **202 names, 411 lines** — is the last large hand-list
holding a property, and **it is misnamed**: it does not measure `pure ∧ total`, it measures *"is
this legal inside a `defmacro` body at expand time."*

★ **MEASURED 2026-08-30, and NOT derivable from the existing axes:**

```
LISTED               143      (of the registered population)
PURE_AND_DET         313
BOTH                 139
LISTED_BUT_NOT_PD      4      ← the proof of independence
PD_BUT_NOT_LISTED    174      ← the invisible gap
```

**The 4 exceptions are principled, not noise:** `core::fresh-symbol`, `kernel::macro-call-site`,
`hashmap::keys`, `hashmap::values` — all `@Determinism Nondeterministic` and all legal at expand
time. `fresh-symbol` is nondeterministic *by design* (it generates a unique symbol) and obviously
must work in a macro body. **A verb can be nondeterministic and expand-time-legal**, which is what
makes this an independent axis rather than a coarser view of purity.

⛔ **And 174 pure∧deterministic verbs are missing from the list.** Not a rounding error — the exact
defect `NOTE-the-registry-asserts-properties-nothing-verifies.md` predicted as INSTANCE 3:

> *"nothing had ever noticed, because a **false REFUSAL only surfaces if some macro body happens to
> call the verb.** Default-deny hides its own gaps: the list cannot tell 'deliberately excluded'
> from 'never added'."*

It predicted six. It is **174**.

**★ DONE.** `:wat::runtime::ExpandTime = Legal | RuntimeOnly | Preserving | Unreviewed`, minted in
`wat/runtime-meta.wat` (T1, door broken with `E0599`), declarable and carried (T2), REQUIRED (T3 —
431 sites answer), and `is_expand_time_legal` now DERIVES from it (T4b), leaving a 59-name homing
backlog. The function was also renamed from `is_pure_total`, which is what retired the
`:wat::i64::/` contradiction: it never measured totality.

⚠ **The 174-verb gap did NOT close and was never going to here.** 288 registered verbs read
`@ExpandTime Unreviewed`; ~174 are pure ∧ deterministic and probably legal. What changed is that
the gap is now **visible in the source, at each verb** — before, a false refusal only surfaced when
some macro body happened to call the verb. Closing it is a census with somewhere to write the
answer, and that is the next work on this axis.

★ **Renaming it also retires the `:wat::i64::/` contradiction** (`is_pure_total` lists it;
`intrinsic_meta` explicitly excludes it from `total`). They were never disagreeing about one
property — they were answering two questions under one name.

---

## ⬜ 3. `primitive?` — the fence's FOURTH axis, and it may not belong here

Arc 278's `where` fence is `pure ∧ deterministic ∧ total ∧ primitive?`. Three now come from the
registry. The fourth — `Axis::RetePrimitive` (`src/rete/purity.rs:148`) — is answered from
`RETE_OPS` / vocabulary membership, and is **in neither the locked model nor the entry**.

⚠ **Open question, not a work item:** vocabulary membership may genuinely be rete's to own — "is
this verb admitted to the rete `where` language" is a statement about rete's surface, not about the
verb. **Do not add it to the registry without a ruling.** Noted here so it is not forgotten, not so
it is done.

---

## ✅ Grown this session, for the record

`totality` — `:wat::runtime::Totality` (`Total | Partial | Preserving | Unreviewed`), minted T1,
declarable T2, carried T2b, **required** T3, and the fence derives it T4b. Not in the locked model:
arc 278 invented the axis after 255 locked, which is the collision recorded as INSTANCE 4-bis of
`NOTE-the-registry-asserts-properties-nothing-verifies.md`.

---

## Rules this list obeys

- ⛔ **A row retires when the property EXISTS ON THE ENTRY**, verified by reading
  `IntrinsicEntry` — not when a stone claiming it is committed.
- ⛔ **Row 3 is a QUESTION, not a task.** Building it without a ruling would put rete's surface
  vocabulary into a registry of language properties on nobody's authority.
- ⛔ **Replaced in place, never appended.**
