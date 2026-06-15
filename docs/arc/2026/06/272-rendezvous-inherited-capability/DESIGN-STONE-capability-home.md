# DESIGN — `src/capability/` : the capability subsystem in its FINAL warded-home form

> Opened 2026-06-16. Builder: *"we've got this huge refactor pending that'll prune nearly everything out
> of src/*.rs into src/<home>/<names>.rs … you wanna get caps built out in its final form? we'll toss
> the grimoire at it and prove it done?"* Strike 1/2 landed the narrow waist as a single top-level
> `src/capability.rs` — a placeholder. This stone builds the subsystem in its **final warded-home
> form** (so the pending src/*.rs→homes refactor never has to touch it), houses **v4** in it, and earns
> the **`vigilatum`** stamp by full-grimoire combat. Build once, ward once, done.

## Scope — what the capability subsystem OWNS

One subsystem, one home: **"what may cross a boundary as a capability, how it serializes, and who may
receive it."** Three concerns, three files:

```
src/capability/
  mod.rs        — home root: module doc, re-exports, the vigilatum stamp once earned
  registry.rs   — the WAIST: `CapCodec` + the OnceLock registry + encode_capability/decode_capability
                  (the generic dispatch) + the registrants (Address' codec; future caps register here)
  policy.rs     — v4 the POWERBOX: `CommsPolicy` + `only-my-peers(PeerCred)` (euid==mine ∧ pid∈lineage)
  tests         — waist_proof (2-cap), the cap_decode_boundary ward, the v4 multi-process proof
```

(Distributed codecs — a cap's `CapCodec` living beside its own type — is a later refinement; for now
the registrants sit in `registry.rs`, the one append-only edge.)

## The boundaries (what does NOT move — kept honest)

- **`edn_shim` keeps the EDN read/write dispatch + the trust door.** `decode_trusted_wire` is the
  *EDN-reader's* trusted entry; it DELEGATES to `capability::decode_capability`. The RustOpaque encode
  arm DELEGATES to `capability::encode_capability`. Those one-line delegations stay in edn_shim — the
  capability home owns the *registry + codecs + policy*, not the EDN reader. Clean dependency: edn_shim →
  capability (never the reverse for the read path).
- **The comms gates (accept/connect) stay in their homes** (`kernel::address::connect`,
  the accept arm) — they *consult* `capability::policy` instead of inlining euid/allow-set checks. The
  policy TYPE lives in capability; the ENFORCEMENT stays at the gate.

## v4 in the home — the comms policy (powerbox)

`policy.rs` defines `CommsPolicy` (today: `only-my-peers`) — a predicate over a verified `PeerCred`.
The accept gate (server vets client) and the connect gate (client vets server) call it; the allow-set
*becomes* the lineage pid-set the policy reads (`allow'`/`deny'` mutate it). This unifies the three
scattered hardcoded checks (subsumes the old 6c pid-trust) and is **shaped to become wat-expressible**
(the predicate as a wat `fn(PeerCred)->bool`) — the v4-predicate the builder wants.

## Decomposition (the strikes)

1. **Mint the home.** `src/capability/{mod,registry}.rs`; move `src/capability.rs` content into
   `registry.rs` (the waist + Address' codec + waist_proof); `git rm src/capability.rs`; `mod capability`
   stays in lib.rs (now a dir). edn_shim's two delegations unchanged. GREEN gate: 6a + waist_proof +
   cap_decode_boundary + lib/nursery baselines.
2. **v4 — `policy.rs`.** `CommsPolicy` + `only-my-peers`; wire the accept + connect gates to consult it
   (allow-set → lineage set). RED probe → GREEN: the multi-process proof (a peer comms; a non-peer is
   refused AT THE GATE), riding the frozen waist.
3. **Earn the `vigilatum`.** Cast the full ward suite via `vigilia` (intueri · solvere · purgare ·
   struere · sequi · secare · cernere · conformare · circumspicere last) — one Shadowdancer per ward,
   each embedding its spell fetched from the SIGNED datamancy MCP; drive findings to L1+L2 = 0; stamp the
   home. *"Vigilatum is earned by COMBAT — the full guard, not a pair"* ([[feedback_vigilatum_by_combat]]).

## The bar

Final form + warded + proven (waist proof + v4 multi-process proof + the vigilatum stamp). No throwaway
left for the pending refactor to relocate; the capability subsystem is *done* — shockingly well written,
not green-with-a-placeholder. Pairs [[feedback_bar_shockingly_well_written]] + [[feedback_vigilatum_by_combat]]
+ REALIZATIONS.md (the narrow-waist / ocap / end-to-end synthesis this home embodies).
