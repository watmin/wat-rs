# DESIGN STONE — the session is the boundary, and its limits are TOTAL

> **THE RULING (builder, 2026-08-29):** *"the session is the boundary - it may not consume more
> than the configured amount of memory, 1G by default…. let's impose this constraint honestly - the
> user can choose to go further… but we have boundaries expressed and enforced.. max-session-memory
> / max-fire-rounds … insert affects memory just as must … as insert via derivation in fire-rules…
> yes?… we can exhaust memory before fire-rules begins?"* — then, the same day:
> *"let's impose session's strict limits via totality."*

Two commitments, and they are one commitment seen from two sides:

1. **The boundary is the SESSION**, not one fire. Both doors a session grows through enforce it.
2. **A limit the substrate cannot prove statically becomes a VALUE, not a raise.**

## Why (2) follows from what we measured, rather than from taste

eBPF refuses an unbounded program at LOAD: static, total, no runtime failure for anyone to handle.
This arc tried to copy that and **measured that we cannot** — a guarded counter's bound is its
SEED, which is runtime data (`12cdf4081`: *provably TERMINATING, not provably BOUNDED*). So the
failure is irreducibly dynamic. A dynamic failure in this substrate is a matchable value: the arc
already built exactly this wall one layer over, for comms (`DESIGN-recv-outcome-wall.md`,
`RecvOutcome`/`SendOutcome`/`TrySendOutcome`/`CloseOutcome`). **`fire-rules` and `insert` are the
odd ones out, and they are the two with ceilings.**

## S1 — ONE CONTRACT, TWO DOORS (landed 2026-08-29)

**The hole, measured.** `2_500_000` facts staged with NO fire → peak RSS **4.0 GB** against a 1 GiB
contract, **no diagnostic**. The ceiling was checked only inside the fixpoint: a FIRE ceiling
wearing a SESSION ceiling's name.

**The session's zero point** is `alloc_counter::mark_session_origin()`, called at `arm-session` —
which `compile-all` calls for every session it builds, and which `stratify.rs` already calls *"the
one door every rule passes"*. The fixpoint's per-fire snapshot is DELETED: a per-fire zero cannot
express a per-session contract, because it forgets everything `insert` staged before it.

**The decision is made once** (`rete::kernel::session::session_ceiling_breach`) and read at both
doors — `insert`/`insert-all` and the fixpoint's round boundary. Two places holding one truth is
the defect this arc pulls out most often.

**Two error variants, not one.** The fire door reports ROUNDS COMPLETED; that number is meaningless
where no rounds run. One variant serving both would hand every insert-site reader a field that is
always zero — a value carrying two facts, the exact shape `Ret::Is`/`NoScheme` was minted to kill
one day earlier. Each door reports how far IT had got: `rounds` at one, `staged` at the other.

### ⚠ WHAT IS MEASURED, stated because the diagnostic must not overstate it

Bytes live on the session's **own thread** since `compile-all` — not a walk of the session's
structures, which `Arc` sharing makes ambiguous (once, or once per holder?) and which would be
O(n) per insert besides. So **anything else the program allocates on that thread after
`compile-all` is charged too.** Driven: a probe's `(range 0 200000)` showed as `used=11_196_940`
against `staged=1`.

That is deliberate and it is the SAFE direction — the ceiling exists to raise a diagnostic before
the allocator aborts, and the allocator does not care whose bytes they are. The messages now say
this outright, and the two numbers read TOGETHER are the diagnostic: **a large `used` against a
small `staged` says the memory is not the facts.**

The origin's placement bounds the exposure: data built BEFORE `compile-all` is never charged.

**It counts LIVE bytes, not cumulative allocation** — the counter decrements on free — so transient
work between fires does not accumulate. A long-lived session on a busy thread is charged for what
is still HELD, which is the quantity that actually matters.

⚠ **THE ONE DRIFT, NAMED: a cross-thread `Arc` free decrements the FREEING thread.** So a session
whose values are handed to other threads drifts UPWARD over its lifetime, and a long-lived service
that arms one session at startup could in principle accumulate a false refusal. Drift is bounded by
how far values actually cross threads; a rete session's facts normally do not. **This is a
measurement to take before anyone builds a long-lived armed session on a hot thread** — it is not
a reason to hold the ceiling back, since the alternative today is a 4.0 GB abort with no diagnostic.

### ⛔ THE GATE THAT SILENTLY CHANGED WHAT IT PROVED

`probe_arc278_session_memory_ceiling.wat` seeded 500 facts at a 4096-byte ceiling and asserted the
FIRE door refused. The moment `insert` began enforcing the same ceiling, **the first insert
refused** — and the gate would have gone green again on a one-word tag change, now proving the
insert door while its name, its prose and its `rounds` assertion all still claimed the fire one.

A control can lose its power without ever failing (recovery-file FM 34). The fix is not a tag edit:
the fire door needs a workload the insert door **cannot** catch. It is now a cross-product —
**400 staged, 40_000 derived**, non-cyclic and range-restricted so the verifier admits it,
multiplying WITHIN one round, which is precisely the axis the round cap cannot see.

**The ceiling is bisected, not picked** (2026-08-29, this exact workload):

| ceiling | outcome |
|---|---|
| 1 MiB · 4 MiB · 16 MiB | refused at the FIRE door |
| 64 MiB · 256 MiB (default) | completes, all 40_000 derived |

16 MiB sits inside the refusing band with staging orders of magnitude below it.

## S2 — TOTALITY: the outcome wall reaches rete (NOT STARTED)

**MEASURED: a ceiling breach kills the program.** Both ceilings raise; a probe printing before and
after gets only the "before". `fire-rules` is declared `Session -> Session` and cannot always
produce a Session — the signature lies, and S1 added a second way for it to lie by giving `insert`
the same power.

### THE ONE CONTRACT DECISION (pinned)

**Two enums, not one, and not `Result`.** Each door returns the outcome of ITS door:

```clojure
(:wat::core::defenum :wat::rete::InsertOutcome :wat::enum::Pure
  :Inserted              [session <- :wat::rete::Session]
  :MemoryCeilingExceeded [limit <- :wat::core::i64  used <- :wat::core::i64  staged <- :wat::core::i64])

(:wat::core::defenum :wat::rete::FireOutcome :wat::enum::Pure
  :Fired                 [session <- :wat::rete::Session]
  :MemoryCeilingExceeded [limit <- :wat::core::i64  used <- :wat::core::i64  rounds <- :wat::core::i64]
  :RoundCapExceeded      [cap   <- :wat::core::i64  still-deriving <- :wat::core::i64])
```

- **NOT one shared enum.** `insert` runs no rounds, so a shared enum forces every insert site to
  match an arm that cannot occur. Unreachable arms are the two-facts defect wearing a match's
  clothes, and this arc has now paid for that shape three times.
- **NOT `Result<Session, E>`.** The substrate's settled idiom is a named closed outcome enum per
  concept — `RecvOutcome`, `SendOutcome`, `ReadOutcome`, `NextOutcome`, `ReadJsonOutcome`,
  `ReadlnOutcome`. A `Result` here would be the only one of its kind and would name neither door.
- **NOT a narrower reshape that leaves a raise path alive** — a live raise path is not the root
  (`DESIGN-recv-outcome-wall.md`'s own pinned decision, for the identical reason).
- **In wat, not `types.rs` — and this is CHECKED, not merely preferred.** `wat/rete/compile.wat:241`
  already carries `(defenum :wat::rete::Axis :wat::enum::Pure)`, and arc 296's doctrine is *wat is
  the source of truth; Rust consumes it*. `RecvOutcome` is a `types.rs` builtin only because
  `recv'` is used inside the stdlib before a wat `defenum` would load; rete has no such load-order
  problem. **The mechanism exists:** `::wat_source_derive::wat_enum_from!` reads a `.wat` `defenum`
  at BUILD time and emits the registration (`src/intrinsic/mod.rs:54,60,66`), and
  `builtin_enum_variant_names` (`runtime.rs:22813`) already carries a `.wat`-declared-exceptions
  arm for exactly the case where Rust must CONSTRUCT a wat-declared variant — which `insert` and
  `fire-rules` will, being native. Rust builds the value with `Value::Enum(Arc::new(EnumValue{…}))`;
  `next_outcome_item` (`runtime.rs:12798`) is the two-field exemplar to copy.

**THE FAILURE ARMS CARRY NO SESSION, and that is what makes this clean.** `Session` is an immutable
VALUE, so the caller still holds the pre-call one. Nothing half-fired or half-staged escapes, and
there is no question of handing back a mid-fixpoint session with inconsistent memories. That is the
objection this design would normally founder on; value semantics dissolve it.

### The corpus sweep — MEASURED, not inherited

Counted 2026-08-29 across `wat/`, `tests/`, `wat-tests/`, `wat-scripts/`, `.edn`:

| verb | sites |
|---|---|
| `fire-rules` (incl. `$oracle` 103) | 529 |
| `insert` | 530 |
| `insert-all` | 110 |
| `fire-once` | 31 |

⛔ **`fire-once` AND `fire-rules-explain` ARE IN SCOPE, and leaving them out is how the wall gets a
hole.** Traced on the disk 2026-08-29, not assumed:

- `fire-once$native` → `eval_fire_once_native` (`fire/mod.rs:1189`) → `fire_once_session`
  (`fire/mod.rs:909`) → `fire_fixpoint_delta_armed(…, FireKind::Once)` (`:924`) → **the ceiling.**
- `fire-rules-explain$native` → `eval_fire_rules_explain` (`fire/delta.rs:802`) →
  `fire_rules_on_session` (`:826`) → **the ceiling.**

Both can breach today and both are declared total. A wall with one unmatched door is not a wall.
`fire-rules-explain` returns `:wat::rete::Explained`, so it needs the same treatment one level out —
either an `ExplainOutcome` or `Explained` reached through `FireOutcome`; **that choice is the one
piece of S2 still open**, and it is small enough to settle when the reshape reaches it.

This is **wat-fix territory, not hand edits** (R21: *"we use wat-fix to unfuck the farm — do not
fear refactors, they are one-to-three shot"*), and the exhaustive-match cascade DRIVES the sweep —
each red site names the next. The `$oracle`s must return the same TYPE by the dual-impl contract
but need no ceilings of their own: they always answer `Inserted`/`Fired`, covered by the standing
asymmetry (*"the `$oracle` is the reference an embedder never runs"*).

### Sequencing — ⛔ SMALLEST VERB FIRST, and that is a method choice, not timidity

The recv wall reshaped ONE verb and swept 160 sites. This wall has FOUR verbs and ~1_200 sites, so
reshaping them all at once means the first time the full pattern is exercised end-to-end is also
the moment 1_200 sites are red. **`fire-once` has 31 sites. It is the disconfirming probe for the
other three** — the same trick `examinare` asks for, applied to a migration: make the cheap thing
fail on exactly the unknown before betting the corpus on it.

The unknowns it settles, all of them cheap to learn at 31 sites and expensive at 530:
- does `wat_enum_from!` carry a `:wat::rete::`-namespaced `defenum` at build time?
- does `builtin_enum_variant_names` need a wat-declared-exceptions arm for it, as `ServiceEvent`
  does, or does the derive suffice?
- what does a `match` over `FireOutcome` cost at a call site, in real wat, read back?
- does the `$oracle` dual-impl contract hold when only the native side can produce a failure arm?

1. ~~**S1** — one contract, two doors, still raising.~~ **LANDED.**
2. **S2a — `fire-once` end to end.** `FireOutcome` in wat, the derive, the rust construction, the
   signature, and its 31 call sites migrated by a recorded codemod. Floor green before anything
   else moves. **This is the strike that proves the pattern.**
3. **S2b — `fire-rules` + `fire-rules-explain`** on the proven pattern (529 + explain).
4. **S2c — `insert` / `insert-all`** with `InsertOutcome` (640).
5. **S2d — the lint wall:** a ceiling breach may not be constructed as a raise anywhere in rete.
   The rung that stops the class regrowing; the recv wall's S4 is the shape to copy.

Each of 2–4 is its own codemod under `wat-scripts/fixes/`, dry-run and diffed on a `/tmp` copy
before it touches the corpus (R21; never hand edits, never python/sed). The exhaustive-match
cascade drives each sweep — every red site names the next.

### Out of scope — affirmative cuts, not deferrals
- **`max-fire-rounds` and `max-session-bytes` stay CONFIG, not per-call arguments.** A bound passed
  per call is a bound each caller can quietly raise; one program, one value, chosen at startup, is
  the same ruling `dim_count` already carries.
- **Bounding a session's whole history across fires** is what S1 now does BY CONSTRUCTION (the
  origin is `compile-all`, not fire entry). `config.rs`'s old "Not cumulative across fires"
  paragraph asserted the opposite and is struck — it was the ruling the builder overturned.
