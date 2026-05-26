# DESIGN — Stone 237.5.fix-nominal-identity → the one type-identity authority

**Status:** ACTIVE (2026-05-25 night-late). REFRAMED from "patch conforms?'s helper" to "mint the single value→type authority" — the band-aid was rejected (it leaves `:wat::core::type` broken and two drifting copies).

**Origin:** Stone 237.5's conforms? was probed on record/primitive/union/vector/alias — never enum/newtype/struct. The 237.6 crawl traced the break, and the fix-probe confirmed it harder: **all three TypeEnv-registered nominal forms return false-for-self.** Root cause is not a conforms? bug — it's that **there is no single authority for "what type is this value."** Two consumers each re-derive it with their own per-`Value`-kind match, both ending in a swallowing `other => type_name()` wildcard:
- `eval_type` (`:wat::core::type`, arc 234.1) — handles HolonAST/Struct/Record; **Enum + Newtype fall to the wildcard → generic kind string.** So `(:wat::core::type some-enum)` returns `"wat::core::Enum"`, not `:my::Color`. **234.1 shipped `type` incomplete; nobody probed enum/newtype, so it sat latent until conforms? made it load-bearing.**
- `concrete_type_name_matches` (237.5) — a second, worse copy (only Record + wildcard).

`Value::type_name()` is the decoy: it *looks* like the type authority but returns the variant *kind* (`&'static str`), not the per-instance declared FQDN. Reaching for it is the trap both consumers fell into.

## Doctrine — this is "exactly one way to do a thing"

There must be **exactly one way** to ask a value its declared type. Today there are two (drifting, both incomplete). This stone collapses them to one and makes the collapse compiler-enforced. Lineage — the structural-impossibility ladder we've climbed before:

- **arc 233.2.l** — `#[wat_value]` proc-macro forbids wrapping variants on `Value` (structural).
- **arc 236** — `CheckResult` sum-type makes silent error-loss un-writable (structural).
- **this stone** — the value→type authority is an **exhaustive, wildcard-free match**, so the compiler forbids a future `Value` variant from silently falling through to the generic kind — the exact mechanism that rotted Enum/Newtype.

### The ✅✅✅ ladder (and the successive-attempt honesty)

| rung | shape | guard |
|---|---|---|
| ✅ | patch conforms?'s helper | none — `type` still broken, copies still drift. REJECTED. |
| ✅✅ | one `declared_type_name` authority; route both consumers through it | convention (a future consumer could re-derive) |
| ✅✅✅ | that authority is **exhaustive — no `other =>` wildcard for type-bearing variants** | **the Rust compiler** — a new variant won't compile until its declared-type is stated |
| (✅✅✅✅) | per-variant identity fields encapsulated; the authority is the ONLY door | re-derivation becomes inaccessible — bigger (Value privacy / a ward) |

This stone targets **✅✅✅**. The fourth rung (encapsulation so private re-derivation is impossible) is named, not forced — per the 233.2 chain, the next structural question manifests from *this* attempt's result. If exhaustiveness or routing surfaces a deeper gap, that's the next stone.

## Scope

`src/runtime.rs`. ONE new authority + route the two consumers + delete the wildcards. No check.rs change expected. No holon-rs (STOP-5). No new Value variant.

## Locked decisions

### D1 — mint the one authority (exhaustive)

`Value::declared_type_name(&self) -> String` — THE single value→declared-type-FQDN function. **Exhaustive match, no swallowing wildcard for type-bearing variants:**

| Value kind | declared FQDN source |
|---|---|
| `Value::holon__HolonAST(h)` | `extract_classifier(h)` (mirror eval_type) |
| `Value::Struct(sv)` | `sv.type_name` (strip leading `:`) |
| `Value::wat__Record { class_fqdn }` | `class_fqdn` |
| `Value::Enum(ev)` | the `EnumValue`'s declared enum FQDN (find the field) |
| newtype value | the newtype's declared FQDN (runtime.rs:3007-3016 carrier) |
| every primitive variant (`i64`/`u8`/`f64`/`bool`/`String`/`keyword`/`nil`/`Uuid`/`Char`/Vector/List/HashMap/HashSet/Tuple/fn/…) | explicit arm → its `type_name()` string |

Primitives are **enumerated, not wildcarded** — so adding any future `Value` variant is a compile error until its declared-type arm exists. (If the primitive tail is large, group genuinely-kind-only variants behind a clearly-named helper that is itself exhaustive — but no bare `_ =>` that a type-bearing variant could slip into.)

### D2 — route both consumers through it

- `eval_type` (`:wat::core::type`) calls `declared_type_name` (fixing `type` for enum/newtype — load-bearing bonus, probed).
- conforms?'s nominal arm calls `declared_type_name` and compares to the stripped Path name. `concrete_type_name_matches`'s wildcard is deleted (it becomes a thin equality over the authority, or is removed in favour of the authority directly).

### D3 — `type_name()` stays the *kind* accessor

`Value::type_name() -> &'static str` is NOT the type authority — it's the variant kind. Leave it (widely used), but it must no longer be reached for "what declared type is this." If cheap, a doc note marks it "variant kind, NOT declared type — use `declared_type_name`." (A lint/ward forbidding `type_name()` in type-comparison contexts is the ✅✅✅✅ residual, not this stone.)

## Probe finding (the drift, proven empirically)

The expanded probe (12 contracts) shows the two copies disagree about the SAME value:

| value | `(:wat::core::type v)` | `(conforms? v :ItsType)` |
|---|---|---|
| struct `:my::Point` instance | **"my::Point" ✓** (probe_12) | **false ✗** (probe_06) |
| newtype `:my::Price` instance | **"my::Price" ✓** (probe_11) | **false ✗** (probe_04) |
| enum `:my::Color::Red` | **"wat::core::Enum" ✗** (probe_10) | **false ✗** (probe_01) |

So `eval_type` is the **more-complete** copy (reads `sv.type_name` → struct + newtype correct; newtype is a `Value::Struct` under the hood, caught by that arm) and conforms? is the **less-complete** copy (only Record). **Both miss enum.** This is the two-copies-drift made concrete: `type` and conforms? give opposite answers for a struct because they extract identity differently.

**Refined fix:** the authority is `eval_type`'s extraction, factored out + completed with the **Enum arm** (the one gap `eval_type` itself has) + made exhaustive (no swallowing wildcard). Route `eval_type` AND conforms? through it. struct/newtype inherit `eval_type`'s existing correctness; enum gets fixed for both; the consumers agree by construction.

## FM 2-bis probe

`tests/probe_arc237_stone5fix_nominal.rs` (committed `f65a2d08`) — proves the authority via **both** consumers. Extend it: alongside the 9 conforms? contracts, add `:wat::core::type` contracts — `(:wat::core::type <enum-val>)` → `"my::Color"`, `(:wat::core::type (newtype/new …))` → `"my::Price"` — so the single authority is proven correct through `type` *and* conforms?. Pre-stone: enum/newtype fail on both. Post-stone: all green.

## Out of scope (REJECTED — not deferral)

- is-<Name>? auto-mint — Stone 237.6 (rides conforms? → rides the one authority for free).
- Encapsulation / lint forbidding `type_name()` re-derivation — the ✅✅✅✅ residual; a possible next attempt, surfaced by this stone's result.
- holon-rs (STOP-5).

## Calibration

One authority fn + two call-site routings + wildcard deletion + probe extension. Single file. **Target band: 20–40 min Mode A; 75 STOP.** Mirror 234.3c.fix shape. Successive-attempt aware: if the exhaustive match fights the primitive tail or a consumer can't route cleanly, that surfaces the next rung — don't force it.
