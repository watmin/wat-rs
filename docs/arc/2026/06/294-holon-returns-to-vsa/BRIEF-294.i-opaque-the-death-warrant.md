# BRIEF — 294.i Part 1 · `#wat-edn.opaque` dies; every resource goes to its own home

**You are a rider, not the orchestrator. Ending your turn ENDS you** — nothing wakes you. Run every
verification in the **FOREGROUND** and block on it.

Work in `/home/watmin/work/holon/wat-rs/`. **Do not commit, push, stash, or revert.** Leave the work
in the tree. Read `DESIGN-STONE-294.i-the-wat-edn-tags-are-annihilated.md` (sibling) in full first —
it carries the builder's ruling, the model, and the destination table.

## The model, in one line (the builder's, and it is the design)

> *"i expect these rust things to just decorate nil….. they contain no edn…. `#wat.io/Sender nil` is
> the data literal for a Sender instance."*

A resource has **no EDN representation**. The tag says *what it was*; the `nil` body says *you learn
nothing more*. That is correct and final — **do not write encoders for anything.** The defect is only
that `opaque` sits in the NAMESPACE slot where the type's HOME belongs.

## Blast radius — MEASURED, so you hunt for nothing

```
src/edn_shim.rs ................. 24 sites   ← the entire strike zone
src/capability/registry.rs:30 ...  1 site    ← a DOC COMMENT only, no code
wat-scripts/scratch-pad/probe-251-keyword-vs-colon-quoted-symbol.wat ... 1 site
golden .edn files ............... ZERO — all 3 carry `#wat-edn.holon`, none carry `.opaque`
```

## Rooms, in order

1. **`src/edn_shim.rs:3745–3860`** — the opaque arms. This is the work.
2. **`src/edn_shim.rs:3913`** — `fn opaque_nil(ns: &str, name: &str)`. Its `ns` argument is
   `"wat-edn.opaque"` at **all 14** call sites. Per-type homes make the parameter meaningless.
3. **`src/edn_shim.rs:3898`** — `tag_from_type_path`. **Already used at five other sites** in this
   same file (structs, enums, records). It is the fix for `RustOpaque`, not a new thing.
4. **`src/edn_shim.rs:3773`** — the `RustOpaque` arm, and the `if let Some(t) = types` door.

## The work

### 1 — every resource to its own home

The destination table is in the stone. Homes are **measured, not invented**: eleven come from the
`Value` variant (`Value::wat__kernel__Sender` → `#wat.kernel/Sender nil`, `Value::io__IOWriter` →
`#wat.io/IOWriter nil`, …), `lazy-seq` is a `Stream::Thunk|NativeThunk` sub-state so it takes Stream's
home, and the VSA five are all `Arc<ThreadOwnedCell<holon::X>>` inside.

⚠ **`HandlePool`'s body is NOT nil** — it carries the pool name today:
```rust
Value::wat__kernel__HandlePool { name, .. } => Tagged(ns(…,"HandlePool"), String(name))
```
Preserve it. "Everything decorates nil" is true of the MODEL and false of that one arm; flattening it
silently drops data. This is the trap in the gate.

### 2 — `RustOpaque` disappears from the wire

It is a **carrier** (`Arc<RustOpaqueInner>`), not a type, and today emits
`#wat-edn.opaque/RustOpaque "trading.cache.L1"` — the tag naming the box while the identity is demoted
to a string body. Route it through the function four lines below it:

```rust
Value::RustOpaque(inner) => OwnedValue::Tagged(tag_from_type_path(&inner.type_path), Box::new(Nil))
```

`RustOpaque` must appear in **no tag name anywhere** when you are done. It is a Rust word a wat
program must never see.

### 3 — the `None` door dies

```rust
if let Some(t) = types { if let Some(cap) = encode_capability(inner, t) { return cap; } }
```
The same value renders two ways depending on whether the caller passed a type registry, and **8 call
sites pass `None`**. Delete the door: portability is a property of the VALUE, not of what the caller
had to hand. If a caller genuinely cannot supply a `TypeEnv`, that is STOP-2 — report it.

## ★ THE KNOWN RESIDUE — two sites you CANNOT clear, and must not invent a home for

`tag_from_type_path` falls back to `.opaque` itself, twice:

```rust
Tag::try_ns(&ns, name).unwrap_or_else(|_| Tag::ns("wat-edn.opaque", "unnamed"))
Tag::try_ns("wat-edn.local", stripped).unwrap_or_else(|_| Tag::ns("wat-edn.opaque", "unnamed"))
```

That fallback replaces a type's identity with the word `"unnamed"` and **raises nothing**. Its
destination is entangled with the `.local` family, which is **Part 2 — the builder's ruling, not
yours** (*fabricate a home, or raise?*). So:

> **Leave both `unnamed` fallbacks exactly as they are.** Report them as the two remaining `.opaque`
> sites. Do NOT invent a namespace, do NOT convert them to a raise, do NOT reach for `.local`.

## The gate

| # | assertion |
|---|---|
| 1 | `grep -rn 'wat-edn\.opaque' src/ crates/ tests/ wat/ wat-scripts/` → **exactly 2**, both the `unnamed` fallbacks in `tag_from_type_path`. Any other survivor is a miss. |
| 2 | `RustOpaque` appears in **no tag name** anywhere |
| 3 | the `if let Some(t) = types` door is GONE from the `RustOpaque` arm |
| 4 | every ex-`.opaque` value emits `#wat.<home>/<Name> nil` — **except `HandlePool`, body preserved** |
| 5 | `opaque_nil`'s `ns` parameter is gone (or the fn is, if per-type homes retire it) |
| 6 | floor GREEN via `scripts/floor.sh` — read the **Summary line**, never a piped exit code |
| 7 | `cargo clippy --release --all-targets` → **0** |
| 8 | run/skip arithmetic accounted for |

## What you report

- The full `git diff` of `src/edn_shim.rs`, or every hunk that is not a tag-home change.
- The final `grep -rn 'wat-edn\.opaque'` output, verbatim (expected: the 2 fallbacks).
- Floor Summary verbatim; clippy count.
- Any test whose EXPECTED TEXT you had to change, and the before/after tag — a test asserting
  `#wat-edn.opaque/Sender` legitimately becomes `#wat.kernel/Sender`, and that is the strike working.
  A test whose **body shape** changed is not.

## STOP triggers — rejection criteria. Ship nothing; report.

- **STOP-1 — a `.opaque` member has no derivable home** in either its `Value` variant or its inner
  Rust type. Name it. Do NOT invent a namespace.
- **STOP-2 — deleting the `None` door reddens a caller** that genuinely has no `TypeEnv`. Name the
  call site verbatim. That is a finding about the 8 `None` callers, not a licence to keep the door.
- **STOP-3 — a golden `.edn` regenerates.** None should: all three carry `.holon`, not `.opaque`. If
  one moves, something outside this strike's scope changed — capture the diff and stop.
- **STOP-4 — a red you did not intend. Do NOT re-run.** `scripts/floor.sh` keeps the untruncated log
  at `.floor/latest/` including `ARM.txt`. Copy the failing test's ENTIRE stdout+stderr block
  **verbatim** — never a summary, never a `| head` window — and name the exact assertion that fired.
  There is no such thing as a known flake; a red is a red.
