# Arc 192 — State-preserving exec + SignalService (SIGEMT)

**Status:** stub opened 2026-05-13 per user direction. Captures cooperative-migration architecture surfaced in arc 170 INTERSTITIAL conversation. Layered atop arc 191.
**Gates on:** arc 191 (bare exec-program) shipping. May ship before arc 191's full implementation if the substrate primitives are pulled forward.

## Motivation

Arc 191 introduced bare exec-program: replace the universe; services kept; no state carried over. The user's exploratory conversation (arc 170 INTERSTITIAL — "how insane can we take this hot reloading?") surfaced the next layer: **carry user state across the universe boundary**.

User direction 2026-05-13:
> *"/everything/ that wat is edn?.. we should just edn-ify our state and boot into a new universe with our value?..."*

Plus the cooperative-migration framing:
> *"we can use something like SIGUSR2 or something and reserve that for reloads.. any compliant thread who wants to be teleported needs to gracefully shutdown with their data and have a start-from interface ... the collection of the thread data is interesting... how do we return the thread's state to be started from?..."*

Then the signal-name resolution:
> *"bleh... term resizing could burn us fast.... SIGEMT looks unused and poetic too?..."*

This arc captures the substrate primitives that make the cooperative-migration pattern possible. The wat-side library codifying the pattern lives in arc 194.

## The two substrate pieces

### 1. `:wat::kernel::exec-program-with-state`

```
:wat::kernel::exec-program-with-state
  (forms :wat::core::Vector<:wat::holon::Atom>)
  (carry-overs :wat::core::HashMap<:wat::core::String, :wat::holon::Atom>)
  -> :wat::core::Result<:wat::core::never, :wat::kernel::ExecError>
```

What it does (on top of arc 191's exec-program):
1. Validates the new universe (preview freeze of `forms`)
2. Serializes carry-over values to EDN
3. Identifies types referenced by carry-overs (TypeEnv inspection — same Gap F-3 machinery)
4. Augments the program AST with prologue defines for carry-overs + type declarations
5. Freezes the augmented AST
6. Inherits services (OS-continuity layer)
7. Execs into the new universe
8. New universe's main accesses carry-overs via ambient (e.g., `(:wat::kernel::carry-overs)`)

The carry-over map is keyed by name; values are any EDN-serializable wat value. The new universe rebinds them in its TypeEnv if it can resolve the type; otherwise emits error.

### 2. `:wat::kernel::SignalService`

A substrate service paralleling StdInService/StdOutService/StdErrService. Owns OS signal handlers; converts signals to wat-land events deliverable via channels.

```
;; Subscribe to a signal event channel
(:wat::kernel::SignalService/subscribe :SIGEMT) -> :wat::kernel::Receiver<:Signal>
```

**SIGEMT is the reserved universe-reload signal.** Convention per user 2026-05-13 ("SIGEMT looks unused and poetic too"): wat-cli IS an emulator for wat-land programs; SIGEMT = "emulator trap" = host interrupting guest = semantically aligned with what reload IS.

wat-cli's startup installs a SIGEMT handler that overrides the default (terminate) and routes the signal to SignalService. Programs that subscribe receive `:wat::kernel::Signal/ReloadRequested` events.

External trigger: `kill -EMT <pid>` or `pkill -EMT <name>` — `EMT` is short enough to type, unusual enough to never invoke accidentally.

## Architecture — three-layer stack

| Layer | Provides | Arc |
|---|---|---|
| Substrate primitive | `exec-program` (bare) | 191 |
| **Substrate primitive** | **`exec-program-with-state` + SignalService** | **192** |
| Wat-side library | Supervisor + `loop-with-shutdown` macro + `spawn-with-state` | 194 |
| Orchestrated primitive | `:wat::kernel::exec` (all-in-one) | 194 (or substrate-level) |

Arc 192 is the substrate-side foundation. User-visible ergonomics live in arc 194.

## Slice plan (rough)

### Slice 1 — SignalService substrate + SIGEMT handler

- `SignalService` runs as substrate service (like StdInService et al.)
- wat-cli installs SIGEMT handler at startup; overrides default-terminate
- Programs receive Signal events via `Receiver<Signal>`
- The `Signal` enum carries variants for each subscribed signal type

### Slice 2 — `exec-program-with-state` substrate primitive

- Extends arc 191's exec-program with carry-over bindings argument
- Serialization machinery for arbitrary EDN-able values
- Type-resolution discipline (carry-overs require types; types must exist in new universe)

### Slice 3 — Worked example: cooperative reload of a simple worker

- Demonstrates the substrate primitives end-to-end
- Manual orchestration (no arc 194's library yet)
- Validates the architecture before the wat-side library ships

### Slice 4 — INSCRIPTION + USER-GUIDE + cross-references

Closure paperwork.

## Open design questions

1. **Signal handler installation timing.** Does wat-cli install SIGEMT at startup unconditionally, or does the program have to opt-in? Recommendation: unconditional install (convention is substrate-wide); programs that don't subscribe to the event simply don't see it; signal still doesn't terminate.

2. **`Signal` event type shape.** Does `Signal` carry payload data (e.g., signal number, sending pid)? Or is it a pure enum variant per signal? Recommendation: enum variant per signal, with optional payload-bearing variants for signals that need it (SIGCHLD has child-pid; SIGEMT has no payload).

3. **Carry-over type resolution.** If a carry-over has type `:user::Foo` and the new universe doesn't declare `:user::Foo`, what happens? Three options:
   - **(a)** Error at exec-with-state time: `Err(CarryOverTypeMissing(name, type))`. User must declare or migrate manually.
   - **(b)** Lift the type declaration into the new universe's prologue automatically (parent's TypeEnv → injection). Gap F-3 already does this for closure-extraction; same mechanism.
   - **(c)** Carry as opaque HolonAST; new universe interprets via deserialization at access time.
   - **Recommendation:** (b) — same machinery Gap F-3 already provides; the new universe gets the type because the parent had it.

4. **Sender/Receiver in carry-overs.** Channels reference live universe state; they can't EDN. Surface as error if user tries to carry them over. Probably: `Result<:wat::holon::Atom, NotEdnable>` returned from a `:wat::kernel::try-edn-encode` helper; user filters before submitting carry-overs.

5. **Signed exec-with-state.** Mirror `signed-load!` pattern: `signed-exec-program-with-state` verifies AST signature before exec. Important for distributed scenarios where a parent sends "reload into this program" over the network. Probably ships in a follow-up arc.

6. **Carry-over namespace.** Should carry-overs land in a designated namespace (e.g., `:wat::runtime::carry-overs::*`) or be injected into top-level user-visible names? Cleaner is a designated namespace + an accessor function `(carry-overs)` returning the HashMap. New universe's main reads from it.

## Cross-references

- Arc 170 INTERSTITIAL — the conversation that surfaced this architecture
- Arc 191 — bare exec-program (foundation)
- Arc 192a (folded in here) — SignalService substrate
- Arc 193 — universe image dump/resume (orthogonal capability)
- Arc 194 — wat-side worker/supervisor library + `exec` orchestrated primitive
- Memory `project_signal_cascade.md` — POSIX pgid + killpg discipline (cascade scope is intra-process for SIGEMT)
- Memory `project_wat_binary_hologram.md` — the binary-as-surface framing; exec-with-state extends it (the binary's universe is just one form; you can construct + carry-over to the next)
- `src/freeze.rs::startup_from_forms` — the freeze machinery exec-with-state reuses
- Gap F-3 (closure type-registry inheritance) — the same machinery that propagates types in carry-overs
- Gap H (closure-extraction prelude lift) — the same machinery that injects carry-over defines into prologue
- Gap I-A (`is_declaration_form`) — the predicate used at the injection site

## Why this matters

Arc 191 alone gives you universe replacement — useful but coarse. Arc 192 gives you universe replacement WITH state survival — the foundation for hot-reload-as-a-real-thing. Without 192, every reload starts from scratch; with 192, reloads can preserve accumulated work, ongoing conversations, learned models, anything the user wants to carry forward.

Combined with arc 194's cooperative-migration library, this is "Erlang-class hot reload but with static typing per universe and image-style state persistence." Neither Erlang nor Smalltalk has both. wat would.
