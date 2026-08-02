# BRIEF — the dimension heresy is a VALUE the caller faces

> **Status: names RATIFIED — intueri cast, verdict weighed against the orchestrator's own read.**
> The ward overrode the draft's framing on the `Malformed` question and was right; the orchestrator's
> weigh added one case the ward's count missed. Both are recorded below.
>
> **This brief was rewritten once.** Its first draft made the dimension mismatch a *raise*
> (`RuntimeErrorKind::DimensionMismatch`) on the reasoning that `d` is a program constant, so a
> mismatch is a must-never-happen. **That reasoning was wrong twice** and both errors are recorded
> here so the strike is not re-derived from the dead version:
>
> - **The wire disposition was wrong.** A raise inside a service op handler kills the service for
>   every client — exactly the denial of service `DESIGN-request-malformed-input-sanitization.md`
>   exists to prevent, and which Stone 1 closed. A misbehaving client is **bounced**, not fatal.
> - **The must-never-happen classification was unearned.** It rested on grepping `from_data`, finding
>   one construction site, and declaring the world closed — without checking whether a `Value::Vector`
>   crosses a process boundary through the *generic* EDN record path rather than `bytes-vector`.
>   A grep that found one door is not proof there is one door.
>
> Builder's ruling: *"the entire check is 'are these two dims the same vec length?' — that's it. This
> is trivially measured and is not deserving of a crash but an expressive enum to be handled."* And
> the standing law it applies: *"for any options — four-questions — we deliver an enum for code to
> handle exceptions with; raise is uncatchable on purpose, a thing that must never happen."*
> Cheap detection + meaningful recovery = **handleable**.

## The ground

A program's encoding dimension is a **static, once-only constant**. `set-dim-count!` is collected from
the entry file by `config::collect_entry_file` (`src/config.rs:431`); a second setter is a load-time
`DuplicateField` error (`:432`). Every `Vector` a program *encodes* is at `EncodingCtx.dim_count`.

That makes a differing `d` rare — not impossible, and not our place to assume impossible.

## Read in order

1. `src/config.rs:431-458` — why `d` is a constant. Context only, do not edit.
2. `src/vm_registry.rs:113-139` — `EncoderRegistry::get`. It **lazily materializes** an encoder at
   whatever `dims` it is handed (`VectorManager::with_seed(dims, …)`). This is why the check in step 2
   below is vacuous. Context only, do not edit.
3. `src/types.rs` — grep `RecvOutcome`, `SendOutcome`, `CloseOutcome`, `AcceptOutcome`,
   `ConnectOutcome`. **The registration pattern you are copying**, including the parametric heads.
4. `src/runtime.rs:19323-19420` — `eval_holon_bytes_vector`, the decode.

## The work

### 1. Mint the decode outcome — `:wat::holon::VectorDecodeOutcome`

Replace `bytes-vector`'s `:Option<wat::holon::Vector>` return with a named enum. Register it beside the
outcome-wall family in `src/types.rs` (`RecvOutcome` L1591, `SendOutcome` L1635, `CloseOutcome` L1711,
`AcceptOutcome` L1806, `ConnectOutcome` L1855); mirror how `RecvOutcome` is registered.

```clojure
(:Decoded          [vector   <- :wat::holon::Vector])
(:DimensionMismatch [expected <- :wat::core::i64  got <- :wat::core::i64])
(:TruncatedHeader  [got      <- :wat::core::i64])
(:LengthMismatch   [expected <- :wat::core::i64  got <- :wat::core::i64])
(:InvalidCell      [at       <- :wat::core::i64])
```

**Why these names** (intueri, weighed and accepted):

- `VectorDecodeOutcome`, not `BytesVectorOutcome` (forces the reader to resolve a *function* name to
  parse a *type* name) and not bare `DecodeOutcome` (the family gets away with verb-only names because
  those verbs are unambiguous in their domain — "decode" is not; `:wat::holon::encode` exists).
- `Decoded` / `Combined` are past participles, matching `Sent`/`Connected`/`Accepted`. `Ok`/`Read`/
  `Vector` are generic filler this family has never used, and a variant of `VectorDecodeOutcome` named
  `Vector` reads as a field access, not a success.
- Field is **`vector`, not `v`** — a tagged-variant field is permanent API surface, not loop scope.

**Why FIVE variants and not one `Malformed[reason, at]`** — this reverses the draft, and the reasoning
is the part to keep. The draft reached for `RequestMalformed`'s String `expected`/`got` as precedent.
**It is a false friend.** Those Strings are honest precisely because the shape-mismatch space is
*open-ended* — arbitrary declared types, arbitrary paths. Byte-decode failure is the opposite: a
**closed set, already explicitly branched in the decoder's own source**. Collapsing a compile-time-known
case set into a rendered string is a state machine wearing a diagnostic's clothes, and it is what
*"a visible author-declared named variant per failure kind, never lumped"* forbids.

The concrete tell: `at` is meaningful only for `InvalidCell`. For `TruncatedHeader` there is no
position — the whole buffer is short. **A shared field honest for one member and vacuous for others is
itself the evidence the lump is wrong.**

`TruncatedHeader` carries `got` alone (the 4-byte minimum is a protocol constant; an `expected: 4` field
would be noise, but the actual length is the one datum a log wants). Do **not** reach for
`:wat::kernel::Failure` — that carrier is for process/thread-crash lifecycle payloads, not wire-byte
validation.

### 2. Close the vacuous door, in the same edit

`src/runtime.rs:19386-19393`:

```rust
if ctx.encoders.get(dim).vm.dimensions() != dim {   // ALWAYS FALSE
    return Ok(Value::Option(Arc::new(None)));
}
```

`get` builds an encoder at any `dim` it is asked for, so this can never reject — and it *creates* a
foreign-`d` encoder as a side effect of "validating," polluting a registry the one-d model says holds
one entry. Written for arc 037's per-d router; went vacuous when arc 077 retired the router.

Replace with a comparison against `ctx.dim_count`, returning the foreign-dimension variant. Carry a
comment recording why the old form was vacuous, so it is not reintroduced.

### ⛔ 2b. STOP-6 — there are FIVE `:None` returns, and one may be UNREACHABLE

The ward's verdict counted three byte-level failures. **The source has four**, plus the vacuous
dim check. Grounded by the orchestrator this session — every `:None` in `eval_holon_bytes_vector`:

| line | condition | maps to |
|---|---|---|
| `19378` | `bytes.len() < 4` | `TruncatedHeader` |
| `19384` | `bytes.len() != 4 + dim.div_ceil(4)` | `LengthMismatch` |
| `19392` | the vacuous cross-dim check | `DimensionMismatch` (step 2) |
| `19406` | invalid `0b11` two-bit pattern | `InvalidCell` |
| **`19412`** | **`cells.len() != dim`** | **UNKNOWN — prove it first** |

`19412` looks **unreachable given `19384` passed**: if the byte length is exactly `4 + dim.div_ceil(4)`,
the fill loop breaks precisely at `cells.len() == dim`. **Prove or disprove it before you map it.**

- If **unreachable** → do NOT mint a variant for it. An unreachable arm accumulates lies, and this
  stone's parent design carries that as a standing STOP. Delete the check with a comment naming the
  guarantee (`19384` subsumes it), or leave it as an explicit unreachable — report which you chose.
- If **reachable** → you have found a real case the design missed. **STOP and report it**; do not
  invent a sixth variant name. Naming is the ward's, not yours.

### 3. Mint the combine outcome — `:wat::holon::CombineOutcome`

```clojure
(:Combined          [vector   <- :wat::holon::Vector])
(:DimensionMismatch [expected <- :wat::core::i64  got <- :wat::core::i64])
```

`vector-bind` (2 wat-corpus callers), `vector-bundle` (1), `vector-blend` (1) each return `:Vector`
today and **raise** `TypeMismatch` on differing dimensions (`src/runtime.rs:19553`, `:19612`, `:19648`).
Convert each to return the outcome. Update all four wat call sites to match exhaustively.

**ONE shared enum, not three per-verb.** The `RecvOutcome`/`SendOutcome`/`TrySendOutcome` split exists
because their outcome shapes *genuinely differ* (`recv'` has a payload, `send'` does not; only
`try-send'` has `WouldBlock`). That reason does not transfer: bind/bundle/blend have **identical**
outcome spaces. Copy the *reason* for the split, not the *shape* of it — three structurally identical
types would imply a distinction that does not exist.

**`CombineOutcome`**, not `VectorOpOutcome` ("Op" is filler) or bare `VectorOutcome` (too broad, and it
reads as a sibling of `VectorDecodeOutcome` when it is not one). All three verbs' own doc comments say
the same thing in different words — compose, superposition, weighted linear combination. That is
*combine*.

**Both enums share the `DimensionMismatch` name deliberately.** It is one fact reached by two routes,
and both reduce to `[expected, got]`. `ForeignDimension` was rejected: it reads well for the wire case
but in the combine case **neither vector is foreign** — both are ordinary in-program values that simply
disagree, and importing a cross-program story there would be a lie.

### 4. holon-rs — make the two similarity paths agree

`../holon-rs/src/kernel/similarity.rs`: scalar `dot_raw` (`:99-104`) opens with
`assert_eq!(a.dimensions(), b.dimensions())`; SIMD `dot` (`:88-91`) returns `unwrap_or(0.0)`. A `0.0`
from cosine reads as *"orthogonal, unrelated"* and sails through `(f64::> … 0.9)` as a confident
no-match — a mask. Make the SIMD path assert dimensions like its twin.

**This is the ONLY edit in holon-rs.** It is a sibling repo the builder has queued for eventual
replacement by a wat implementation; invest nothing beyond making the two paths agree.

## ⛔ OUT OF SCOPE — do not touch

- **The cosine family (`src/runtime.rs:18539`, and `:18650`/`:18702`/`:18743`).** 22 wat-corpus callers,
  and its canonical form is the VSA seam's one-liner
  `(:wat::core::f64::> (:wat::holon::cosine ?a ?b) 0.9)`. Converting it to an outcome turns that into a
  match. It is held for the builder's ruling, against the `:undefined` mechanism designed in
  `DESIGN-STONE-where-admits-only-rete-ops.md`. **Leave every cosine site exactly as it is.**
- **`src/runtime.rs:9170`** — the `values_equal` Vector arm returning `Some(false)`. That is **equality
  semantics** (two vectors of different dimension are simply not equal), not a guard against an illegal
  state. Leave it.
- **The service boundary.** A foreign-`d` vector arriving in a client request is `RequestMalformed`'s
  job, in `wat/service.wat`'s `guarded-arm` slot, and belongs to the in-flight per-op-limits stone —
  the thread tier never decodes, so a Rust-side fix could not cover it anyway. Not this strike.

## Blast radius

`src/types.rs`, `src/runtime.rs`, `src/check.rs` (the `bytes-vector` signature at `:17057`), ~5 inline-wat
unit tests in `src/runtime.rs`, 4 wat-corpus call sites for bind/bundle/blend, and exactly one function
in `../holon-rs/src/kernel/similarity.rs`. `bytes-vector` has **zero** wat-corpus callers.

## Gates — run these, in this order, and report each Summary line

```
cargo build --release
cargo test --release --test lint                      # repo lints — briefs have been blind to these twice
./target/release/wat --check <each .wat you edited>   # ~0.2s per file
```

Do **not** run the full `cargo nextest run` — the orchestrator weighs the floor centrally, once.

Also run the load-order gate: a two-line `:user::main` printing
`(:wat::deporder::verify-stdlib)` must return `[]`.

## STOPs — rejection criteria, not permission slots

- **STOP-1 — never invent a name.** Every name in this brief is intueri-cast and ratified. If the work
  seems to need a name that is not written here, stop and report it — naming is a ward's act.
- **STOP-2 — no raises.** If a conversion seems to need a raise, you have found something the design did
  not anticipate: stop and report it rather than reaching for `RuntimeError`.
- **STOP-3 — if a wat call site cannot match exhaustively** without a `_` wildcard, stop. The
  `_`-arm-on-an-enum ban is doctrine whose checker rule is unbuilt, so nothing will stop you taking it.
  Taking it is a rejected strike.
- **STOP-4 — one edit in holon-rs, and only that one.**
- **STOP-5 — if the enum registration needs more than the sibling pattern's sites** (an exhaustive match
  elsewhere refusing to compile), stop and report the extra sites rather than improvising a shape.

## Do not

Do not commit. Do not push. Do not stash. Do not revert anything you did not write.
