# DESIGN — STONE layer-2: `atom.rs`'s vector family, and the DOOR/IMPL RULE in writing

> Builder: *"atom.rs next."* — after `layer-1` (`6af696239`) landed the collections.

## ⛔ atom.rs is NOT collections' shape, and the difference decides this stone

`layer-1` was a **move**: seven bodies went whole into `src/collection/`, which already held 50
`*_inner` helpers waiting for them. Nothing resisted; zero imports changed.

`atom.rs` is a **split**. Measured — its bodies call almost entirely into the **external `holon`
crate** (`holon::vector` ×12, `holon::primitives`, `holon::simhash`, `holon::eval`, …), with just
two `crate::` calls in the whole file. The VSA algebra already lives outside the tree. So most of
what these bodies contain is *adaptation*, and adaptation is the door's job — moving it wholesale
would be exactly as wrong as leaving it.

```
60 verbs · 2,922 file lines · 1,802 body lines · median body 24
35 bodies > 20 lines        16 already <= 10 lines (already delegates)
families: other 21/667 (from-holon alone is 300) · constructors 15/462
          similarity 14/277 · vector+bytes 6/255 · term 4/141
```

## ★ THE RULE, derived from a worked case rather than invented

Two candidate gate predicates failed before `layer-1` because they were guessed. This one is read
off `:wat::holon::bytes-vector`'s 93-line body, which has a visible seam:

```
lines  1-35   eval the arg · unwrap Value::Vec · convert each Value::u8 -> u8 · build wat errors
lines 36-93   4-byte LE header · dim = u32::from_le_bytes · div_ceil(4) · length validation
              · per-cell decode
```

> **THE DOOR converts wat values to Rust domain values and adapts errors back.
> EVERYTHING PAST THE CONVERSION IS IMPLEMENTATION.**

It classifies every hard case seen so far, which is why it is worth trusting:

| verb | verdict | why |
|---|---|---|
| `eval_program_env_intrinsic` | **stays** — pure door | calls out, maps `None` to a wat error. Nothing past the conversion. |
| `eval_length` (layer-1) | **moved** — was impl | dispatched over `MapContainer`/`StreamContainer` *after* the conversion |
| `bytes-vector` | **splits** | a codec sits past the conversion |

⚠ Still a judgement, not a regex — but a **nameable** one, which is what a gate predicate needs
before it can be written. That gate is still not this stone.

## The wave: the six vector/bytes verbs

```
  93  eval_holon_bytes_vector      :wat::holon::bytes-vector    decode — the exemplar
  51  holon_vector_bytes           :wat::holon::vector-bytes    encode — its twin
  50  eval_holon_vector_bundle     :wat::holon::vector-bundle
  29  eval_holon_vector_permute    :wat::holon::vector-permute
  20  holon_vector_blend           :wat::holon::vector-blend
  12  holon_vector_bind            :wat::holon::vector-bind     likely ALREADY pure door
 ---
 255  body lines, of which an unknown fraction is past-the-conversion
```

★ **`src/holon/outcome.rs` already holds `vector_decode_outcome_{decoded,dimension_mismatch,
truncated_header,length_mismatch,invalid_cell}`** — the codec's result constructors are already in
the impl layer, and only the codec itself is misplaced. The same shape as collections' `*_inner`
helpers, and the reason this family was chosen to go first.

## The one contract decision, pinned

**"No change needed" is a LEGAL and EXPECTED outcome per verb.** `vector-bind` at 12 lines is
probably already pure door. A wave that changes all six because it is a six-verb wave has applied
a quota, not a rule. Each verb is judged on its own body and the report says which needed nothing.

This is the opposite of `layer-1`, where all seven moved by construction.

## Destination

The codec logic extracts to a **new `src/holon/codec.rs`** — encode and decode of the
`dim:u32-LE ++ packed-cells` wire format, in domain types (`Vec<u8>` / `holon::Vector`), with no
`WatAST`, no `Value`, no `RuntimeError` in its signatures. That last constraint is the test of
whether the seam was cut in the right place: **a correctly-extracted impl does not mention wat.**

Anything else that proves to be past-the-conversion goes to the existing `src/holon/` module that
fits, or stays if no module fits and the report says why.

## Out of scope = REJECTED

- **The other 54 verbs.** `from-holon`'s 300 lines are the hardest case in the file and are
  deliberately not first.
- **No gate, lint, or ledger.** Two stones of evidence is not yet enough to write it.
- **No behaviour change.** The wire format is not touched; bytes in, bytes out, identical.
- **No `#[wat_intrinsic]` leaves `src/intrinsic/`** — the completeness gate finds verbs by scanning
  that directory.

## Calibration

Predicted 40–60 min. Comparable: `layer-1`, which was larger in lines but simpler in judgement.
