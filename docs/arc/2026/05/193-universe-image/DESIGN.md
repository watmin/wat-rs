# Arc 193 — Universe image: dump + resume

**Status:** stub opened 2026-05-13 per user direction. Captures the universe-as-data framing from arc 170 INTERSTITIAL conversation. Orthogonal to arc 191/192/194 — uses similar machinery but a different capability.
**Gates on:** arc 192 (state-preserving exec). Builds on the same serialization + injection primitives.

## Motivation

User direction 2026-05-13:
> *"/everything/ that wat is edn?.. we should just edn-ify our state and boot into a new universe with our value?..."*

Arc 192 carries SELECTED state across a universe-swap. Arc 193 generalizes: **the entire universe-at-rest is data, dumpable to EDN, loadable from EDN.** Smalltalk-style image-based persistence: save the universe; transmit it; resume from it.

A wat universe at rest is:
- Types (struct/enum/newtype/typealias declarations) — AST → EDN ✓
- Defines + defs (name → value bindings) — EDN map ✓
- Macros — AST → EDN ✓
- Dispatches — AST → EDN ✓
- All user values held by those bindings — EDN ✓

The boundary: open handles (Senders/Receivers, service handles, active thread state, mid-flight call stacks) are NOT data. They're runtime resources. The image preserves the universe's state-at-rest; runtime resources reopen / reinitialize when the image resumes.

This matches the Smalltalk-image / Common-Lisp `save-lisp-and-die` model. The wat substrate is well-positioned for it because its design choices (AST-as-data, universe-granular static typing, services-as-OS-continuity) eliminate the categories of state-management hardness that make image-based persistence brittle in other runtimes.

## The two substrate primitives

### 1. `:wat::kernel::dump-image`

```
:wat::kernel::dump-image -> :wat::core::Bytes
```

Returns EDN-encoded bytes representing the current universe-at-rest. Schema (sketch):

```edn
{:types {:user::Foo (struct ...)
         :user::Status (enum ...)
         :my::Alias (typealias ...)}
 :defines {:user::handler (fn ...)
           :user::config {...}}
 :macros {:my-when (macro ...)}
 :dispatches {:my-poly [...]}
 :runtime-values {:user::accumulator 42
                  :user::history [...]}
 :metadata {:wat-version "..."
            :dumped-at "2026-05-13T..."
            :digest "sha256:..."}}
```

What it DOESN'T include:
- Live Sender/Receiver handles
- Service handles
- Active thread states
- Mid-flight call stacks

These are reconstructed/reinitialized at resume time.

### 2. `:wat::kernel::resume-image`

```
:wat::kernel::resume-image (image :wat::core::Bytes)
  -> :wat::core::Result<:wat::core::never, :wat::kernel::ResumeError>
```

Loads an EDN-encoded universe image; freezes it as the new FrozenWorld; execs into it. Same never-return semantics as arc 191/192/194's exec primitives. Services continue.

Validation:
- Parse EDN → image schema
- Validate schema (right shape; required fields present)
- Reconstruct AST forms from schema components
- Run `startup_from_forms` on reconstructed AST
- If validation fails: return Err; old universe continues

## What this unlocks

- **Time-travel debugging.** Dump image at checkpoints; resume from any. The whole universe IS the checkpoint. Standard debug-by-replay tooling becomes substrate-supported.
- **Snapshot replication.** Dump on machine A; transmit bytes; resume on machine B. The universe migrates across hosts.
- **Signed snapshot replication.** `signed-resume-image` mirrors `signed-load!` / `eval_signed_in_frozen` — cryptographically-verified universe migration. Important for distributed scenarios.
- **Universe diffing.** `diff image-A image-B` shows what changed at the universe level. Tooling can render at any granularity: type-set diff, define-set diff, value diff.
- **Universe merging.** Selective combination of pieces from multiple images. User picks which bindings to take from which source.
- **Offline universe construction.** A tool reads + writes images outside any running wat process. The image format becomes the substrate's wire format for its own state.
- **Universe inspection without running.** A developer can `wat inspect image.edn` and read the universe's structure as text. No need to start the process.

## Slice plan (rough)

### Slice 1 — Image schema design

- Settle the EDN schema (top-level keys, nested structures)
- Version field (substrate evolves; old images need migration story)
- Digest field for integrity (sha256 of canonical EDN representation)
- /gaze pass on naming the top-level fields

### Slice 2 — `dump-image` substrate primitive

- Walks the current SymbolTable + TypeEnv + dispatch registry + active bindings
- Renders to canonical EDN
- Returns bytes

### Slice 3 — `resume-image` substrate primitive

- Parses EDN
- Validates schema
- Reconstructs AST forms
- Reuses arc 192's exec-program-with-state internals for the universe-swap

### Slice 4 — CLI integration: `wat dump-image > image.edn` / `wat resume-image < image.edn`

- Operational tooling for image manipulation outside wat programs
- `wat inspect image.edn` for human-readable rendering

### Slice 5 — Signed image variants

- `signed-dump-image` (signs the bytes)
- `signed-resume-image` (verifies signature before resuming)

### Slice 6 — INSCRIPTION + USER-GUIDE + cross-references

Closure paperwork.

## Open design questions

1. **Image versioning + migration.** When the substrate evolves and adds a new top-level field (or changes a schema shape), how do old images upgrade? Three options:
   - **(a)** Image carries `:wat-version`; resume rejects mismatched versions; user migrates externally
   - **(b)** Substrate supports schema migration at resume time (reads old schema; emits warning; resumes with best-effort)
   - **(c)** Image schema is forward-compatible by design (additive only; never break existing fields)
   - **Recommendation:** (c) for the schema design + (a) as the safety net.

2. **Open handles in the image dump.** Should `dump-image` ERROR if any live channels/threads exist, or just OMIT them? Recommendation: omit + warn (channels are runtime; image is universe-at-rest; the omission is by design).

3. **Image format: canonical EDN vs binary.** Canonical EDN is human-readable, diffable, version-controllable — but verbose. Binary is compact but opaque. Recommendation: canonical EDN as the canonical form; a separate "compact" wire format for high-volume scenarios. Both are losslessly convertible.

4. **Mid-running-thread dump.** What does `dump-image` do if threads are mid-execution? Either:
   - **(a)** Refuse-if-live (same policy as arc 191)
   - **(b)** Dump only the universe-at-rest pieces; warn about omitted thread state; user reconstructs threads on resume
   - **(c)** Cooperate with arc 194's compliant-worker pattern — signal threads to checkpoint, collect state, include in image. Closes the loop: image dump = cooperative shutdown + state collection.
   - **Recommendation:** (c) is the cleanest. Image dump becomes "cooperative migration but the destination is a file instead of a new universe."

5. **Image as program.** Can an image be EXECUTED directly (as if it were a wat program), or must it always be resumed (replacing the current universe)? Probably both: `wat run image.edn` executes the image as the program; `(:wat::kernel::resume-image bytes)` swaps current universe with the image.

6. **Cross-substrate-version compatibility.** A dump from wat-rs version X — can it be loaded by version Y? Depends on schema versioning + migration. Important for long-lived data.

## Cross-references

- Arc 170 INTERSTITIAL — the conversation that surfaced this architecture
- Arc 191 — bare exec-program
- Arc 192 — state-preserving exec (shares the freeze + injection machinery)
- Arc 194 — cooperative-migration library (slice 4 of THIS arc may use 194's pattern for cooperative dump)
- Memory `project_wat_binary_hologram.md` — the binary-as-surface framing; image-dump generalizes it (the binary is just one form; the IMAGE is the universe-at-rest as a portable artifact)
- `src/freeze.rs::startup_from_forms` — the freeze machinery resume-image reuses
- wat-edn crate — provides the canonical EDN serialization the image format builds on
- Common Lisp `save-lisp-and-die` — historical precedent
- Smalltalk image — closest cultural reference

## Why this matters

Arc 191 + 192 + 194 give you hot reload within a running OS process. Arc 193 lets the universe LEAVE the process — to disk, to network, to another machine. The universe becomes a portable artifact.

Combined: a running wat universe can dump its image at any checkpoint, hand it to a tool that inspects/edits/diffs/merges it, the tool produces a new image, the universe resumes from the new image. **The universe just gave a program permission to edit its mind**, then took the result back.

That's the deepest form of homoiconicity: not just code-is-data but RUNTIME-STATE-IS-DATA, including the code. The image format IS the substrate's wire format for its own state.
