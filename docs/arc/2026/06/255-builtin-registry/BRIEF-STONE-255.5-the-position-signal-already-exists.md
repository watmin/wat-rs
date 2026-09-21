# BRIEF — STONE 255.5: the POSITION SIGNAL already exists. Wire it to the slots that need it.

**Drawn 2026-09-21 against `main` @ `8df49e008`** (floor 5939/5939, clippy 0, census `no STOP-8`).
Arc: **104 → 97 → 80 → 77.** ⛔ **This is the last large class before 251.8d can restart.**

## ⛔ THE BRIEF THAT WAS ABOUT TO BE WRITTEN WAS WRONG — again, and measurement caught it again

255.2 proved a **predicate over leaves** cannot close this class (three cures, each RAISED the count:
116 · 108 · 114) and concluded *"a position grammar is the next peel."* The orchestrator was about to
brief exactly that. ⭐ **Then it measured, and the grammar already exists.**

## ⭐ THE MECHANISM IS ALREADY THERE — `also_accept_type`

`src/resolve/normalize.rs` has **two** call sites and they differ by one boolean:

```rust
:119  general symbol walk           resolve_namespaced_symbol(…, false)   ← every ordinary reference
:604  normalize_type_binder_head    resolve_namespaced_symbol(…, true)    ← knows it is a type slot
```

and inside (`:521`) a **general** type acceptance, already routed through the one door:

```rust
if also_accept_type && sym.types().is_some_and(|t| t.is_known_type(&primary)) { … }
```

⇒ ⭐ **The position signal exists, is already plumbed, and is set for exactly ONE slot.** An
annotation slot — `[n :- wat/WatAST]` — travels the general walk with `false`, so a perfectly known
type is asked the *reference* question and fails.

**Measured, annotation position:**

| spelling | verdict |
|---|---|
| `wat.type/i64` | ✅ OK |
| `wat/WatAST` · `wat.core/i64` · `wat.time/Instant` · `wat.rete/Overlay` | ⛔ `UnresolvedReference` |
| all four, colon spelling | ✅ OK |

**Only `wat.type/X` survives**, via a *second* block at `:531` that skips the flag entirely.
`normalize.rs:525-528` explains why, and says the quiet part:

> *"`:wat::type::` is a TYPE-ONLY namespace: a name under it is never a call head, **in ANY
> position**… **The namespace IS the position**; `normalize` carries no position context and needs
> none."*

⛔ **That sentence is TRUE for `wat.type` and FALSE for every other type namespace.** `wat`,
`wat.core`, `wat.time`, `wat.rete` hold types **and** functions, so their namespace cannot encode
position — **they need the flag, and no caller sets it for them.**

⭐ **This also EXPLAINS 255.2's three dead ends.** A predicate over leaves failed because the
discriminator is **the caller's position knowledge** — which exists, and simply is not passed.
**255.2 was measuring the wrong layer, and its own conclusion pointed here.**

## THE RESIDUE — 44 unresolved references, by path

| path | n | kind |
|---|---|---|
| `:wat::WatAST` | **65** | a TYPE in annotation position |
| `:wat::time::Instant` · `:wat::rete::Overlay` · `:wat::holon::HolonAST` · `:wat::core::i64` | 19 | same |
| `:probe-homog::{serve-proc,Reply,Op}` · `:probe::same-shape?` | 20 | **user DECLARATION names** |
| `:wat::telemetry::journal::start` · `:wat::query::mem-store::start` | 12 | functions — ⚠ **classify before assuming** |
| `:wat::rete::core::defn` · `:wat::core::not-a-special-form` | 7 | ⚠ `not-a-special-form` is a **deliberate negative-test name** — expect it to stay |

## The work

1. ⭐ **Wire the signal to the annotation slot.** Find every position that is a **type slot** and
   route it through the `also_accept_type = true` path. ⛔ **Derive the set** — `:-` annotations,
   return slots, binder heads, nested parametric args, tuple elements. **Do not enumerate from this
   brief**; the last two briefs' enumerations were the eighth and ninth corrections.
2. **RE-RUN THE DELTA** (baseline **77**) **+ the classification table.** ⛔ One tree, named.
3. **Then the declaration-name class** (the `probe-homog` group) — `items[1]` of a declare form is a
   **name**, not a reference. ⚠ **Separate step, separate delta reading**, so the two causes stay
   attributable the way 255.3's did.
4. ⛔ **Fix the comment.** `normalize.rs:525-528` asserts *"`normalize` carries no position context
   and needs none."* If this stone gives it position context, **that sentence must go** — a comment
   that states a retired design is how the next hand gets it wrong
   (`[[feedback_a_comment_can_ship_a_gap_as_a_law]]`).

## The gate

- ⭐ **THE DELTA (baseline 77) + the classification table**, one named tree, before and after.
- `scripts/floor.sh` green; clippy `-D warnings --all-targets --workspace` **0**; census
  `no STOP-8`. ⛔ Run crate clippy **and the lint suite** yourself first.
- ⛔ **A type accepted in annotation position must NOT become acceptable as a call head.**
  `(wat.time/Instant)` in call position must still fail. **A non-vacuity control for the
  widening** — this stone loosens a check, and a loosened check needs proof it did not loosen
  everything (`[[feedback_a_negative_control_that_can_be_kept_must_be_kept]]`).
- Predict the test delta; confirm with `cargo nextest list`.
- ⛔ **NOT ONE `.wat` CONVERTED.** New fixtures expected.

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture whole the first time.
- ⛔ **ONE DOOR.** 255.4's 32-red came from adding a *second* retirement consult when one existed.
  **`also_accept_type` is the door here — pass it, do not duplicate the question.**
- ⛔ **`wat/*.wat` is `include_str!`'d** — invisible until `cargo build --release`.
- ⛔ **REPORT WHAT A DOOR CANNOT EXPRESS.** Five stones running, the best output of this arc.
- ⛔ **If this brief contradicts the code or the runner, THEY WIN.** **Ten corrections across eight
  stones.** This brief exists *because* the tenth was caught before it shipped. **Assume an
  eleventh.**
- Work only in `/home/john/work/holon/wat-rs`. **Do not push.**

## Out of scope

- **8d-ii / 8d-iii** — they wait on the delta.
- **The `:restricted-to` validation pass** — its *resolution* half is the declaration-name class in
  step 3; **say whether it closed.**
- **The `wat.type` NAME SET**, **capacity `:panic`** — deferred by the builder.
