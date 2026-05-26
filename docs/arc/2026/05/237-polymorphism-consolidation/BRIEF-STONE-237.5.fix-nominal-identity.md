# BRIEF — Stone 237.5.fix-nominal-identity → the one type-identity authority

**Status:** READY TO SPAWN. `model: "sonnet"`. (REWRITTEN from the band-aid version — do NOT patch `concrete_type_name_matches` in isolation; mint the shared authority.)

## What to do

There must be **exactly one way** to ask a value its declared type. Today two consumers re-derive it and drift: `eval_type` (`:wat::core::type`) reads `sv.type_name` (struct/newtype correct) but misses Enum; conforms? only special-cases Record (struct/newtype/enum all wrong). Mint ONE authority, route both through it, delete the swallowing wildcards. Make the 12-contract probe go 12/12 (it's 8/12 now).

## The fix

### 1. Mint the one authority — exhaustive, wildcard-free

`Value::declared_type_name(&self) -> String` in `src/runtime.rs` — the single value→declared-type-FQDN function. **No `other =>` / `_ =>` arm that a type-bearing variant could fall into** — every variant explicit, so the Rust compiler forbids a future variant from silently returning the generic kind (the exact rot that hit Enum). This is the ✅✅✅ guard (same shape as `#[wat_value]` / `CheckResult`).

Per-form FQDN source (factor from `eval_type` at runtime.rs:15893-15898, which already has the first three; ADD Enum):
| Value kind | declared FQDN |
|---|---|
| `Value::holon__HolonAST(h)` | `extract_classifier(h)` |
| `Value::Struct(sv)` | `sv.type_name` (strip leading `:`) — also covers newtype (it's a Struct under the hood; probe_11 confirms `type` already gets it right via this arm) |
| `Value::wat__Record { class_fqdn }` | `class_fqdn` |
| `Value::Enum(ev)` | the `EnumValue`'s declared enum FQDN — **the missing arm**; find the field on `EnumValue` (NOT `type_name()`, which gives generic `"wat::core::Enum"`) |
| every primitive/kind variant (i64/u8/f64/bool/String/keyword/nil/Uuid/Char/Vector/List/HashMap/HashSet/Tuple/fn/…) | explicit arm → `self.type_name().to_string()` |

If the primitive tail is large, list them explicitly anyway (the verbosity IS the compiler-guard) — do NOT use a bare catch-all that a future type-bearing variant slips into.

### 2. Route both consumers through it

- `eval_type` (`:wat::core::type`, runtime.rs:15892) → call `declared_type_name` (fixes `type` for enum — load-bearing, probe_10).
- conforms?'s nominal arm → call `declared_type_name`, compare to the stripped Path name. Delete `concrete_type_name_matches`'s wildcard (it collapses into the authority).

### 3. `type_name()` stays the *kind* accessor

`Value::type_name() -> &'static str` is the variant kind, NOT the declared type. Leave it (widely used). If cheap, a one-line doc note: "variant kind, not declared type — use `declared_type_name`."

## Read in order

1. `docs/arc/2026/05/237-polymorphism-consolidation/DESIGN-STONE-237.5.fix-nominal-identity.md` — reframed sub-DESIGN (the ladder, the drift table, the refined fix).
2. `tests/probe_arc237_stone5fix_nominal.rs` — **LOAD-BEARING** 12 contracts (9 conforms? + 3 `type`); pre-fix 8/12. Make 12/12.
3. `src/runtime.rs:15892` (`eval_type`) — the extraction to factor out + the per-form arms it already has.
4. `src/runtime.rs` `concrete_type_name_matches` — the second copy to collapse into the authority.
5. `Value` enum defn + `Value::type_name()` (runtime.rs:1219) + `Value::Enum`/`EnumValue` defn — to find the enum FQDN field + enumerate variants for the exhaustive match.

## Discipline

- Modify `src/runtime.rs` ONLY. No check.rs. No holon-rs (STOP-5). No new Value variant.
- The authority is **exhaustive — no swallowing wildcard for type-bearing variants.**
- BOTH `eval_type` and conforms? route through the one authority (no third copy).
- Do NOT touch the working Record/primitive behavior (probe_08/09 + 237.5's 12/12 stay green).
- Do NOT build is-<Name>? (237.6). Do NOT do the ✅✅✅✅ encapsulation rung. Do NOT commit.

## STOP triggers (REJECTION — not permission to defer)

1. Lib baseline < 827.
2. 237.5 probe (`probe_arc237_stone5_conforms`, 12/12) regresses; any 237.1–237.4 probe regresses; arc 234.0 `type` probe (`probe_diagnostic_polymorphic_type`, 8/8) regresses.
3. holon-rs touched (STOP-5).
4. Files outside `src/runtime.rs` touched.
5. fix-probe doesn't reach 12/12.
6. The authority keeps a bare `_ =>`/`other =>` catch-all that a type-bearing variant could fall into (defeats the ✅✅✅ guard).
7. 75 min elapsed.

## FM 2-bis evidence

`tests/probe_arc237_stone5fix_nominal.rs` (committed `beb8a78a`) — 12 contracts. Pre-fix 8/12: enum/newtype/struct self-conformance (probe_01/04/06) + `(type enum)` (probe_10) fail. The drift is visible: probe_12 (`type` struct) passes while probe_06 (conforms? struct) fails — same value, opposite answers. Post-fix 12/12, both consumers agreeing by construction.

## SCORE doc

`docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-237.5.fix-nominal-identity.md` (NEW). 12-row scorecard + the per-form FQDN-source table + a line confirming the authority is wildcard-free (grep it: no bare `_ =>` swallowing type-bearing variants) + line count + honest deltas. Mirror 234.3c.fix SCORE shape (tight).

## Calibration

One authority fn (exhaustive match) + two call-site routings + wildcard deletion. Single file. **Target band: 20–40 min Mode A; 75 STOP.** Successive-attempt aware (per 233.2 chain): if exhaustiveness fights the variant set or a consumer can't route cleanly, surface it as the next rung — don't force.
