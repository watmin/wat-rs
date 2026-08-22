# BRIEF — the type registry holds the BUILTIN types

DESIGN: `DESIGN-STONE-the-type-registry-holds-the-builtin-types.md`.
**RULED: E (consumption — the door that already exists) implemented by C (storage).**
⛔ Read the DESIGN's final **CORRECTION** section: E alone was not implementable, and the two axes it
separates are what this brief builds.

## The work in one paragraph

`TypeEnv` currently answers "is this a type name?" for 36 aggregate error/outcome records and nothing
else — not `:wat::core::i64`, not `:wat::core::Vector`, not `:wat::kernel::Peer`. Give it a second
store for **names that have membership but no structure**, have `contains` consult both, leave `get`
alone, and populate it. Nothing above `TypeEnv` changes: `SymbolTable::registrations` already routes
its `Type` facet through `contains`, so THE DOOR starts telling the truth for free.

## Read in order

1. `src/value/symbol_table.rs:244` — `registrations()`, **THE DOOR**, and `RegistryKind` above it.
   Read its comments. This is the interface that must NOT change; you are making one of its five
   facets honest, not adding a sixth.
2. `src/types.rs:470-490` — `TypeEnv`'s fields. `types: HashMap<String, TypeDef>` is why a name
   cannot be registered without a structure, and why the second store exists.
3. `src/types.rs:522-528` — `contains` and `get`. `contains` grows one `||`. **`get` does not change.**
4. `src/types.rs:791` — `register_builtin_types`, the home. 36 `register_builtin` calls today, all
   aggregates.
5. `src/check.rs:993` — `BARE_PRIMITIVES`, and `BARE_CONTAINER_HEADS` below it. **These two are
   DERIVED FROM, never copied** — see below.

## The population — derive what you can, verify what you cannot

**Groups 1 and 2 — DERIVE. Do not transcribe.** `BARE_PRIMITIVES` and `BARE_CONTAINER_HEADS` are
existing consts and the checker's own source of truth. Register by iterating them, so the two can
never drift. ⚠ `BARE_CONTAINER_HEADS`'s FQDN column follows `TypeExpr::Parametric.head`'s convention
and carries **no leading colon**; the rest of the registry is colon-prefixed.

**Group 3 — VERIFY, then register with its reason.** These have no const to derive from: they are
Rust structs exposed to wat with no `TypeDef` — a token, not a structure. The list below was measured
empirically on branch `arc109-type-refs-parked` (`git show
arc109-type-refs-parked:src/resolve/type_refs.rs`, lines ~96-148), by imposing the check and reading
what the stdlib rejected — 25 distinct names over 312 sites.

```
scalars      :wat::core::bigint · rational · keyword
AST leaves   :wat::holon::HolonAST · :wat::WatAST
sentinels    :wat::core::Value · :wat::core::Never
container    :wat::core::List
opaques      :wat::core::Uuid · :wat::holon::Hologram · :wat::holon::Vector
             :wat::io::IOReader · IOWriter
             :wat::kernel::Process · Thread · Address · Listener · Peer · ThreadSelfPeer
             :wat::stream::Stream · :wat::time::Duration · Instant
rust-backed  :rust::crossbeam_channel::Sender · Receiver
```

⛔ **That list is EVIDENCE, not an instruction.** It came from a rider's convergence and this file has
made my counts wrong repeatedly. **Verify each name is genuinely used in a TYPE position in the
corpus before registering it** (`grep -rn "<name>" --include=*.wat wat/` and confirm it appears as an
annotation/field/return type, not only in a comment or a string). A name you cannot justify is
**STOP-2**, not a registration.

## What must NOT change

- `TypeEnv::get` — a builtin has membership, not structure. It returns `None` and that is correct.
- `TypeDef`, `Nature`, and all 311 `TypeDef::` sites. No new variant (option A, rejected), no new
  nature (option B, rejected).
- `SymbolTable::registrations` and `RegistryKind`. You are not adding a sixth kind.
- Any observable wat-level behaviour. **This stone is invisible from wat**: nothing reads the new
  answer yet. The wall that will read it is parked on `arc109-type-refs-parked`.

## The gate — or this is a list that rots

Ship a test that **derives** the expectation: every entry in `BARE_PRIMITIVES` and
`BARE_CONTAINER_HEADS` must be `contains`-true on a `with_builtins()` env. That one cannot drift,
because it reads the same const the registration does.

⚠ Group 3 has no such gate in this stone and you must not pretend otherwise. Its gate is the parked
type-reference wall: once that lands, an unregistered opaque used in a stdlib signature goes red. Say
so in the code comment rather than implying group 3 is self-checking.

## STOP triggers — ship nothing further and report

- **STOP-1 — the floor must not move.** This stone adds membership answers nothing reads yet. If any
  test changes behaviour, a consumer was depending on `contains` being FALSE for these names —
  that is a finding about the substrate, not a number to accept. STOP and report which test and why.
- **STOP-2 — a name you cannot justify.** If any group-3 name is not demonstrably used in a type
  position in the corpus, do not register it. Report it.
- **STOP-3 — a contains-then-get caller.** I measured zero sites that do `contains` then unwrap
  `get`, but I measured it by grep. If you find one, the membership/structure asymmetry breaks it —
  STOP and report it rather than making `get` fabricate a `TypeDef`.

## Acceptance

| # | what | expected |
|---|---|---|
| 1★ | `registrations(":wat::core::i64")` — **through THE DOOR, not the new store** | contains `RegistryKind::Type` |
| 2 | same for a container (`:wat::core::Vector`), an opaque (`:wat::kernel::Peer`), a rust-backed (`:rust::crossbeam_channel::Sender`) | contains `Type` |
| 3★★ | `registrations(":user::NoSuchType")` — **the negative control** | **empty** |
| 4★ | `get(":wat::core::i64")` | **`None`** — membership without structure, asserted so the asymmetry is documented by a test rather than by a comment |
| 5 | the derived gate | every `BARE_PRIMITIVES` + `BARE_CONTAINER_HEADS` entry is `contains`-true |
| 6 | scoped suite | `cargo nextest run --release -E 'binary_id(wat::types)'` green |
| 7 | clippy | 0 under `-D warnings` |

**Row 1 must go through `registrations`, not through the new field.** Testing the store directly
proves the store works and says nothing about whether the door tells the truth — and the door is the
entire point of the ruling.

**Row 3 is the row that catches a registry that says yes to everything.** Rows 1, 2 and 5 are all
positives; a `contains` that returned `true` unconditionally passes every one of them.

## Boundaries

- `src/types.rs` and new tests. Nothing else.
- Do NOT run `scripts/floor.sh` or a full `cargo nextest` — the orchestrator measures centrally. Your
  own check is the scoped `binary_id(wat::types)` run. ⚠ A scoped run is not the floor.
- Do NOT commit, push, stash, revert or amend. Leave everything in the working tree.
- Do NOT touch branch `arc109-type-refs-parked` or merge from it. Read it for evidence only.

Prefix long commands with `systemd-run --user --scope -q -p MemoryMax=16G -p MemorySwapMax=0 timeout 900`.
Read exit codes DIRECTLY — never through a pipe, and never after a trailing `; echo` (that reports the
echo's status; it masked a red floor for me earlier today).

## Your report

Every acceptance row with verbatim output, rows 1, 3 and 4 especially. Your verification of each
group-3 name — which file and line proves it is used in a type position. Anything in the evidence list
you refused to register, and why. What surprised you. Anything you inspected and left alone.
