# BRIEF — `get` becomes total by fallback; the `Option` arm is the fourth cure

**Ruled 2026-08-05 by the builder:**

> *"fallback — that's the UX — the item either is or isn't in the vec… if it isn't the result is
> undefined by nature… there's no meaningful value there so it mandates a user supplied value in
> such cases."*

That is cure 3's criterion stated exactly (`NOTE-the-four-cures-for-a-domain-hole.md`): the hole is
real, the substrate has **no meaningful value** to offer, so the caller must supply one.

Anchor `/home/watmin/work/holon/wat-rs/`; verify with `pwd`. Tree clean at HEAD `404ab6a1`.
Floor **`4356 / 4356 / 0 / 262`**, clippy clean, `check-where-shapes.sh` → `9 pair(s), 98 rows`,
63 rows, `RETE_MODULES` = 2.

---

## The shape

```clojure
(where (:wat::rete::core::i64::> (:wat::rete::core::PersistentVector/get v 0 :undefined -1) 5))
```

One line at the surface. **A match-and-branch underneath** — the builder's own framing. No `expect`,
no panic, no `nth`.

## PART A — the `Option` arm, and it is the FOURTH pattern

`dispatch_rete_op`'s `Fallback` arm now faces three ways a core op signals its hole. This is a fourth,
and it is **not** covered by any existing arm:

| family | core signals by | the arm does |
|---|---|---|
| i64 | raising `Err` | catch it |
| f64 | returning a non-finite | inspect the `Ok` scalar |
| holon | returning an outcome **enum** | inspect the variant, unwrap the payload |
| **Option** | **returning `None`** | **unwrap `Some`'s payload; `None` takes the caller's value** |

**⚠ `Option` is `Value::Option(_)`, NOT `Value::Enum`** (`runtime.rs:7581` —
`Value::Option(_) => ":wat::core::Option"`). The holon arms match `Ok(Value::Enum(ev)) if
ev.type_path == …`; those **will not fire**. This needs its own arm on `Value::Option`.

**★ This arm is the most reusable of the four**: every core verb returning `Option<T>` becomes
fallback-able by it, with no per-verb work. Write it generically over `Value::Option`, not
specially for `get`.

## PART B — the three sequence rows

**`:wat::rete::core::PersistentVector/get` already exists as an `Alias`** returning `OptionOf("T")`.
It **converts** to `Fallback`; it is not a new row. Two siblings join it.

| rete_name | core_name | params → ret |
|---|---|---|
| `:wat::rete::core::PersistentVector/get` | `:wat::core::PersistentVector/get` | `[PersistentVectorOf("T"), I64, Keyword, Var("T")] -> Var("T")` |
| `:wat::rete::core::Vector/get` | `:wat::core::Vector/get` | `[VectorOf("T"), I64, Keyword, Var("T")] -> Var("T")` |
| `:wat::rete::core::List/get` | `:wat::core::List/get` | `[ListOf("T"), I64, Keyword, Var("T")] -> Var("T")` |

All `meta: { pure: true, deterministic: true, total: true }` — **earned by the fallback**, exactly as
the i64/f64/holon rows earn theirs. Confirm each core verb exists and returns `Option<T>` before
writing its row; do not take this table's word for it.

Row count: **63 → 65** (two new; one converted in place).

### ⚠ The cost, stated because it is real

A rule author **loses the ability to distinguish "absent" from "present and equal to my default"**
inside a `where`. That is the deliberate trade the ruling makes — a rule *condition* wants a value,
not a sum type. The `Option`-returning form remains available in core for ordinary wat code; only
the **rete spelling** changes. Record this in the rows' comment so it is a known trade, not a
discovered surprise.

## ⛔ DECLARED OUT, with reasons — the keyed family

`HashMap/get`, `PersistentMap/get`, `Record/get`, `HashSet/get`, and the generic `:wat::core::get`
are **NOT in this stone**:

- `HashMap<K,V>` / `PersistentMap<K,V>` are **two-parameter generics** and `ParamType` has no variant
  that can spell one. That is real machinery (a `MapOf(K,V)` variant), not a row — and it is the same
  question for every future two-param container.
- `Record/get` takes a record and a field name; a different shape again, and the
  declaration-derived accessor door may already cover it — **ground before assuming**.
- `HashSet/get` — what a set's `get` even means needs grounding, not a guess.
- The generic `:wat::core::get` is polymorphic over all of them; per the standing per-type ruling
  the rete surface does not carry generic dispatch.

**Report these; do not mint them.** They are a declared follow-up, and this brief is their record.

## ⛔ STOPs

- **⛔ STOP-1 — write the `Option` arm GENERICALLY**, over `Value::Option`, not special-cased to
  `get`. It is the cure that will serve every future `Option`-returning verb.
- **⛔ STOP-2 — do NOT touch the three `first` rows.** They catch `first`'s own `MalformedForm`
  raise and are correct as they stand. Whether `first` should instead route through `get` is a
  separate question (its row carries no index to pass).
- **⛔ STOP-3 — do NOT mint an `nth` row, and do NOT mint any row for a verb defined via
  `Option/expect`.** Ruled 2026-08-05: `expect` is the *discard* of an already-faced outcome; a
  fallback wrapper would legitimise a panic inside a rule condition. See the four-cures note.
- **⛔ STOP-4 — do NOT touch core.** `PersistentVector/get` keeps returning `Option<T>` for ordinary
  wat code. Only the rete row changes shape.
- **⛔ STOP-5 — if converting `PersistentVector/get` from `Alias` to `Fallback` breaks any existing
  caller, STOP and report the list.** The rows are hours old and the fence is unarmed, so the
  expected answer is zero — a non-zero answer is a finding.
- **⛔** No `_` wildcard arm on an enum scrutinee.
- **⛔** Do not commit, stash, push, or touch git.

## Verify — FOREGROUND, block, SOLO

```
cargo build --release
cargo nextest run --release
cargo clippy --release --all-targets
./wat-scripts/perf/grid/check-where-shapes.sh
```

Read the **Summary line**, never a piped exit code.

## EXPECTATIONS

| # | what | expected |
|---|---|---|
| 1 | row count | **65** |
| 2 | ★★ **in-range returns the element** | `(… PersistentVector/get (PV 7 8) 1 :undefined -1)` → `8`, fallback not taken |
| 3 | ★★ **out-of-range takes the fallback** | same PV, index `9` → `-1` |
| 4 | ★★ **non-vacuity** | same out-of-range expression, `:undefined -1` then `:undefined 42` → `-1` then `42` |
| 5 | ★ **empty container** | `(… get (PV) 0 :undefined -1)` → `-1` |
| 6 | ★ all three containers | `Vector/get` and `List/get` behave identically to `PersistentVector/get` |
| 7 | ★ **the seam still composes** | the full `where`-shaped expression at the top of this brief type-checks and evaluates |
| 8 | ★ i64/f64/holon fallbacks unregressed | `(… i64::/ 1 0 :undefined -1)` → `-1`; `(… f64::/ 0.0 0.0 :undefined -1.0)` → `-1.0`; `(… holon::cosine zero other :undefined -1.0)` → `-1.0` |
| 9 | ★ `first` unregressed | the three `first` rows still fall back on empty |
| 10 | ★ the naming-rule tests still pass | `every_row_is_admitted` and its three siblings — the new rows must satisfy them or be documented exceptions |
| 11 | ★ floor | ≥ `4356`, nothing lost |
| 12 | ★ gate | `9 pair(s), 98 rows` |
| 13 | clippy | clean |

Rows 2, 3, 4, 7, 8, 10 re-run by the orchestrator by hand.

**Runtime prediction: 40–60 minutes.** Time-box 120.

**Trap doors:**
1. **Matching `Value::Enum` for `Option`.** It is `Value::Option`. The holon arms will not fire.
2. **Special-casing the arm to `get`.** STOP-1 — write it over `Value::Option` generally.
3. **Forgetting `PersistentVector/get` is a CONVERSION.** Editing it as if adding a row leaves a
   duplicate `rete_name`; `every_rete_name_is_unique` will catch it, but know it first.
4. **Minting the keyed family because it "looks the same."** It needs a two-param `ParamType`.
5. **Skipping the non-vacuity row.** Rows 2/3/5 all pass if the arm returns a constant.

## Scratch

Scratch `.wat` in `wat-scripts/scratch-pad/` — never a tmp dir; it is parsed and type-checked on
every build, which is the point. Real program, real `:user::main`.
