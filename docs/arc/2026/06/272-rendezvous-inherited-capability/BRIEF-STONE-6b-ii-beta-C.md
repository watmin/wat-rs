# BRIEF — Stone 6b-ii-β (design C): defservice agnostic service-forms + per-locus launch arm provides __transport

> Single-hop sonnet Shadowdancer. Do NOT spawn sub-agents. Work only in `~/work/holon/wat-rs`. Commit
> nothing; the orchestrator weighs the diff + re-runs the gate. Grounded against HEAD `c148ed6e`,
> branch `arc-170-gap-j-v5-deadlock-state`. Design: `DESIGN-STONE-6b-ii-beta-IDEALIZED.md`.

## The work (one paragraph)

Make a `defservice` run on a forked `(process)` through the SAME client face it uses on a `(thread)`,
WITHOUT defservice ever naming a transport. defservice emits a transport-AGNOSTIC `service-forms` value
(Op/Reply + records + serve + an agnostic child `:user::main` that binds on a free `service-locus` it does
NOT define). The per-locus `launch` arm provides the transport by prepending `(def :…::service-locus
<its-locus>)` and concatenating `service-forms`. The `(process)` literal lives ONLY in the ProcessOpts
arm; a future RemoteOpts arm provides its own. Gate: `probe_arc272_6b_defservice_on_process` GREEN
(returns 5); c3 thread GREEN.

## Already proven (build ON these, don't re-probe)

- process serve loop + owner-drop termination (`probe_arc209_c0b3aii_process_service_loop`); child-mints
  capability handoff (6a); state0 record crossing parent→child over the lineage (6b-ii-α); socket peers
  carrying records (6b-ii-α); `poll'` accepts any self-peer type (`check.rs:11327`) → the child main calls
  serve via `apply` (dynamic) so its lineage self-peer (`Peer'<Address',St>`) need not statically match
  serve's `self <- Peer'<Reply,Op>`.

## Build

**1. `wat/service.wat` — emit `<fqdn>::service-forms` (transport-agnostic) in the `do` block.**
A def binding a `(:wat::core::forms …)` value, REUSING the same generated nodes already spliced into the
top-level `do` (request-records, response-records, `(defenum Op …)`, `(defenum Reply …)`, `(defn serve …)`),
PLUS a generated agnostic child `:user::main`:
```wat
(:wat::core::def :<fqdn>::service-forms
  (:wat::core::forms
    ~@request-records
    ~@response-records
    (:wat::core::defenum ~enum-name ~@variants)
    (:wat::core::defenum ~reply-name ~@reply-variants)
    (:wat::core::defn ~serve-name ~serve-params -> :wat::core::nil ~serve-body)
    (:wat::core::defn :user::main [] -> :wat::core::nil
      (:wat::core::let
        [b    (:wat::kernel::listener' :wat::spawn::service-locus ~enum-name ~reply-name)
         self (:wat::program::self-peer
                 :wat::kernel::Address'<~enum-fqdn,~reply-fqdn> ~state-ty)
         _    (:wat::kernel::send' self (:wat::spawn::Bound/address b))
         st   (:wat::kernel::recv' self)]
        (:wat::core::apply -> :wat::core::nil
          (:wat::core::keyword/from-string ~serve-name-str) self
          (:wat::spawn::Bound/listener b)
          (:wat::core::Vector ~peer-ty)
          st [])))))
```
KEY: the child main references `:wat::spawn::service-locus` — a free name defservice does NOT define (the
per-locus arm provides it). Op/Reply/serve/state-ty are LITERAL (defservice has them → the child main
type-checks in the child universe once `service-locus` is prepended). serve is called via `apply` (dynamic).
If a `(def …)` binding a `(forms …)` in the `do` is problematic, fall back to a 0-arg fn
`(defn :<fqdn>::service-forms [] -> :wat::core::Vector<wat::WatAST> (forms …))` (STOP-and-report which).

**2. `wat/spawn.wat` — `launch` gains a `service-forms` param; the ProcessOpts arm.**
```wat
(launch<S,R,St> [self state0 serve service-forms <- :wat::core::Vector<wat::WatAST>] -> :wat::spawn::Launched<S,R>)
```
- ThreadOpts arm: add the param, IGNORE it (serve is already in the parent universe; body otherwise the β-1 closure).
- ProcessOpts arm (NEW `extend-type`): prepend the transport def, concat, spawn, handshake:
```wat
(launch [self state0 serve service-forms]
  (:wat::core::let
    [prog (:wat::core::concat
            (:wat::core::forms (:wat::core::def :wat::spawn::service-locus (:wat::spawn::process)))
            service-forms)
     svc  (:wat::kernel::spawn-program' self prog)
     addr (:wat::kernel::recv' svc)
     _    (:wat::kernel::send' svc state0)]
    (:wat::spawn::Launched/new svc addr)))
```
The `(:wat::spawn::process)` literal is the PROCESS arm's own (the user's config rode `self` into
`spawn-program'`; the child's `service-locus` is the transport identity for autobind).

**3. `wat/service.wat` — `start` passes `service-forms` to launch.** The β-1 launch call gains the arg:
`(~launch-head-kw locus state0 (keyword/from-string ~serve-name-str) :<fqdn>::service-forms)` (reference
the emitted service-forms value).

## Rooms (read in order)
1. `wat/spawn.wat:117-213` — Launched/Bound/Locus/launch + the β-1 ThreadOpts arm.
2. `wat/service.wat:60-110` (binders: enum-name/reply-name/serve-name-str/peer-ty/state-ty/fqdn-str),
   `:330-365` (serve gen), `:483-547` (start + the `do`).
3. `tests/probe_arc272_6b_defservice_on_process.rs` (GATE) + `tests/probe_arc272_6b_state_over_lineage.rs`
   + `tests/probe_arc209_c0b3aii_process_service_loop.rs` (proven child-main + serve shapes).
4. `tests/probe_arc209_c3_defservice_client_face.rs` (thread gate).

## STOP triggers (halt + report; ship nothing)
1. STOP if `(def :name (forms …))` in the `do` block won't bind/reference cleanly — report; the 0-arg-fn
   fallback is the alternative.
2. STOP if `(:wat::core::concat (forms …) service-forms)` doesn't concatenate two `Vector<wat::WatAST>` —
   report the actual op (it may be a different vector-append verb).
3. STOP if the child main fails to typecheck for a reason OTHER than `service-locus` (e.g. a real Op/Reply
   gap) — report it; do not work around with `:Any`.
4. STOP if `recv' self` for state0 needs a `-> :T` arrow ascription (must infer; the arrow is annihilated — 258.5b).

## Gate (orchestrator re-runs)
- `cargo test --release -p wat --test probe_arc272_6b_defservice_on_process -- --include-ignored --test-threads=1`
  → GREEN (returns 5); remove the `#[ignore]`.
- `cargo test --release -p wat --test probe_arc209_c3_defservice_client_face -- --test-threads=1` → GREEN (5).
- `cargo test --release -p wat --lib -- --test-threads=1 | grep "test result"` → 929/36 (zero new).
- `cargo test --release -p wat --test nursery -- --test-threads=1 | grep "test result"` → 893/4 baseline.
- `cargo build --release -p wat` → clean.

Report: exact files+lines changed, how you emitted `service-forms` (def vs fn) + concatenated, the gate
results from your OWN runs (pasted), and any STOP hit.
