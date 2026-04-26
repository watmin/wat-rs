# wat-rs arc 063 — `:wat::core::Bytes` ↔ hex — INSCRIPTION

**Status:** shipped 2026-04-26. One slice, one commit, ~30 minutes
of focused work.

Builder direction (2026-04-26, post-arc-062):

> "i'm adding to core to make this more ergonomic..."

> [confirms arc 062 doesn't unblock cross-process byte transmission
> since hermetic stdout is :Vec<String>; user picks Path A — small
> arc for byte-string conversion]

The minimum substrate addition: hex text encoding/decoding for
`:Bytes`. Hex chosen over CSV-decimal (5000 vs 10000 chars at
d=10000 vector size) and over base64 (no library dep, simpler
implementation). The pattern (`:wat::core::Bytes::to-X` /
`from-X`) is established here; future base64 / base32 follow the
same shape.

---

## What shipped

| Slice | Module | LOC | Tests | Status |
|-------|--------|-----|-------|--------|
| 1 | `src/runtime.rs` — `eval_bytes_to_hex` (lowercase hex, no separators, two chars per byte via `NIBBLE` lookup table); `eval_bytes_from_hex` (mixed-case decode via `decode_nibble` ascii-byte matcher; `:None` on odd length / non-hex / `0x` prefix; empty string round-trips); 2 dispatch arms. `src/check.rs` — 2 type schemes (`Bytes → String`, `String → Option<Bytes>`). `docs/USER-GUIDE.md` — 2 surface-table rows under arc 062's Bytes section. | ~110 Rust + ~5 doc | 9 new (lowercase emit, round-trip, mixed-case decode, empty-string round-trip, odd-length rejection, non-hex rejection, `0x`-prefix rejection, two arity-mismatch tests) | shipped |

**wat-rs unit-test count: 652 → 661. +9. Workspace: 0 failing.**

Build: `cargo build --release` clean. `cargo test --release`
(workspace-wide per arc 057's `default-members`): 0 failures.

---

## Architecture notes

### `to-hex` — lowercase, no separators, two-char-per-byte

`NIBBLE` is a 16-entry char lookup table; each byte produces two
chars (`NIBBLE[(b >> 4)]` + `NIBBLE[b & 0x0f]`). Single allocation
sized at `xs.len() * 2` up front; one pass through the byte buffer.
Lowercase matches Rust's `hex::encode` default and the conventions
in `git log` / file checksums / `sha256sum` output.

Determinism is structural — same input bytes always produce the
same string. Round-trip via `from-hex` recovers the original bytes
exactly.

### `from-hex` — mixed case, no separators, no `0x` prefix

`decode_nibble` is an ascii-byte matcher accepting `0-9`, `a-f`,
`A-F`. Uneven length immediately returns `:None` (can't pair into
bytes). Non-hex characters anywhere return `:None`. The `0x`
prefix isn't recognized — `0` is a hex character, but `x` isn't,
so `(from-hex "0xdead")` returns `:None` cleanly without a special
case in the parser.

Empty string is the boundary case: `(from-hex "")` returns
`:Some(empty Bytes)` because zero bytes is a valid byte sequence.
Same posture as arc 056's `from-iso8601` / arc 061's
`bytes-vector` — well-defined edge cases produce `Some` with the
canonical empty value, not `:None`.

### Why hex over base64

DESIGN decision matrix at d=10000 (Vector wire size = 2504 bytes
per arc 061's 2-bit-per-cell packing):

| Encoding | String length | Library dep | Implementation |
|----------|---------------|-------------|----------------|
| CSV decimal | ~10000 chars | none | trivial |
| Hex | 5008 chars | none | trivial (this arc) |
| Base64 | ~3344 chars | `base64` crate | trivial w/ dep |

Hex wins for the minimum-substrate stance: half the size of CSV,
no dep, debuggable in any hex viewer. Base64 ships when a consumer
surfaces a real space win (the 30% reduction over hex doesn't
matter at the substrate's current data scales).

### Naming pattern — `Bytes::to-X` / `Bytes::from-X`

Co-locates with the `:Bytes` alias from arc 062. Reads as
"operations ON Bytes" rather than "this is the hex namespace."
Future text encodings (base64, base32) get their own pairs under
`:wat::core::Bytes::*` — `Bytes::to-base64` / `Bytes::from-base64`
etc. The pattern scales cleanly.

Symmetric `to-X` / `from-X` matches the existing substrate
convention (`i64::to-string` / `string::to-i64`,
`vector-bytes` / `bytes-vector`).

---

## What this unblocks

- **Lab experiment 009 T8** — encode form F under seed_42 in one
  hermetic child, write hex to stdout (one line of `:Vec<String>`);
  parent reads the hex line, decodes via `from-hex`, deserializes
  via `bytes-vector`, runs the verifier. Universe-binding empirical
  proof becomes a clean four-step chain: encode → bytes → hex →
  transmit → hex → bytes → vector.
- **Future log-file transmission** — hex-encoded bytes ride any
  text-only log channel. The substrate's `LogEntry` payloads
  carrying byte buffers can use this pattern.
- **Future config / spec files** — wat sources that want to embed
  byte literals (e.g., test fixtures for crypto signatures) can
  encode them as hex strings and decode at runtime via
  `from-hex`.

---

## What this arc deliberately did NOT add

Reproduced from DESIGN's "What this arc does NOT add":

- **Base64 / base32.** Future encoding arcs as consumers surface.
  Pattern is established; same `Bytes::to-X` / `from-X` shape
  applies.
- **Length-prefixed framing.** Out of scope when transmitting
  multiple byte sequences over a single string channel.
- **Streaming hex encode/decode.** Out of scope; v1 is whole-buffer.
- **`from-hex-loose`** (separator tolerance) or **`from-hex-prefixed`**
  (`0x` tolerance). Future arcs if real consumers surface.

---

## The thread

- **2026-04-26 (post-arc-062)** — builder confirms cross-process
  byte transmission needs a text bridge.
- **2026-04-26 (DESIGN)** — proofs lane drafts the arc; Q1–Q6
  resolve naming, namespace, error handling, separator policy.
- **2026-04-26 (this session)** — slice 1 ships in one commit:
  to-hex / from-hex + 9 inline tests + USER-GUIDE rows + this
  INSCRIPTION.
- **Next** — experiment 009 T8 lands cleanly; future
  base64/base32 arcs follow the established pattern.

PERSEVERARE.
