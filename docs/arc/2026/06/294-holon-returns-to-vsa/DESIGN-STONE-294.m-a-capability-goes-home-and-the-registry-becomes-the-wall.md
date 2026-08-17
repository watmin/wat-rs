# DESIGN STONE — 294.m · a capability goes home, and the REGISTRY becomes the wall

**Builder's ruling, 2026-08-16:** *"capabilities are a kernel namespace thing?"* — yes.

```
#wat-edn.cap/address   →   #wat.kernel/Address
```

⛔ **This is the LAST `#wat-edn` family and the ONLY one on a trust boundary. It is two sites and it
is the one where a mistake matters most.** Draw it alone; do not fold it in with 294.l.

## The two sites

```
src/capability/registry.rs:71   Tag::ns("wat-edn.cap", codec.name)   ← the EMITTER
src/edn_shim.rs:2839            if ns == "wat-edn.cap" { … }         ← the REFUSAL
```

The refusal's own comment states the property being defended:

> *"a `wat-edn.cap` tag is REFUSED: an object-capability is obtained by being handed it over a
> channel, **never forged from data** (ocap unforgeability + transfer-only)."*

## Measured: there is exactly ONE real capability

```
name: "address"      type_path: ADDRESS_TYPE_PATH = ":wat::kernel::Address"   ← the real one
name: "test-token"   type_path: ":test::Token"                                ← test-only
```

`:wat::kernel::Address` → `#wat.kernel/Address`, which is the home `tag_from_type_path` already
derives, in a namespace already hosting `Frame`, `HandlePool`, `Location`. **No new namespace is
minted by this stone.**

## ★ THE REAL FINDING — one registry, TWO keys, meeting on the security door

```rust
// ENCODE — looks up by TYPE_PATH …
let codec = caps.iter().find(|c| c.type_path == inner.type_path)?;
// … but stamps the NAME
Some(Tagged(Tag::ns("wat-edn.cap", codec.name), body))

// DECODE — looks up by NAME
pub fn decode_capability(name: &str, body: &OwnedValue, types: &TypeEnv) -> …
```

`CapCodec` carries **both** `type_path` and `name`. Encode resolves by one and writes the other;
decode resolves by the one that was written. Today's wire tag is therefore
`#wat-edn.cap/address` — a lowercase codec nickname, not the type.

★ **Fourth instance in this arc of one concept implemented twice** (`holon_to_watast` vs
`from_holon_item`; `watast_to_holon` vs `to_holon_inner`; `tag_from_type_path` vs `struct_tag_for`;
and this) — **and the only one sitting on a trust boundary.**

Moving to `#wat.kernel/Address` makes **`type_path` the single key in both directions**, and
`codec.name` becomes deletable. The rename is not cosmetic; it collapses the asymmetry.

## ★ AND THE NAMESPACE STOPS BEING THE WALL — this is the load-bearing half

Today `if ns == "wat-edn.cap"` **is** the refusal. The namespace string carries the security meaning.
Once a capability wears its real home, `wat.kernel` no longer means "capability" — it also holds
`Frame` and `Location`, which are ordinary data. **So the refusal must ask the REGISTRY:**

> *is this type path a registered capability codec?*

Which is arc 198's ruling, verbatim from
`255-builtin-registry/NOTE-a-capability-declaration-cannot-be-verified-to-name-anything.md`:

> *"And it must NOT be a name-shape test… **Ask the registry whether the key is live; never ask a
> string what it looks like.**"*

**The encoder already does this** (`find(|c| c.type_path == …)`). Only the refusal asks a string. This
stone brings the two into agreement, which is the same shape as the rest of the arc — with the
difference that here the disagreement is a *forge surface*.

## ⛔ ORDERING IS THE SAFETY PROPERTY

The emitter and the refusal **must move in the same commit.** Any interval where the emitter writes
`#wat.kernel/Address` while the refusal still matches `"wat-edn.cap"` is an interval in which a
**forged capability parses from untrusted data.** This is not a style note; it is the whole risk of
the stone, and it is why 294.m is drawn separately from 294.l.

## ★ The negative control ALREADY EXISTS — keep it green

`src/edn_shim.rs`, `mod cap_decode_boundary`:

> *"Arc 272 6a-i / 6c.2 — the trap-door ward. A capability (`wat-edn.cap`) tag reconstructs ONLY
> through the trusted door; the general/untrusted decode path REFUSES it. **If this ever flips, the
> forge-hole reopens** (parsed data minting live capabilities). This is the regression alarm bolted
> onto the exact trap we fell through — it must never open again."*

It is written, it is named, and it must be **updated to the new tag and stay green**. Do not delete it,
do not weaken it, do not let it pass by asserting a stale spelling — a ward that tests the OLD tag
after the rename is a ward that proves nothing.
`[[feedback_a_negative_control_that_can_be_kept_must_be_kept]]`

## The four questions — flat

**Obvious? YES.** A capability is a `:wat::kernel::Address`, so it tags as `#wat.kernel/Address`.
Today's `#wat-edn.cap/address` names a Cargo crate and a nickname.

**Simple? YES.** Two sites; one registry key instead of two; one field (`codec.name`) deleted. Net
negative.

**Honest? YES**, and this is the sharpest of the four. Today the *namespace string* is the security
boundary while the *registry* is the actual authority — the wall is a spelling. After: the registry is
the wall, and the tag is just a name.

**Good UX? YES.** A reader seeing `#wat.kernel/Address` learns what the value is. `#wat-edn.cap/address`
teaches them the Cargo layout.

## The gate

| # | assertion |
|---|---|
| 1 | `grep -rn 'wat-edn' src/ crates/ tests/ wat/ wat-scripts/ wat-tests/` → **0 tag-namespace sites** (the `crates/wat-edn` path itself remains; it is the crate's name) |
| 2 | a capability emits `#wat.kernel/Address` |
| 3 | encode and decode resolve by the **same key** (`type_path`); `codec.name` is gone or justified |
| 4 | ⛔ **`cap_decode_boundary` green, updated to the NEW tag** — the untrusted path still REFUSES; the trusted door still accepts |
| 5 | ⛔ a forged `#wat.kernel/Address` from **untrusted** data is refused, and the refusal consults the **registry**, not a namespace string |
| 6 | `tests/comms/probe_arc272_6a_capability_handoff.rs` green |
| 7 | emitter + refusal in **one commit** — no interval where they disagree |
| 8 | floor GREEN via `scripts/floor.sh` — the **Summary line** |
| 9 | `cargo clippy --release --all-targets` → **0** |
| 10 | `#[ignore]` count **13**, unmoved |

Rows **4** and **5** are load-bearing and they are the reason this stone exists. Row 5 in particular:
a green row 4 that passes because the refusal now matches a *different* string would be the defect
wearing the ward's clothes.

## Completion — what this closes

294.m is the **last** `#wat-edn` family. With 294.i (`.opaque`), 294.j (`.holon`), 294.k
(`.local`/`.opaque` residue) and 294.l (`.float`) landed, `#wat-edn` reaches **zero** and the builder's
ruling — *only `#wat.*` survives* — is complete. The arc's INSCRIPTION should not be written until
this one lands.
