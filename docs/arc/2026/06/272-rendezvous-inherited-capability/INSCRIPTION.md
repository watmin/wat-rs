# INSCRIPTION — Arc 272: rendezvous is an inherited capability, not a discovered name

> Opened 2026-06-15, closed 2026-06-16. A pivot, not a feature. The process-tier rendezvous was built
> on a fixed, human-chosen abstract-UDS name — **collidable** (EADDRINUSE), **forgeable** (squat-first),
> and its "mTLS" one-directional. This arc **annihilated the collision class structurally** (no shared
> fixed namespace can exist), made identity **mutual by construction**, brought the process tier to the
> thread tier's standard, and — as it ran — grew to carry host/locus parity for `defservice` and the
> record-state final-state contract. Deep lessons live in `REALIZATIONS.md` (6 sections); this is the
> closure ledger. **INSCRIPTION = DONE.**

## What shipped (the threads, oldest → newest)

**1. The capability subsystem (the home + the waist).**
- `src/capability/` minted as a warded home (`30f86d58`); a 2nd capability proven to ride the same
  frozen narrow waist (`5269a8d7`) — the hourglass.
- The powerbox / comms policy: `CommsPolicy::OnlyMyPeers` (accept-gate) + the connect-gate
  (`410af5e1`, `3d6357ed`), **zero-mutex** (allow-set is a `ThreadOwnedCell`, `06bfdf92`).
- **Step 5 — annihilate the name** (`4e473da1`/`731e2bbf`/`4806ccbe`): `socket-address'`, the 2-arg
  named `listener'`, and `Address::from_socket_name` are GONE. Rendezvous is capability-only autobind;
  `EADDRINUSE` is unreachable code, not a handled error (extirpare, top rung).
- **6c.2 — mutual identity by construction** (`74b28e48`→`079733ef`): both gates verify uid+pid; the
  minter stamps its own `getpid()` into a registered `SocketAddressWire` record; `connect_admits` →
  `OnlyThisPeer{pid}`, symmetric with accept; `AnyOfMyUser` annihilated. Death-then-rebind closed by
  construction. The capability **vigilatum CONVERGED (L1+L2=0) and the home is STAMPED** (`079733ef`).
- The substrate's immune system caught a **false premise in its author's own code** (the "autobind
  names are unguessable" claim, retracted; see REALIZATIONS §4 + `feedback_perfect_knowledge_and_false_substrate_premise`).
- Records cross the EDN wire both directions (`95fe6c74` 234.7a base records; holon via holon_form-as-edn)
  — the prerequisite the capability codec rides.

**2. Host / locus parity — the same `defservice` runs on any execution context** (`9edf0b2f`→`c3cf58b5`).
- 6b-ii-α wired parent→child lineage `recv'` (socket-tier decode-with-registry, `611d68e3`); the
  generic-method type-argument application dep (`82b21ce8`); 6b-ii-β-1 the `launch` reshape (`9fa2bf23`);
  6b-ii-β **design C** — defservice names NO transport, emits transport-agnostic `service-forms`; the
  per-locus `launch` arm owns its transport (`3162029a`/`c148ed6e`). A new transport = one `extend-type`
  + one `spawn-program'` clause, **zero defservice/start edit** (the narrow waist).
- 6b-iii — wat-level proof: `defservice` on thread AND process (`c3cf58b5`).
- **Host → Locus** comprehensive rename (`006542e0`): a thread/process/remote is a **Locus**, not a host.

**3. Record-state — a service's value is its final state, and that state is a record.**
- **rs-2 — the `:Stop` terminal op** (`a57b9f0b`): `(<svc>/stop c) -> <State>` terminates a service and
  returns its final state (gen_server `{stop, State}`). The state rides back as a `:Stop` reply over the
  client connection — constant shape across thread/process/remote, no new substrate, no lineage reshape;
  `serve` stays `-> :nil`. Far-side crash surfaces to the client as a raise (locked by a probe).
- **rs-1 — `:state` MUST be a record, by CONSTRUCTION** (`87ad7c29`/`f90d1935`): `defservice` takes the
  state's FIELDS inline and MINTS `:<fqdn>::State`, so a non-record state is **unexpressible** (top rung,
  not a check). Optional trailing `:record-parent` selects base (`:wat::Record`, default) vs a real holon
  record (`:wat::holon::Record`, with the VSA `holon_form`); trailing options parse as a validated kwargs
  map that names any unknown key directly.

## What is affirmatively OUT of arc 272's scope

- **rs-3 (await final state on owner-drop) — REJECTED, cut** (`99d59597`). The trigger encodes intent with
  no third case: call `stop` to get the final state, or drop the handle (the existing owner-drop →
  `ServiceEvent::Shutdown` path, proven by the c3 probe) when you don't care. No `await` verb; the
  `ServiceUp` lineage reshape evaporates. The final-state feature is complete with rs-1 + rs-2 alone.
- **mTLS remote-trust** — out of scope; tracked in `NOTE-remote-mtls-trust.md` (the design-C seam is
  ready: a RemoteOpts locus = a new `spawn-program'` clause + `launch` arm + `CommsPolicy` rung, zero
  defservice/start edit — its own arc opens when remote trust is built).
- **The confinement horizon** — out of scope; tracked in `NOTE-confinement-horizon.md`.
- **Portable capability tags** — out of scope; tracked in `NOTE-portable-capability-tags.md`.
- **`record?` reflection completeness** (the runtime VALUE predicate is holon-only) — out of scope;
  tracked in arc 273 (`docs/arc/2026/06/273-record-reflection-completeness/STUB.md`). rs-1 did NOT need
  it (rs-1 uses the TYPE-level mint, not the value predicate).
- **The macro-time kwargs pattern** (defservice's inline opts-map) is recognized but NOT extracted —
  one consumer today; recorded in arc 260's DESIGN as the macro-time sibling of runtime keyword-args,
  to extract (as a macro-eval intrinsic) when a second option-taking macro appears.
- **Residual `host` in source** — the LIVE concept rename is complete (zero `:wat::spawn::Host` /
  `service-host` survives). Remaining `host` strings are historical stone-names (e.g. "host-parity-4a")
  and arc-259 test-comment cosmetics — kept by design as historical record (FM-14 buckets C/D); the
  arc-259 comments track with arc 259, not here.

## Prior-art collisions (independent rediscovery — full detail in REALIZATIONS)
Object-capability security (Dennis & Van Horn, 1966) at the EDN boundary; the narrow-waist / hourglass;
end-to-end (Saltzer/Reed/Clark, 1984); Erlang/OTP gen_server (call/reply + terminate-with-state +
supervised state handover). What is genuinely ours: these textbook models on a typed-ADT-on-Rust
substrate where the wire-conformant record makes resumability/handover/identity fall out of the value
itself — and host-parity the BEAM never had to engineer (the per-locus launch arm vs the BEAM VM).

## Verification at close (weighed on the orchestrator's own build)
Full `defservice` family GREEN (c1/c2/c3, locus-agnostic-start, naming, acronym-registry, 6b-on-process,
rs-1 ×4, rs-2 thread/process/crash); wat-level locus-parity deftests GREEN; lib **929 / 36** and nursery
**893 / 4** (the failures pre-existing, unrelated to this arc). HEAD at close: `f90d1935` (+ `628886fb`
docs). Capability home vigilatum STAMPED.

## Pairs
`REALIZATIONS.md` · `project_rendezvous_inherited_capability` · `project_lockstep_blocking_channels_fpga` ·
`feedback_perfect_knowledge_and_false_substrate_premise` · `feedback_gate_whole_family_when_generated_surface_changes`.
