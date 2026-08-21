# SCORE — ②a strike A: **THE STONE FAILED. REVERTED.** And it found the reason 109 could not see.

Rider: 9.5 min, 157 of 166 sites filled, 2 findings raised. **Reverted whole. Floor is green at
4818/4818 with `wat/rete.wat` untouched.**

The failure is mine, in two distinct places, and the finding it produced is worth more than the stone.

## What happened

| step | result |
|---|---|
| rider filled 157/166, raised 2 STOP-1 findings | reasonable work, verified against its own probe |
| orchestrator built + floored | **2198 passed, 2620 FAILED** — the stdlib would not load |
| reverted the 24 `Value`-typed annotations | still 2620 |
| **reverted `wat/rete.wat` whole** | ✅ 4818/4818 |

## ⛔ FAILURE 1 — `Value` for `bindings` was WRONG, and I handed it over as SETTLED

The first floor's arm:

```
:wat::core::>:  parameter #2 expects :wat::core::Value; got :wat::core::i64      wat/rete.wat:3563
:wat::core::<:  parameter #2 expects :wat::core::Value; got :wat::core::i64
PersistentMap/get:      parameter #2 expects :wat::core::i64; got :wat::core::Value
PersistentVector/conj:  parameter #2 expects :wat::core::i64; got :wat::core::Value
```

**rete DOES consume binding values** — as comparison operands, as map keys, and as vector elements,
at ~16 sites. That is `types.rs:1160`'s arc-278 R7 warning firing verbatim: *"a `Value` payload can be
PRODUCED but never CONSUMED."*

**I dismissed R7 on a measurement that was too narrow.** I tested three operations — `=`, a
presence-`match`, and `get`→`assoc` — found all three transport-safe, and generalised to every
operation rete performs. `>` and `<` are not transport; they need an ordered concrete type. I never
tested them.

★ Worse than being wrong: **I wrote it into the brief as "the worked example, already settled
(builder-ruled)"**, which is precisely the framing that tells a rider not to question it. The rider
applied it faithfully to 24 sites and had no reason to check. A wrong answer labelled *settled*
disables the one check that would have caught it.
`[[feedback_a_claims_support_does_not_travel_with_the_claim]]`

## ⛔ FAILURE 2 — THE ROLES ARE COUPLED, and this is the real finding

Reverting `Value` did **not** fix the floor. 2,907 occurrences of one root cause remained:

```
:wat::core::if: parameter else-branch
    expects  PersistentMap<i64, PersistentVector<wat::rete::Token>>
    got      PersistentMap<i64, PersistentVector<wat::core::Record>>
                                                    wat/rete.wat:2563
```

`walk-sorted-ids` threads **ONE** `acc` parameter through three phases that return three different
memory types — alpha (`Element`), beta (`Token`), production (`Record`).

The rider **found this and raised it as STOP-1**, quoting the function's own comment: *"Walking by
index keeps Acc unparameterized."* It left `acc` bare, correctly.

**What neither of us saw is that leaving it bare is not enough.** Typing the memories at their *other*
sites makes the phase functions return concrete types, and the `cond` in `walk-sorted-ids` then cannot
reconcile three of them. **The memories cannot be typed while the phase-walker is polymorphic over
them.** They are one decision, not fifteen.

★ That is the thing 109 could not see from outside: rete's bare annotations are not 166 independent
omissions. A large block of them is **one architectural coupling** — a genuinely polymorphic walker
that wat has no generic-fn mechanism to express here. The comment *"Walking by index keeps Acc
unparameterized"* is not laziness; it is the author recording a constraint.

## What the rider got right, and it is most of the report

- **Caught an overloaded name by reading bodies, not the census.** `bm` at `:1934`/`:1936` is a
  local bindings accumulator, NOT `beta-mem`, despite 25 other `bm*` sites being beta-mem. My
  name-based census would have mistyped both.
- **Caught a STALE DOC COMMENT — which invalidates my brief's headline method.** The Session comment
  says `alpha-memory: node-id → {join-bindings → [Element …]}`, a nested shape. The code is FLAT.
  Verified independently: `join-bindings` appears **twice in the whole file**, and both are the doc
  comments themselves. It trusted constructors and consumers over the prose. **My brief said "read the
  comment, write the type it already describes." For at least two roles the comment is aspirational.**
- **Raised both real findings** (`Export`'s Rust-only fields; `walk-sorted-ids`'s polymorphic `acc`)
  rather than filling them with `Value` to reach a tidy zero.
- **Reported that `wat/rete.wat` is baked into the binary via `include_str!` (`src/stdlib.rs:362`)**, so
  `--check` cannot self-validate on-disk edits — and wrote a separate probe for the type shapes
  instead. Honest about what its instrument could and could not see.

⚠ That last point is exactly where the stone escaped: its probe validated that the type SHAPES were
legal in isolation. It could not validate that `rete.wat` still LOADS. Only the orchestrator's build
could, and that is where 2620 appeared.

## What survives

**Nothing in `wat/rete.wat`.** The probe is kept — `wat-scripts/scratch-pad/probe-stone-2a-bracket-mechanics.wat`,
loader-gated, exercising every composite type against the live Persistent schemes, green.

The 13 schemes (`9c82f157`) and the bracket acceptance (`f454c465`, `df90b990`) are untouched and
still green — this stone changed no `src/`.

## What ②a must become

1. **`walk-sorted-ids`'s polymorphism is the FIRST question, not a leftover.** Until it is resolved,
   the memory roles cannot be typed at all. It is one decision gating ~60 sites.
2. **`bindings`' V is REOPENED.** Not `Value` — rete compares and keys on binding values. R7's own
   resolution (parametric over `T`) points at `Token<V>`, which cascades.
3. **The doc comments are evidence, not authority.** Constructors and consumers outrank them; two are
   already known stale.
