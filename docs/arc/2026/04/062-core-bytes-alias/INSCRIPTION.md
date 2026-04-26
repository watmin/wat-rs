# wat-rs arc 062 — `:wat::core::Bytes` typealias — INSCRIPTION

**Status:** shipped 2026-04-26. One slice, one commit, ~20 minutes
of focused work — smallest arc to date.

Builder direction (2026-04-26, mid-arc-061 review):

> "thoughts on type alias for :Vec<u8> ?... :wat::holon::Bytes ?.."

> "you wanna rig up a small arc for this?... :wat::core::Bytes feels
> fine to me ... we should use the gaze spell for this..."

> "we get the best name results from a subagent"

The naming question went to /gaze (subagent). The ward returned a
sharp answer with Level-1 calls against the alternatives —
`:wat::holon::Bytes` lies (bytes aren't holon-domain),
`:wat::core::ByteBuffer` lies ("Buffer" implies mutability),
`:wat::core::ByteVec` mumbles. `:wat::core::Bytes` communicates;
"Bytes" is the universal name across Rust / Python / Go / Haskell.
This arc shipped what gaze recommended.

---

## What shipped

| Slice | Module | LOC | Tests | Status |
|-------|--------|-----|-------|--------|
| 1 | `src/types.rs` — `TypeDef::Alias` for `:wat::core::Bytes ≡ :Vec<u8>` registered alongside `EvalError` and the holon-domain aliases. `src/check.rs` — arc 061's `vector-bytes` / `bytes-vector` schemes updated to use `:wat::core::Bytes` on the surface (verbose `:Vec<u8>` form still works at call sites; alias resolves structurally). `src/runtime.rs` — no change (alias is checker-layer only). `docs/USER-GUIDE.md` — alias row added; arc 061 surface-table rows reference the alias. | ~25 Rust + ~5 doc | 1 new (alias-resolution round-trip showing both forms work at let-binding sites) | shipped |

**wat-rs unit-test count: 651 → 652. +1. Workspace: 0 failing.**

Build: `cargo build --release` clean. `cargo test --release`
(workspace-wide per arc 057's `default-members`): 0 failures.

---

## Architecture notes

### Why an alias

Arc 058 set the precedent — `:wat::holon::BundleResult` aliasing
`:Result<HolonAST, CapacityExceeded>` ("44 characters wide
collapsed to one named type"). Same argument for byte buffers:
`:Vec<u8>` reveals the storage representation but hides the
INTENT — these are opaque transmission/storage shapes.

`:wat::core::Bytes` lets call sites read as "give me the wire
form" instead of "give me a Vec of u8s for some reason." The
checker treats both as the same type; the alias is purely about
what the reader sees.

### Why `:wat::core::*`, not `:wat::holon::*`

Per /gaze's substrate-namespace audit. Existing alias precedents:

| Alias | Lives in | Justification |
|-------|----------|---------------|
| `BundleResult` | `:wat::holon::*` | The shape `Bundle` returns — domain-specific |
| `Holons` | `:wat::holon::*` | Vec of HolonAST — element type is holon-specific |
| `EvalError` | `:wat::core::*` | Eval failures are general — not domain-specific |

Bytes follow the EvalError pattern. They're general — vectors
today (arc 061), AEAD ciphertext tomorrow (future crypto arc),
file contents after (future IO arc). Pinning to `:wat::holon::*`
would have aged badly the moment a non-holon consumer surfaced.

### Why now, not after a second consumer

The verbose-is-honest argument applies when an alias hides real
information. `:Vec<u8>` doesn't — nobody reading
`vector-bytes : Vector → Vec<u8>` thinks "I might pick a different
Vec impl." The element type `u8` is the only signal carrying
weight, and "Bytes" carries that by convention.

The cost of waiting (rename when a second consumer surfaces) is
real but small. The cost of skipping the alias is paid every time
a reader has to guess "what's this Vec<u8> for?"

### How the alias resolves at call sites

`TypeDef::Alias` in the substrate's type registry. The check.rs
unification pass treats `:wat::core::Bytes` and `:Vec<u8>` as the
same type — no nominal protection (can't prevent passing arbitrary
`Vec<u8>` where `Bytes` is expected), structural identity. A let
binding annotated `((bs :wat::core::Bytes) ...)` can take the
return of a function declared `... -> :Vec<u8>`, and vice versa.

The single inline test exercises this directly: it binds the
result of `vector-bytes` (declared `... -> :wat::core::Bytes`)
under both `:wat::core::Bytes` and `:Vec<u8>` annotations; both
must type-check and produce equal byte buffers (substrate
determinism).

---

## Naming process — /gaze ward as subagent

This arc is the first to use a /gaze subagent specifically for a
naming question (the ward typically reviews whole files for
communication failures; this run focused it on a discrete question
with candidates). The pattern:

1. Builder surfaces a naming uncertainty during arc review.
2. Builder asks for the /gaze ward, recommending the subagent
   pattern ("we get the best name results from a subagent").
3. Implementer drafts a candidate list with substrate context
   and embeds the SKILL.md content in the subagent prompt
   (per `feedback_skill_source_in_wards` memory).
4. Subagent returns Level-1 / Level-2 / taste verdicts on each
   candidate plus a recommendation.
5. Implementer ships what the ward recommended.

The pattern worked: gaze's `:wat::core::Bytes` recommendation
came back cleanly with sharp justifications. Worth repeating
for future naming questions.

---

## What this unblocks

- **Future `:wat::crypto::*`** — AEAD inputs/outputs, signing,
  hashing all read `:Bytes` instead of `:Vec<u8>`. The signature
  reads cleanly.
- **Future `:wat::io::IOReader/read-bytes` / `IOWriter/write-bytes`** —
  byte-oriented file/stream IO.
- **Future `:wat::net::*`** — when sockets land, the wire shape
  is `Bytes`.
- **Cleaner arc 061 surface** — `vector-bytes : Vector → Bytes`
  reads as "give me the wire form" at every call site that
  picks up the alias.

---

## What this arc deliberately did NOT add

- **`:wat::core::ByteString` alternate.** Same shape, different
  name. Won't ship until a consumer surfaces a real difference.
- **Mutable byte buffers.** `:Bytes` is the values-up shape;
  mut would be a different surface (different arc, maybe never).
- **Stream / iterator over bytes.** Out of scope; build when a
  consumer needs streaming.
- **Sweep of existing `:Vec<u8>` call sites.** Optional. The alias
  resolves structurally; both forms work. Future arcs that ADD
  new ops should declare `:Bytes` on the surface; existing call
  sites that wrote `:Vec<u8>` themselves stay correct without
  migration.

---

## The thread

- **2026-04-26 (mid-arc-061 review)** — builder asks about an
  alias for the wire-format type.
- **2026-04-26 (gaze subagent)** — naming question goes to /gaze
  with substrate-context briefing. Ward returns
  `:wat::core::Bytes` with Level-1 calls against alternatives.
- **2026-04-26 (this session)** — slice 1 ships in one commit:
  alias registration + arc 061 scheme updates + 1 inline test
  + USER-GUIDE rows + this INSCRIPTION.
- **Next** — future crypto/IO/hashing/network arcs declare
  `:Bytes` on their surface from day 1.

PERSEVERARE.
