# DESIGN — Stone 6b-ii-β (IDEALIZED): the user passes the execution locus; the launch arm owns the transport

> Opened 2026-06-16. Grounded against HEAD `fc78f0b8`. SUPERSEDES the literal-`(process)` framing in
> `DESIGN-STONE-6b-process-launch.md` (β-2 v1, reverted) and `BRIEF-STONE-6b-ii-beta-2.md`. Recovered the
> established design via a doc hunt: `259-forced-hand/DESIGN.md` (the `spawn-program'` defclause; "new
> locus kinds = new clauses against new locus types, zero existing edits"), `209/DESIGN-STONE-host-parity-4a-start.md`
> (`start [locus <- :Locus]`), `wat/spawn.wat:43-90` (the typed opts records + constructors).

## The law (what was already agreed — do not re-litigate)

1. **The user passes a configured execution locus to `start`.** `start [locus <- :wat::spawn::Locus  state0]`
   (β-1, shipped). The locus is a typed opts **record** the user constructs: `(thread)`, `(thread/init f)`,
   `(process)`, `(process/env "…")`, a future `RemoteOpts(host port cert …)`. The locus carries its config.
2. **Launching is dispatched on the locus TYPE, via the `spawn-program'` defclause.** A new transport = one
   new `spawn-program'` clause + one new `Locus/launch` `extend-type` — **zero edit to `start`, zero edit to
   defservice.** `RemoteOpts` is perpetually-awaiting-definition (its constructor arity will be the lock).

## The flaw β-2 v1 introduced (now corrected)

β-2 v1 baked `(listener' (:wat::spawn::process) …)` into **defservice's** generated child-forms. That put
the **transport in the macro** — adding remote would edit defservice's codegen. It breaks law #2.

## The idealized shape: defservice is transport-AGNOSTIC; the launch arm owns the transport

- **defservice emits `<fqdn>::service-forms`** — a transport-agnostic fragment: the `Op`/`Reply` enums,
  the Request/Response records, and `serve`. NO autobind, NO child `:user::main`, NO transport keyword.
  defservice never names thread/process/remote. (It also emits the agnostic client face + `start`, β-1.)
- **`launch<S,R,St> [self state0 serve service-forms] -> Launched<S,R>`** — the per-locus arm assembles the
  tier's program:
  - **Thread arm** (shared memory): mints `(listener' self :S :R)` in-process, builds the serve closure
    capturing it + state0 (uses `serve`), spawns. Ignores `service-forms` (serve is already in the parent
    universe). — shipped in β-1, just gains the unused param.
  - **Process arm** (separate memory): assembles the child program = `service-forms` ++ a child
    `:user::main` it builds, whose autobind is **the process arm's own** `(listener' (process) :S :R)`
    (the `(process)` literal lives HERE, in the process `extend-type` — correct, not in the macro); +
    capability handoff (send addr) + `recv'` state0 + call serve. Spawns via `(spawn-program' self …)`.
  - **Future remote arm**: same `service-forms`, but its child main autobinds the remote transport
    (`(listener' <remote-bind> …)` from `self`'s config). **Zero defservice edit, zero `start` edit** —
    exactly law #2.

So the locus flows end-to-end: `user builds locus → start locus → launch locus (dispatches per type) →
spawn-program' locus (defclause executes it)`. The transport literal appears ONLY inside the locus's own
`launch` arm.

## The one crux to confirm — how the process arm assembles the child `:user::main`

The arm has `serve` (a runtime keyword), `service-forms` (a runtime `Vector<WatAST>`), `self` (the locus),
and `S`/`R` (type-params). It must produce `service-forms ++ [child-main-AST]` where the child main
splices in `serve`. Three ways:

- **(A) The arm builds the child main at runtime** (RECOMMENDED) — a forms template with `serve` spliced,
  concatenated onto `service-forms`. Precedent: β-1's `start` built the `Locus/launch<Op,Reply>` head at
  runtime via `string::concat` + `keyword/from-string`. The arm owns its transport literal; only `serve`
  is dynamic. Obvious/Simple/Honest: YES (the arm that owns the transport owns its child program); the
  runtime forms-assembly is the only new mechanism (confirm wat can splice a runtime value into a forms
  template, else build via `read-string` of a constructed source string).
- **(B) defservice builds an agnostic child main** that dispatches `(Locus/child-listen <transport> …)`,
  with the transport crossing to the child first (an extra lineage recv before autobind). Cleaner phase
  split, but adds a child-listen protocol + a transport-config crossing. More moving parts.
- **(C) child re-runs the parent program** — rejected (the fork-source path is slated to die;
  [[feedback_dont_patch_the_grave]]).

**Recommendation: (A).** It places the transport exactly where law #2 wants it (the per-locus arm), needs
no new protocol or crossing, and reuses the β-1 runtime-keyword-building precedent. (B)'s extra crossing
buys a cleaner phase split we don't need yet.

## Buildable now vs the seam

- **NOW (thread + process):** the agnostic `service-forms`, the `launch` `service-forms` param, the
  ProcessOpts arm (with its own `(listener' (process) …)`), the runtime child-main assembly. Gate: the
  headline probe `probe_arc272_6b_defservice_on_process` GREEN; c3 thread GREEN.
- **SEAM (remote, deferred):** a `RemoteOpts` + its `spawn-program'` clause + its `launch` arm (remote
  autobind from `self`'s bind config) + the mTLS trust ([[NOTE-remote-mtls-trust]]). The architecture
  above leaves this a pure addition — no defservice/`start` edit.

Pairs [[project_rendezvous_inherited_capability]] + [[project_shared_memory_partition_hosting]]
+ [[feedback_four_questions_weigh_hard_constraint_parity]] (the narrow-waist that forces transport into
the arm) + DESIGN-STONE-6b-DEP (the generic-method type-arg dep, shipped) + NOTE-service-final-state-return.
