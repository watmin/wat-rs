# BRIEF — the dimension heresy screams by its own tongue

The program's encoding dimension is a **static, once-only constant**. `set-dim-count!` is collected
from the entry file by `config::collect_entry_file` (`src/config.rs:431`) and a second setter is a
load-time `DuplicateField` error (`:432`). Every `Vector` a program encodes is at
`EncodingCtx.dim_count`. Two vectors at different `d` are therefore an **illegal state**, not a data
condition — and the substrate must refuse it loudly, naming both numbers, so whoever wired two
programs at different dim-counts sees it immediately instead of receiving a plausible-looking answer.

Today it does neither: the one door where a foreign `d` can enter is guarded by a **vacuous** check,
and the four real guards raise a lumped `TypeMismatch` whose payload is the static string
`"mismatched-dim Vector pair"` — the numbers are lost.

## Read in order

1. `src/config.rs:431-458` — `set-dim-count!`, entry-file-collected, `DuplicateField` on a second
   setter. **This is why mismatch is illegal, not merely unusual.** Context only, do not edit.
2. `src/vm_registry.rs:113-139` — `EncoderRegistry::get`. It **lazily materializes** an encoder at
   whatever `dims` it is handed (`VectorManager::with_seed(dims, …)`). Context only, do not edit.
3. `src/value/signal.rs:199-203` — `RuntimeErrorKind::IntegerOverflow`. **Your template.**
4. `src/value/signal.rs:525-527` — its `Display` arm. **Your template.**
5. `src/types.rs:2208-2211` — its registration entry, and `ArityMismatch` at `:2201-2204` for the
   `op`/`expected`/`got` field naming. **Your template.**

## The work

### 1. Mint the variant

`RuntimeErrorKind::DimensionMismatch { op: String, expected: i64, got: i64 }` — mirror
`IntegerOverflow`'s shape exactly across the three template sites. Field naming follows
`ArityMismatch` (`op`/`expected`/`got`), not `IntegerOverflow`'s `a`/`b`.

`Display` arm, in the register of the surrounding messages:

```rust
RuntimeErrorKind::DimensionMismatch { op, expected, got } => {
    write!(f, "{}{} got a Vector at d={} but this program's dim-count is {}", prefix, op, got, expected)
}
```

### 2. Convert the four real guards

Each currently raises `TypeMismatch` with `ValueSnapshot::unavailable("mismatched-dim …")`. Replace
with `DimensionMismatch` carrying both dims. Keep each site's existing `list_span` and its `op` string.

| site | fn |
|---|---|
| `src/runtime.rs:18539` | the cosine-family normalize (`Vector`/`Vector` arm) |
| `src/runtime.rs:19553` | `:wat::holon::vector-bind` |
| `src/runtime.rs:19612` | `:wat::holon::vector-bundle` (the loop — `expected` is `d`, `got` is `v.dimensions()`) |
| `src/runtime.rs:19648` | `:wat::holon::vector-blend` |

### 3. ★ Close the one door — and make it SCREAM

`src/runtime.rs:19386-19393`, inside the bytes→Vector decode. The present check is:

```rust
if ctx.encoders.get(dim).vm.dimensions() != dim {
    return Ok(Value::Option(Arc::new(None)));
}
```

`get` builds an encoder at any `dim` it is asked for, so the predicate is **always false** — it can
never reject, and it *creates* a foreign-`d` encoder as a side effect of "validating." Written for arc
037's per-d router; went vacuous when arc 077 retired the router.

Replace with a check against the program's declared dim, and **raise** — do not return `:None`:

```rust
let ctx = require_encoding_ctx(OP, sym, list_span)?;
if dim != ctx.dim_count {
    return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::DimensionMismatch {
        op: OP.into(),
        expected: ctx.dim_count as i64,
        got: dim as i64,
    }).into());
}
```

Carry a comment recording *why* this is the one door (every other Vector is encoded at `dim_count`;
this decode is the only path that builds one from foreign bytes) and *why* the old form was vacuous.

### 4. holon-rs — make the two similarity paths agree

`../holon-rs/src/kernel/similarity.rs`. The scalar `dot_raw` (`:99-104`) opens with
`assert_eq!(a.dimensions(), b.dimensions())`; the SIMD `dot` (`:88-91`) returns
`i8::dot(...).unwrap_or(0.0)`. A `0.0` from cosine reads as *"orthogonal, unrelated"* and sails
through `(f64::> … 0.9)` as a confident no-match — a mask over a must-never-happen.

Make the SIMD path fail the same way its twin does: assert dimension equality before the SIMD call, so
both builds treat mismatch as the must-never-happen it is. **This is the ONLY edit in holon-rs.**

## Blast radius

`src/value/signal.rs`, `src/types.rs`, `src/runtime.rs` (5 sites), and exactly one function in
`../holon-rs/src/kernel/similarity.rs`. No new types beyond the one variant. No wat corpus changes.

## Gates — run these, in this order

```
cargo build --release
cargo test --release --test lint          # ← repo lints; briefs have been blind to these twice
cargo test --release --test wat_holon     # (or the nearest holon-named target; find it, run it)
```

Do **not** run the full `cargo nextest run` — the orchestrator weighs the floor centrally, once.
Report the exact commands you ran and their Summary lines.

## STOPs — rejection criteria, not permission slots

- **STOP-1 — `src/runtime.rs:9170` is NOT in scope.** That `values_equal` arm returns `Some(false)` for
  different-`d` vectors. That is **equality semantics** — two vectors of different dimension are simply
  not equal — not a guard against an illegal state. Leave it exactly as it is.
- **STOP-2 — if step 3 turns tests red, do NOT soften it back to `:None`.** The red is the point: a test
  that depended on a foreign-`d` vector decoding silently is a heretic identifying itself. Report each
  one with its name and assertion; the orchestrator dispositions them.
- **STOP-3 — the other `:None` returns in that decode fn stay.** Short byte string, wrong data length,
  invalid 2-bit cell: those are genuine malformed-input from untrusted bytes, a different failure kind
  from an illegal dimension. Do not convert them.
- **STOP-4 — one edit in holon-rs, and only that one.** It is a sibling repo queued for eventual
  replacement; invest nothing beyond making the two paths agree.
- **STOP-5 — if `DimensionMismatch` requires touching more than the three template sites** (an
  exhaustive match somewhere else refusing to compile), stop and report the extra sites rather than
  improvising a shape.

## Do not

Do not commit. Do not push. Do not stash. Do not revert anything you did not write.
