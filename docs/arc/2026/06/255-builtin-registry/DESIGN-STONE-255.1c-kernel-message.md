# DESIGN STONE — 255.1c-kernel-message · HOME #5: the first rows NO GATE CAN CHECK

## The boundary is the Category — home #4's lesson, applied

`:Message`'s shipped prose names its population outright, derived by reading bodies during
255.1c-taxonomy:

```
;; Delivers or receives a payload across a peer/channel boundary to another locus —
;; `send`, `try-send`, `recv`, `select`, `poll`. The locus is a TYPED VALUE (`peer<I,O>`)
;; the caller already holds — contrast `:Io`, whose target is an ambient OS stream with
;; no caller-held handle.
```

Five verbs, all still literal arms. **The carve takes the Category, not the decomposition table's
`concurrency` row** — that row also holds `spawn-thread`, `spawn-process`, `close` and `after`, which
are a different DOING and belong to other homes. Home #4 established this: the table was drawn from
prefixes and adjacency; the taxonomy from bodies.

## ★★ THE POINT — the first registered rows whose declared types NOTHING verifies

Every home so far has had its `@arg`/`@ret` cross-checked by
`doc_arg_ret_types_match_checker_scheme` against a registered `TypeScheme`. **These five have no
registered scheme at all.** Measured: no `env.register` for any of them; instead `check.rs` holds a
bespoke inference arm per verb (`:4049` `:4061` `:4069` `:4176` `:4188` → `infer_send_prime`,
`infer_try_send_prime`, `infer_recv_prime`, and siblings), because the types are **projective and
∀-parametric** and a fixed-arity scheme cannot express them. From `check.rs:4042`:

```
//   send'     — projective: I flows from peer<I,O> into the payload arg.
//   recv'     — projective: O flows from peer<I,O> into the return.
//   close'    — ∀-parametric: peer<∀I,∀O>; clause cannot enumerate all (I,O).
```

And the gate's own first move is:

```rust
let scheme = match check_env.get(entry.name) {
    Some(s) => s,
    None => continue, // not yet in checker — skip
};
```

**So the gate will go green on these five by skipping them.** That green means nothing about them —
`[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]`. This home's honest claim is not
"five more rows verified"; it is **"five rows whose declared types are documentation only, and the
stone says so instead of letting a green scorecard imply otherwise."**

Home #3 met a weaker version of this: `readln'` has a *vestigial stub* scheme registered so the gate
would agree, and its module doc says so plainly (*"the registered scheme is vestigial"*). These five
do not even have a stub. **Minting stubs to manufacture gate coverage is REJECTED by this stone** —
a stub that exists only to be agreed with is the gate reading a copy of the truth, the exact defect
home #4 spent a ruling to remove one axis over.

## The one contract decision, pinned

**Declare `@arg`/`@ret` from what the INFERENCE ARM actually produces, and mark them unverified.**
Each row's doc block carries a line naming its `infer_*_prime` fn as the real authority. No stub
schemes. If a declared type cannot be written down honestly because the real type is projective, the
doc says *that*, the way `readln'`'s does.

## Rooms — the bodies are IN `runtime.rs` this time

Home #3's brief assumed inline bodies and was wrong (stdio delegates to `crate::services::`). Home #5
is the opposite and it was measured:

```
runtime.rs:6826  send      → eval_peer_send_prime       body at runtime.rs:31053
runtime.rs:6832  try-send  → eval_peer_try_send_prime   body at runtime.rs:31223
runtime.rs:6833  recv      → eval_peer_recv_prime       body at runtime.rs:31458
runtime.rs:6858  select    → eval_peer_select_prime     body at runtime.rs:32235
runtime.rs:6861  poll      → eval_poll_prime            body at runtime.rs:33232
check.rs:4049,4061,4069,4176,4188   the five inference arms — the REAL type authority
```

No `#[restricted_to]` on any of the five — measured.

## The axis prediction — the rider RE-DERIVES it

| verb | predicted `@Purity` | predicted `@Determinism` | `@Category` |
|---|---|---|---|
| `send` `try-send` | Effectful | Nondeterministic | Message |
| `recv` `select` | Effectful | Nondeterministic | Message |
| `poll` | ⚠ **derive it** | Nondeterministic | Message |

**`poll` is the one to read closely.** The other four move or consume a payload — an effect another
locus can observe. If `poll` only *reports readiness* without consuming, it may have no observable
effect at all, which would make it `Pure` + `Nondeterministic` — and that lands it in the census
beside the four `:Ambient` readers, since `effectful_by_prefix` will still say effectful. **That is
not a failure**; it is the census doing its job, and it would be its fifth entry. The rider derives
from the body and reports, exactly as home #4 did.

`is_effectful_op` now consults the registry (home #4, site 1), so whatever these five declare becomes
the answer `step_*` and `rete/purity.rs` get. For four Effectful rows nothing changes. A `Pure`
`poll` WOULD change it — flagged, not assumed.

## In scope: the Level-2 mumble this stone must fix

**`eval_poll_prime` (`runtime.rs:33232`) has NO doc comment** — verified; its four siblings each carry
one describing the tier-by-tier contract. `255.1c-taxonomy`'s design named this and routed it here:
*"since `poll'` is admitted to `:Message` on 'same locus-delivery DOING as the other four', its own
doc should say so rather than leave a reader inferring parity from the body."* Carving it forces a
`///` block anyway; this stone makes that block say the contract, not just the axes.

## Out of scope — affirmative cuts, homes named

- **`close`, `after`, `spawn-thread`, `spawn-process`** — in the table's `concurrency` row but not in
  `:Message`. `close` releases custody of a peer (`:Resource`); the others are their own homes.
- **Registering the five with the checker.** Their inference is deliberately bespoke; making them
  ordinary schemes is a type-system change, not a registration one.
- **Carving `Uuid/v4`** — it now has `:Entropic` waiting (299.3), and it is a `:wat::core::` verb, not
  this tier.

## Progress meter

60 → 65 registered production names. Five arms leave `runtime.rs`. The honest claim: **the registry
now holds rows it cannot type-check, and the record says which five and why.**
