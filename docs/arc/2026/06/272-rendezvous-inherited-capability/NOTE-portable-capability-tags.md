# NOTE — portable capability tags (`wat-edn.cap`): the IPC-portable-handle pattern

> Born 2026-06-16 (arc 272 6a-i). The kernel's decode-refusal panic
> (`#wat-edn.opaque/RustOpaque "unsupported substrate tag"`) opened a new path. Builder: *"we're
> carving a new path here … we'll make excessive use of tags for this kind of work."* This note is
> the reusable pattern so the next capability follows the same shape.

## The two namespaces (the rule)

| namespace | meaning | decode |
|---|---|---|
| **`wat-edn.opaque`** | a **process-local** substrate handle — an fd, a crossbeam `Sender`, a `Peer'`, an `IOReader`. Has NO meaning across a process boundary. | **REFUSES** (`UnsupportedTag`). Correct: shipping it would send a dangling fd. |
| **`wat-edn.cap`** | a **portable capability** — its wire content is genuinely reconstructable on the far side (kernel-minted bytes, a stable id). | **reconstructs** via `cap_tag_to_value`. |

**Portability is per-type and DELIBERATE** — the 272 *minted-not-built* doctrine. A type joins
`wat-edn.cap` only when its wire content is safe + meaningful across the boundary. There is no blanket
"make any opaque portable" opt-in — that would be a footgun (a portable `Peer'` ships a meaningless fd).
The default (`wat-edn.opaque`, refuse) is the safe one; opting in is a decision.

## The shape (how to add a new portable capability)

First inhabitant: **`Address'`** (`#wat-edn.cap/address [byte …]`). To add the next one:

1. **Decide portability honestly.** Is the wire content meaningful + safe on the far side? (Address':
   the kernel-minted abstract UDS name bytes — yes. A `Peer'`: its fd — no, stays opaque.) If a type is
   only *sometimes* portable (Address' is portable as a socket addr, NOT as a thread-tier `Sender`), put
   the decision on the owning type, not the wire layer — see `Address::portable_name_bytes`
   (`kernel/address.rs`): returns `Some(bytes)` only for the portable inner, `None` → falls to opaque.
2. **Encode** (`edn_shim::value_to_edn`, the `Value::RustOpaque(inner)` arm): if `inner.type_path` is the
   capability's path AND it has a portable form, emit `OwnedValue::Tagged(Tag::ns("wat-edn.cap", "<name>"),
   <portable body>)`; else fall through to the `wat-edn.opaque` refusal.
3. **Decode** (`edn_shim::cap_tag_to_value`): add a `"<name>" =>` arm that validates the body shape and
   reconstructs the `Value` (e.g. `from_socket_name_bytes` → `make_rust_opaque(ADDRESS_TYPE_PATH, addr)`).
   Reject malformed bodies with `UnsupportedTag` — never fabricate.

## Why a tag, not a bare value (the four-questions verdict, 2026-06-16)

A tag makes the wire **self-describing**: `recv'` reconstructs the capability with NO runtime type hint
— the runtime complement of the 258.5a arrow-kill (which removed the *check-time* `-> :T`; the tag
removes the *runtime-decode* `-> :T`). The rejected alternative — ship a bare `Vector<i64>` + an
`address-from-bytes` verb — fails Honest + UX: it leaks the byte representation into user space and
hands out a "build a capability from arbitrary bytes" surface, exactly the *built-not-minted* path 272
deletes. The tag keeps reconstruction substrate-owned.

Pairs [[project_rendezvous_inherited_capability]] + the 258 recv'-infer arrow-kill (the tag is its
runtime half) + ZERO-MUTEX (the capability rides the channel).
