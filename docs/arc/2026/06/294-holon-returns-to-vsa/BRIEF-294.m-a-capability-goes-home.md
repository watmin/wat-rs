# BRIEF — 294.m · a capability goes home, and the REGISTRY becomes the wall

**You are a rider, not the orchestrator. Ending your turn ENDS you** — nothing wakes you, no
notification is coming, and **a Monitor cannot wake you either**. Run every verification in the
**FOREGROUND** and block on it. Two riders on this arc have ended their turns while their own floor
ran; nothing was lost, but do not be the third.

Work in `/home/watmin/work/holon/wat-rs/`. **Do not commit, push, stash, or revert.**

## ⛔ THIS IS A TRUST BOUNDARY

This is the **last** `#wat-edn` family and the only one defending a security property. The refusal it
touches exists so that *an object-capability is obtained by being handed it over a channel, **never
forged from data***. Read the stone before touching anything.

`DESIGN-STONE-294.m-a-capability-goes-home-and-the-registry-becomes-the-wall.md`

## The two sites

```
src/capability/registry.rs:71   Tag::ns("wat-edn.cap", codec.name)   ← the EMITTER
src/edn_shim.rs:2850            if ns == "wat-edn.cap" { … }         ← the REFUSAL
```

## The change

```
#wat-edn.cap/address   →   #wat.kernel/Address
```

Measured: there is exactly **one** real capability — `ADDRESS_TYPE_PATH = ":wat::kernel::Address"`
(`src/kernel/spawn.rs:152`) — plus `test-token` at `:test::Token`, which is test-only. `wat.kernel`
already hosts `Frame`, `HandlePool`, `Location`. **No new namespace is minted.**

## ★ The load-bearing half: the namespace stops being the wall

Today `if ns == "wat-edn.cap"` **is** the refusal — a string carries the security meaning. After the
move, `wat.kernel` no longer means "capability" (it also holds ordinary data like `Frame`). **So the
refusal must ask the registry:** *is this type path a registered capability codec?*

That is arc 198's ruling verbatim: *"Ask the registry whether the key is live; **never ask a string
what it looks like.**"*

**The door already exists** — `ns_to_wat_path(ns, name)` (`src/edn_shim.rs:3033`, `pub(crate)`):

```rust
pub(crate) fn ns_to_wat_path(ns: &str, name: &str) -> String {
    format!(":{}::{}", ns.replace('.', "::"), name)
}
```

`("wat.kernel", "Address")` → `":wat::kernel::Address"` — exactly `ADDRESS_TYPE_PATH`. Use it; do not
hand-roll a second path-joiner.

## ★ And it collapses a two-key asymmetry

```rust
encode_in:          caps.iter().find(|c| c.type_path == inner.type_path)   // keys on TYPE_PATH
                    Tag::ns("wat-edn.cap", codec.name)                     // …stamps NAME
decode_capability:  fn decode_capability(name: &str, …)                    // keys on NAME
```

One registry, two keys, meeting on the security door. After this stone **`type_path` is the single key
in both directions**, and `codec.name` should become deletable. If it cannot be deleted, say why.

## ⛔ ORDERING IS THE SAFETY PROPERTY

**Emitter and refusal must change together.** Any state in which the emitter writes
`#wat.kernel/Address` while the refusal still matches `"wat-edn.cap"` is a state in which **a forged
capability parses from untrusted data.** Do not stage this as two steps, and do not leave the tree in
a half-moved state at any point you would consider reporting from.

## ★ The ward already exists — update it, keep it green

`src/edn_shim.rs`, `mod cap_decode_boundary`:

> *"the trap-door ward. A capability (`wat-edn.cap`) tag reconstructs ONLY through the trusted door;
> the general/untrusted decode path REFUSES it. **If this ever flips, the forge-hole reopens** (parsed
> data minting live capabilities). This is the regression alarm bolted onto the exact trap we fell
> through — it must never open again."*

**Update it to the new tag and keep it green.** Do not delete it, do not weaken it. ⚠ A ward that
still asserts on the OLD spelling after the rename **passes while proving nothing** — that is the
failure mode, and it is subtle enough to slip past a green floor.
`[[feedback_a_negative_control_that_can_be_kept_must_be_kept]]`

## The gate

| # | assertion |
|---|---|
| 1 | `grep -rn '"wat-edn\.[a-z]*"' src/ crates/ tests/ --include=*.rs` → **0**. This is the last family; the count goes to zero here |
| 2 | a capability emits `#wat.kernel/Address` |
| 3 | encode and decode resolve by the **same key** (`type_path`); `codec.name` deleted, or its survival justified |
| 4 | ⛔ **`cap_decode_boundary` green AND updated to the NEW tag** — untrusted path REFUSES, trusted door ACCEPTS |
| 5 | ⛔ a forged `#wat.kernel/Address` from **untrusted** data is refused, **and the refusal consults the registry**, not a namespace string. Show the code path |
| 6 | `tests/comms/probe_arc272_6a_capability_handoff.rs` green |
| 7 | floor GREEN via `scripts/floor.sh` — the **Summary line**, never a piped exit code |
| 8 | `cargo clippy --release --all-targets` → **0** |
| 9 | `grep -rnE '^[[:space:]]*#\[ignore' tests/ src/ crates/ benches/ --include=*.rs \| wc -l` → **13** |

Rows **4** and **5** are why this stone exists. Row 5 in particular: a green row 4 that passes because
the refusal now matches a *different hardcoded string* is the defect wearing the ward's clothes.

## What you report

- the `git diff` of both sites
- **the refusal's new code path, quoted** — show that it consults the registry
- what happened to `codec.name`
- the ward's diff, and proof it exercises the NEW tag
- **the measured wire string for a capability** — verbatim
- floor Summary verbatim; clippy count; `#[ignore]` count; the row-1 grep output
- honest deltas

## STOP triggers — ship nothing on that axis; report and stop.

- **STOP-1 — the registry cannot answer "is this type path a capability?" without a new mechanism.**
  Do NOT mint a registry. Name what is missing and stop — that is arc 255's territory, not this
  stone's.
- **STOP-2 — the refusal cannot be moved atomically with the emitter** (e.g. they turn out to live
  behind different feature gates or build paths). **Stop immediately.** A half-move is a forge window.
- **STOP-3 — `test-token` (`:test::Token`) behaves differently from `address` under the new scheme.**
  Name the difference; do not special-case it into working.
- **STOP-4 — the `#[ignore]` count moves off 13.**
- **STOP-5 — an unintended red. Do NOT re-run.** `scripts/floor.sh` keeps the untruncated log at
  `.floor/latest/`. Copy the failing test's **entire** stdout+stderr **verbatim** — never a summary,
  never a `| head`/`| tail` window — and name the exact assertion or match arm. There is no such thing
  as a known flake.

## Out of scope

Everything else. `.opaque`, `.holon`, `.local`, `.float` are all landed. This is the last family; when
it is green, `#wat-edn` is **zero** and arc 294's ruling — *only `#wat.*` survives* — is complete.
