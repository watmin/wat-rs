# Arc 063 — `:wat::core::Bytes` ↔ hex string

**Status:** shipped 2026-04-26. See `INSCRIPTION.md` for the
canonical post-ship record. Implementation matched the DESIGN
verbatim — Q1–Q6 all settled before code was written.

**Predecessor:** arc 061 (vector-portability) shipped
`vector-bytes` / `bytes-vector`; arc 062 (Bytes alias) shipped the
`:wat::core::Bytes` name. The wire-shape primitive exists. What's
missing is a way to transport `:Bytes` through string-based
channels — hermetic stdout/stdin, log files, network protocols.

**Consumer:** experiment 009 T8 needs to transmit a `:wat::holon::Vector`'s
bytes from one hermetic child (encoded under seed_42) through the
parent and into a verifier reasoning about a different universe.
Hermetic stdout is `:Vec<String>`; raw `:Bytes` cannot ride that
channel today. Hex is the conventional binary-in-text encoding —
1:2 byte-to-character mapping, universally readable in dumps,
trivially cheap.

Builder direction (2026-04-26, mid-arc-061 review):

> "thoughts on type alias for :Vec<u8> ?... :wat::holon::Bytes ?.."
>
> [arc 062 ships :wat::core::Bytes typealias]
>
> "i'm adding to core to make this more ergonomic..."
>
> [confirms arc 062 doesn't unblock cross-process byte transmission
> since hermetic stdout is :Vec<String>; user picks Path A — small
> arc for byte-string conversion]

The minimum substrate addition: hex text encoding/decoding for
`:Bytes`. Hex chosen over CSV-decimal (5000 vs 10000 chars at
d=10000 vector size) and over base64 (no library dep, simpler
implementation).

---

## What's already there (no change needed)

| Surface | Status |
|---------|--------|
| `:wat::core::Bytes` (typealias for `:Vec<u8>`) | shipped (arc 062) |
| `:wat::core::u8` (range-checked cast from i64) | shipped |
| `:wat::holon::vector-bytes` (Vector → Bytes) | shipped (arc 061) |
| `:wat::holon::bytes-vector` (Bytes → Option<Vector>) | shipped (arc 061) |
| `:wat::core::string::concat`, `string::join`, `string::split` | shipped |

The substrate already has the byte-handling primitives. What's
missing is the text-bridge.

## What's missing (this arc)

| Op | Signature |
|----|-----------|
| `:wat::core::Bytes::to-hex` | `:wat::core::Bytes → :String` (hex digits, lowercase, no separators) |
| `:wat::core::Bytes::from-hex` | `:String → :Option<wat::core::Bytes>` (`:None` on bad input) |

Two additions. Pure additions; non-breaking.

---

## Decisions to resolve

### Q1 — Hex case (lowercase vs uppercase vs accept-both)

`to-hex` should output a single canonical case for determinism
(round-trip via `from-hex` produces the same Bytes; printing the
result twice yields the same String).

`from-hex` should accept both cases for ergonomics — humans paste
hex from various sources and shouldn't have to normalize first.

**Recommended:**
- `to-hex` → lowercase only (matches Rust's `hex::encode` default,
  matches conventions in `git log` / file checksums).
- `from-hex` → accepts mixed case (a-f and A-F both decode).

### Q2 — Separator support

Some hex formats use spaces or colons as separators between bytes
(e.g., MAC addresses: `aa:bb:cc:dd:ee:ff`).

**Recommended: NO separator support in v1.** `to-hex` emits raw
hex (no separators); `from-hex` requires raw hex (rejects with
`:None` if separators present). If a consumer surfaces a need for
separator-tolerant decoding, a future arc adds `from-hex-loose` or
similar.

The test for "raw" hex is: total length is 2 × bytes.length, and
every character is in `[0-9a-fA-F]`.

### Q3 — Error reporting on bad input

`from-hex` returns `:Option<Bytes>`. `:None` on:
- Odd length (can't parse pairs)
- Non-hex character anywhere
- Empty string → `:None` or `:Some(empty Bytes)`?

**Recommended:** empty string → `:Some(empty)`. It's a valid
zero-length byte sequence. Round-trip test: `to-hex(empty Bytes)` →
`""`; `from-hex("")` → `:Some(empty Bytes)`.

This matches arc 056's `from-iso8601` posture (well-defined edge
cases produce `Some` with the canonical empty value, not `:None`).

### Q4 — Naming: `to-hex` / `from-hex` vs `encode-hex` / `decode-hex` vs other

Looked at other substrate naming patterns:
- `:wat::core::i64::to-string` / `:wat::core::string::to-i64`
- `:wat::holon::vector-bytes` / `:wat::holon::bytes-vector`
- Both follow the `to-X` / `from-X` shape.

**Recommended:** `to-hex` (output verb starting with "to") and
`from-hex` (input verb starting with "from"). Symmetric pair.
Mirrors existing convention.

Alternative: `Bytes::hex` / `hex::Bytes` (more terse, less
explicit on direction). Rejected for clarity.

### Q5 — Namespace placement

Two options:
- `:wat::core::Bytes::to-hex` / `:wat::core::Bytes::from-hex`
- `:wat::core::hex::encode` / `:wat::core::hex::decode`

**Recommended:** `:wat::core::Bytes::to-hex` and
`:wat::core::Bytes::from-hex`. Co-locates with the `:Bytes` alias
from arc 062. Reads as "operations ON Bytes" rather than "this is
the hex namespace."

If a future arc adds more text encodings (base64, base32), each
gets its own pair under `:wat::core::Bytes::*`:
- `Bytes::to-base64` / `Bytes::from-base64`
- `Bytes::to-base32` / `Bytes::from-base32`

This pattern scales cleanly.

### Q6 — Should `from-hex` be tolerant of leading "0x" prefix?

Hex literals in many languages use `0x` prefix. `0xdeadbeef` reads
as a hex value.

**Recommended: NO `0x` tolerance in v1.** `from-hex` accepts pure
hex characters only. A "0x" prefix triggers `:None` (the "0" is
hex-valid but "x" is not).

If a consumer surfaces a need (e.g., parsing config files that use
0x-prefixed values), a future arc adds `from-hex-prefixed` or
similar.

---

## What ships

One slice. Pure additions. Existing surface unchanged.

- `:wat::core::Bytes::to-hex` — new primitive in `runtime.rs`,
  scheme registration in `check.rs`
- `:wat::core::Bytes::from-hex` — same, returning `:Option<Bytes>`
- Tests inline in `src/runtime.rs::mod tests` (matching arcs
  058/059/060/061/062 convention):
  - Round-trip: `from-hex(to-hex(b)) == :Some(b)` for sample bytes
  - Empty round-trip
  - `from-hex` rejects odd length
  - `from-hex` rejects non-hex characters
  - `from-hex` accepts mixed case
  - `to-hex` is lowercase
- `docs/USER-GUIDE.md` — add the two surface table rows under
  arc 062's Bytes section

Estimated effort: ~50 lines Rust + ~30 lines tests + doc updates.
Single commit. Mirrors arcs 058/059/060/061/062's small-addition
shape. Probably the smallest arc to date by Rust LOC.

---

## Open questions

- **Base64 / base32**: future encoding arcs as consumers surface.
  Pattern is established by this arc; adding more text encodings
  follows the same `:wat::core::Bytes::to-X` / `from-X` shape.
- **Length-prefixed framing**: when transmitting MULTIPLE byte
  sequences over a single string channel, framing becomes important.
  Out of scope; build when needed.
- **Streaming hex encode/decode**: for very large `Bytes`, a
  streaming variant might be valuable. Out of scope; v1 is
  whole-buffer.

## Slices

One slice. Single commit. Pattern matches arcs 058/059/060/061/062.

## Consumer follow-up

After this arc lands, experiment 009 T8 lands cleanly:

```scheme
;; Child A (seed 42) — encode form, write hex to stdout
(:user::main ...
  (:wat::core::let*
    (((form ...) ...)
     ((v :wat::holon::Vector) (:wat::holon::encode form))
     ((bytes :wat::core::Bytes) (:wat::holon::vector-bytes v))
     ((hex :String) (:wat::core::Bytes::to-hex bytes)))
    (:wat::io::IOWriter/print stdout hex)))

;; Parent — read hex from child's stdout, decode, deserialize
((hex-line :String) (... extract from r-a stdout ...))
((bytes-opt :Option<wat::core::Bytes>) (:wat::core::Bytes::from-hex hex-line))
((bytes :wat::core::Bytes) (... unwrap Some ...))
((v-imported :Option<wat::holon::Vector>) (:wat::holon::bytes-vector bytes))
;; ... compare to local encoding for universe-binding demonstration ...
```

T8's universe-binding empirical proof becomes a clean four-step
chain: encode → bytes → hex → transmit → hex → bytes → vector.
