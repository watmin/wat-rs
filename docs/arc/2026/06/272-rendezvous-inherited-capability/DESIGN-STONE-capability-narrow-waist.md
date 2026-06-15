# DESIGN — the capability protocol: a narrow waist for `wat-edn.cap`

> Opened 2026-06-16. Builder: *"this looks like a one-off … we need to know how to build the next
> capability — extremely rigid to the point where changes are probabilistically zero but enabling
> unlimited expression … how do we engineer organic evolution of this tooling?"* The 6a-i `Address'`
> implementation is hand-coded into edn_shim's core (an `if type_path == ADDRESS` encode arm + an
> `"address" =>` decode arm). Adding the next capability = editing the core. This stone makes the
> capability mechanism a **narrow waist**: a frozen protocol + open registration. **Precedes v4** —
> the comms policy and every future capability ride this waist.

## The principle — the narrow waist (hourglass)

A thin, frozen, universal interface with unbounded diversity above and below. **The rigidity is what
*causes* the unlimited expression**: because the waist never moves, anything can build on it without
coordination; a waist that flexes kills the ecosystem above it. Prior art (honest): IP's hourglass
(one packet format; unlimited apps/links), the **Unix fd** (one tiny interface; unlimited resources),
**Clojure protocols** (a frozen dispatch contract, extended forever without editing it), Hickey's
*accretion-not-breakage*. This is the same "new transport = a new `extend-type`, zero central edit" law
the substrate already lives by — applied to capability serialization.

## The frozen waist (changes ≈ zero)

1. **The wire contract** — `#wat-edn.cap/<name> <body>`. Fixed.
2. **Generic encode dispatch** — `value_to_edn`, for a `RustOpaque`: *is this type a registered portable
   capability?* → emit `#wat-edn.cap/<its-name> <its-bytes>`. ONE arm, no per-cap code.
3. **Generic decode dispatch** — `cap_tag_to_value(name, body)`: look `name` up in the registry → call
   its reconstructor. ONE arm, no per-cap code.
4. **The trust door** (v1, `decode_trusted_wire`) — unchanged. Caps reconstruct only off the trusted wire.
5. **The registration interface** — `PortableCapability { name() , to_wire_bytes(&self), from_wire_bytes(bytes) }`.
   **This signature is the ABI.** Once shipped it only *accretes* (a new method gets a default), never breaks.

## The open edge (unlimited expression, zero central edit)

A type becomes portable by **registering** `(name, type_path, encode, decode)` — riding the existing
rust-type registration idiom (the `#[wat_dispatch]` / marshal registry, immutable-at-startup like every
`FrozenWorld` registry). Adding `Address'`, a future `Grant`, `Lease`, `Token`, … = a new module + one
registration. `edn_shim`'s core never moves.

## Four questions

- **Obvious?** YES — a registry keyed by capability name is the obvious shape; the waist names itself.
- **Simple?** YES — two generic dispatch arms *replace* N hand-coded ones; complexity leaves the core.
- **Honest?** YES — the contract is explicit and frozen; a new capability physically cannot corrupt the
  core dispatch or the trust door (it only adds a registry row).
- **Good UX?** YES — to ship a capability you implement a trait + register; you never read, let alone
  edit, `edn_shim`.

## The proof (the gate) — "unlimited expression," demonstrated not asserted

Register a **second, trivial capability** beside `Address'` (e.g. a toy `Token(u64)` portable cap), and
round-trip BOTH over the trusted wire — with `edn_shim`'s core dispatch **untouched** for the second one
(it was already generic after strike 1). The diff for capability #2 touches only its own module + one
registration line. That is the waist working: N capabilities, one frozen core.

## Decomposition

1. **The waist** — introduce the `PortableCapability` registration + the registry; rewrite the encode arm
   (`RustOpaque` → registry lookup) + `cap_tag_to_value` (name → registry lookup) as the two generic
   dispatches. **Lift `Address'`** to be the first registrant. 6a probe + boundary ward stay GREEN.
2. **The proof** — register a second trivial capability; round-trip both; show the core diff is zero for #2.
3. **Freeze + document** — declare the registration interface the ABI in NOTE-portable-capability-tags.md
   (accrete-don't-break); the waist is closed-inscribed.

Then **v4 (comms policy)** rides the frozen waist. Pairs [[project_rendezvous_inherited_capability]] +
NOTE-portable-capability-tags.md + [[feedback_honest_abstraction_decomplect_crutch_open_seam]]
(open trait for a growing set) + [[feedback_bar_shockingly_well_written]].
