# Arc 062 — `:wat::core::Bytes` typealias

**Status:** shipped 2026-04-26. See `INSCRIPTION.md` for the
canonical post-ship record. Implementation matched the sketch
verbatim — first arc to ship exactly as designed because /gaze
returned a sharp answer with full justification before any code
was written.
**Predecessor:** arc 061 (vector portability) — shipped two byte-buffer
ops (`vector-bytes` / `bytes-vector`) using the verbose `:Vec<u8>`
shape; this arc adds the substrate-general alias before a second
consumer lands.

**Driver:** builder direction during arc 061 review:

> "thoughts on type alias for :Vec<u8> ?... :wat::holon::Bytes ?.."

> "you wanna rig up a small arc for this?... :wat::core::Bytes feels
> fine to me ... we should use the gaze spell for this..."

> "we get the best name results from a subagent"

The naming question went to /gaze (subagent). The ward converged on
`:wat::core::Bytes` with sharp Level-1 calls against alternatives.

---

## /gaze findings (subagent)

| Candidate | Verdict |
|-----------|---------|
| `:wat::core::Bytes` | **Communicates.** "Bytes" is the universal name across Rust (`bytes::Bytes`), Python, Erlang, Go (`[]byte`), Haskell (`ByteString`). Reader arrives with full context. |
| `:wat::core::ByteBuffer` | **Level 1 lie.** "Buffer" implies mutability/position/limit/capacity (Java's `ByteBuffer`). Promises more than `Vec<u8>` delivers. |
| `:wat::core::ByteVec` | **Level 2 mumble.** Cosmetic rename of `Vec<u8>`; saves zero cognitive load over the verbose form. |
| `:wat::holon::Bytes` | **Level 1 lie.** Bytes are not holon-domain. The precedent justifies `holon::` aliases by domain specificity — `BundleResult` IS the shape Bundle returns; `Holons` IS a list of holons. Bytes carry vectors today, AEAD ciphertext tomorrow, file contents after. Pinning to `holon::` ages badly. |
| `:wat::io::Bytes` | **Level 1 lie.** IO is one consumer, not the home. Crypto and hashing aren't IO. |
| No alias (keep `:Vec<u8>`) | **Weak verbose-is-honest.** `Vec<u8>` reveals storage representation, but nobody reading `vector-bytes : Vector → Vec<u8>` thinks "I might choose a different Vec impl." Threshold: add the alias the moment a second primitive returns/consumes byte buffers. Arc 061 alone is borderline; the alias prepays a debt that's about to come due. |

The ward's recommendation: `:wat::core::Bytes`, **add it now**.

---

## What ships

One slice. Pure addition; non-breaking. Existing `:Vec<u8>` call
sites unchanged because typealiases resolve structurally —
`:wat::core::Bytes` and `:Vec<u8>` are the same type at the checker
layer (same posture as arc 032's `:wat::holon::BundleResult` next
to `:Result<HolonAST, CapacityExceeded>`).

### `src/types.rs`

Register the alias in `register_builtin_types`:

```rust
env.register_builtin(TypeDef::Alias(AliasDef {
    name: ":wat::core::Bytes".into(),
    type_params: vec![],
    expr: TypeExpr::Parametric {
        head: "Vec".into(),
        args: vec![TypeExpr::Path(":u8".into())],
    },
}));
```

### `src/check.rs`

Update arc 061's two op schemes to use the alias on the surface:

- `:wat::holon::vector-bytes` — `Vector → Bytes` (was `Vector → Vec<u8>`)
- `:wat::holon::bytes-vector` — `Bytes → Option<Vector>` (was `Vec<u8> → Option<Vector>`)

Both forms remain valid at call sites because the alias resolves
structurally.

### `src/runtime.rs`

No runtime change. The Value variant `Value::Vec(Arc<Vec<Value>>)`
is the underlying storage for both `:Vec<u8>` and `:wat::core::Bytes`
— the alias is checker-layer only.

### `docs/USER-GUIDE.md`

- Update arc 061's two surface-table rows to reference `:wat::core::Bytes`.
- Add a row for the alias itself (analogous to the `BundleResult`
  alias row).

### Tests

A round-trip alias-resolution test inline (mirrors arc 032's pattern):

```scheme
;; Calling vector-bytes and feeding into bytes-vector is the same
;; pipeline; the alias should not interfere with type-checking
;; whether the binding annotation uses :Vec<u8> or :wat::core::Bytes.
(:wat::core::let*
  (((bs :wat::core::Bytes) (:wat::holon::vector-bytes <vec>))  ;; alias form
   ((bs2 :Vec<u8>) (:wat::holon::vector-bytes <vec>))           ;; verbose form
   ((maybe-v1 :Option<wat::holon::Vector>) (:wat::holon::bytes-vector bs))
   ((maybe-v2 :Option<wat::holon::Vector>) (:wat::holon::bytes-vector bs2)))
  ...)
```

Existing arc 061 tests already cover the round-trip behavior; the
new test specifically exercises that both type annotations work.

---

## Decisions resolved

### Q1 — Where it lives

`:wat::core::*`. Per gaze's substrate-namespace audit: `:wat::core::*`
houses substrate-general types (HashMap, Vec, HashSet, EvalError);
domain-specific aliases (`BundleResult`, `Holons`) live in domain
namespaces. Bytes are general — they go in core.

### Q2 — Add now or wait for a second consumer?

Now. Arc 061's two ops + likely-imminent arcs (crypto, file IO,
hashing, network) all read or produce byte buffers. Adding the alias
prepays a debt that's about to come due. Cost is small (one
TypeDef::Alias entry + 2 surface-table updates).

### Q3 — Migrate existing `:Vec<u8>` call sites?

Optional. Aliases resolve structurally; both forms work. Update
arc 061's two op schemes to use the alias on the surface (so the
canonical form in the USER-GUIDE reads cleanly), but no need to
sweep call sites that wrote `:Vec<u8>` themselves.

### Q4 — Pluralization (`Byte` vs `Bytes`)?

`Bytes`. Matches every adjacent ecosystem (`bytes::Bytes`, Python's
`bytes`, Haskell's `ByteString`'s plural-implied content). The
type IS a sequence of bytes, plural shape, plural name.

---

## What this arc does NOT add

- **A `:wat::core::ByteString` alternate.** Same shape, different
  name. Won't ship until a consumer surfaces a real difference.
- **Mutable byte buffers.** `:Bytes` is the values-up shape; mut
  would be a different surface.
- **Stream / iterator over bytes.** Out of scope; build when a
  consumer needs streaming.
- **Crypto / IO consumers.** Different arcs; this one just
  installs the alias.

---

## Implementation sketch

```
src/types.rs:    +10 LOC  (1 TypeDef::Alias entry)
src/check.rs:     +0 LOC  (existing schemes can switch to :Bytes
                            but it's cosmetic; both forms work)
src/runtime.rs:   +0 LOC  (alias is checker-layer only)
inline tests:    ~30 LOC  (alias-resolution round-trip)
docs/arc/.../INSCRIPTION.md:  post-ship
docs/USER-GUIDE.md:           +6 LOC (1 alias row + 2 surface-row tweaks)
```

**Estimated cost:** ~50 LOC. **~30 minutes** of focused work.
Smallest arc to date; matches arc 020/058/059's small-shape pattern.

---

## What this unblocks

- **Future `:wat::crypto::*`** — AEAD inputs/outputs, signing, hashing
  all read `:Bytes` instead of `:Vec<u8>`.
- **Future `:wat::io::IOReader/read-bytes` / `IOWriter/write-bytes`** —
  byte-oriented file/stream IO surfaces a clean type.
- **Future `:wat::net::*`** — when sockets land, the wire shape is
  `Bytes`.
- **Cleaner arc 061 surface** — `vector-bytes : Vector → Bytes`
  reads as "give me the wire form" instead of "give me a Vec of u8s
  for some reason."

---

PERSEVERARE.
