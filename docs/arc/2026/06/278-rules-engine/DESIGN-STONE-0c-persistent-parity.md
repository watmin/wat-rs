# DESIGN — Stone 0c: full PersistentVector op-parity (the transform + sequence ops)

> Arc 278 stone 0, part c — close ALL remaining persistent-collection gaps in one comprehensive pass, so
> the engine (and any wat code) never trips on a missing persistent op again. Surfaced when stone 1b's
> `compile` couldn't `foldl` over a PersistentVector of rules. Builder: "do full parity now, not piecemeal —
> we're starting the work." Mechanical mirror of the std `Vec` transform ops (the 0a/0b pattern).

## The gap (grounded against the dispatch + eval fns)

- **PersistentMap = ALREADY at full parity** with std `HashMap` (length/empty?/contains-key?/get/assoc/
  dissoc/keys/values — 0a). Nothing to add. (Map iteration is `keys`/`vals`→Vec→`foldl`, exactly as std
  HashMap + as `render-dag` does.)
- **PersistentVector has the ACCESSOR ops** (length/empty?/contains?/get/conj/first/second/third/rest —
  0a/0b) but is MISSING the **transform + sequence ops**, all `eval_vec_*` in `src/collection/transform.rs`,
  all currently std-`Vec`-only:

  | op | semantics on a PersistentVector | returns |
  |---|---|---|
  | `map`     | apply fn to each element            | PersistentVector (type-preserving) |
  | `filter`  | keep elements where pred holds      | PersistentVector |
  | `foldl`   | left fold with acc + fn             | the accumulator (any type) |
  | `foldr`   | right fold                          | the accumulator |
  | `concat`  | append two vectors                  | PersistentVector |
  | `reverse` | reverse order                       | PersistentVector |
  | `take`    | first n                             | PersistentVector |
  | `drop`    | all but first n                     | PersistentVector |

## The work — mirror the std `Vec` arm for each (8 ops)

Each `eval_vec_<op>` in `src/collection/transform.rs` currently matches `Value::Vec`. Add a
`Value::wat__core__PersistentVector(pv)` arm to each, mirroring the std logic over `pv.iter()` and building
the result with `rpds::VectorSync` (`.push_back`) where the op returns a vector. Type-preserving: a
PersistentVector in → a PersistentVector out (for map/filter/concat/reverse/take/drop); foldl/foldr return
the accumulator value. The GENERIC heads (`:wat::core::foldl`/`map`/`filter`/`foldr`/`concat`/`reverse`/
`take`/`drop`) already dispatch through these fns, so adding the arm makes `(foldl f init pv)` etc. work.
(Add `:wat::core::PersistentVector/<op>` qualified dispatch heads too, mirroring 0a/0b's per-type heads, for
symmetry — optional if the generic dispatch covers the engine's needs; confirm against how Vector/<op> heads
are wired.)

## Checker (collection/infer.rs)
Mirror the std `Vector<T>` arms for these ops onto `PersistentVector<T>` in their infer fns (e.g.
`infer_map`/`infer_filter`/`infer_foldl`/…), type-preserving (PersistentVector<T> → PersistentVector<U> for
map; → PersistentVector<T> for filter/reverse/take/drop/concat; → Acc for foldl/foldr). Mirror whatever
infer arms exist for the std transform ops.

## Proof (FM-2-bis — RED at HEAD)

`tests/probe_arc278_0c_persistent_parity.rs` (RED, un-ignore on green): build a PersistentVector, run it
through EACH op and assert results — `foldl` (+ over a PersistentVector), `map`, `filter`, `reverse`,
`take`, `drop`, `concat`. RED at HEAD: `foldl`/`map`/etc. on a PersistentVector error (no dispatch arm).

## Out of scope
- PersistentMap transform ops beyond std HashMap parity (std HashMap has none; map iteration is keys/vals).
- The formal `:Seq`/`:Map` defprotocol (arc 285, Layer 2) — still deferred; this stone is Layer-1 op-parity,
  which IS the "consistent access" the engine needs. (Build the formal protocol only when generic-typed
  `[s <- :Seq]` code forces it.)

## Four questions
- **Obvious?** YES — "PersistentVector gets the same transform ops as Vector."
- **Simple?** YES — a mechanical mirror of 8 existing std-Vec arms.
- **Honest?** YES — true op-parity (the probe runs each op); no crutch (no to-vec-everywhere).
- **Good UX?** YES — consistent access: `foldl`/`map`/etc. work on every collection; the engine stops tripping.

## Blast radius
`src/collection/transform.rs` (8 PersistentVector arms) · `src/collection/infer.rs` (mirror the infer arms) ·
maybe `src/runtime.rs` (PersistentVector/<op> qualified heads, if added) · the probe. NO behavior change to
std Vec. No git in the worker.
