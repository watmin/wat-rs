# FINDING — the locus waist is three layers wide, and it fails open

**Measured 2026-09-23 by the orchestrator at `cdc6b18c2`.** Prompted by the builder's ruling (SEAM,
*"the locus is a narrow waist"*).

## The target, derived from the ruling

**Adding one locus touches ONE place** — the locus module declares the locus, binds its transport kind,
implements `launch`. Services know no locus; loci know no service. Services × loci meet at one
interface: **N + M, never N × M.** Choosing a locus at a call site (`:locus (:wat::spawn::thread)`) is
*above* the waist and is correct; only infrastructure that must know specific loci widens it.

## What adding one locus costs today — three layers beyond the locus module

| layer | site | what a new locus requires |
|---|---|---|
| **the service macro** | `wat/service.wat:2552-2573`, `:2586-2587`, `:2647-2652`, `:2771-2776` | a `start$impl-<locus>` **and** `resume$impl-<locus>` minted in **every service**, plus a `handle-<transport>-name` — ⛔ **N × M** |
| the service macro's routing | `wat/service.wat:2705-2709`, `:2829-2833` | a new `starts-with?` branch, **in two places** |
| **the type checker** | `src/check.rs:10536-10548`, `:10645`, `:12146` | the transport set is **hardcoded to exactly two**: `if p == ":wat::kernel::Shared" \|\| p == ":wat::kernel::Wire"`; `require-wire-address` names `Wire` literally |

Why the per-locus copies exist — they are **not** redundant (`a6da457e3`, 293.W.2f): *"`Address<S,R,T>`
carries Shared vs Wire. /start stamps Handle from the locus constructor. Process map/each
require-wire-address."* The locus writes a **transport type** into the handle so that a process dialling
a shared-memory address is a **type error**. See `FINDING-generic-lifecycle-and-the-transport-type.md`:
one type-checker capability (inferring a type variable from an implementor's `extend-type` binding) is
what would let ONE generic `start` carry the transport instead.

## ⛔⛔ The routing FAILS OPEN — twice

The macro picks the stamped copy by **string prefix on the locus constructor's name**, and falls through
to the generic, **unstamped** `start$impl`:

```wat
(if (starts-with? ctor-nm ":wat::spawn::process") start-impl-process-call
  (if (starts-with? ctor-nm ":wat::spawn::thread") start-impl-thread-call
    start-impl-call))                                ; ← unstamped; transport rule does not apply
```

1. **A new locus** (e.g. `:wat::spawn::uds`) matches neither prefix → silently takes the unstamped path.
   It does not fail. The transport rule simply does not apply to it.
2. ⛔ **The corpus conversion.** `ctor-nm` is `ast-name` of the locus head (`:2684-2702`). Measured:

   ```
   ast-name of :wat::spawn::process                    → ":wat::spawn::process"   (control)
   ast-name of wat.spawn/process                       → "wat.spawn/process"      ← raw text
   (starts-with? <that> ":wat::spawn::process")        → false
   ```

   **After 8d-iii every thread and process `start` falls through and loses its transport stamp.** ⛔ **A
   latent 8d-iii blocker.** Whether a downstream check then fails loudly is **unmeasured**; that the stamp
   is lost is measured.

⚠ **The heresy ledger cannot see this.** It reads `src/` Rust; this exact keyword string comparison
lives in wat (`wat/service.wat`). The countdown that schedules the terminal cut is blind to wat-level
heresy. `head-nm` is also compared with `(= head-nm ":wat::spawn::with-label")` — same class.

## What keeps it narrow — the wall

⭐ **Dispatch on the locus by TYPE (`extend-type`), never by name string.** Transport kinds and their dial
rules declared **once, beside the loci**, not hardcoded in the type checker. And a **wall**: locus-kind
and transport-kind names may appear only in the locus module (and its one transport registry); any other
*infrastructure* that names a specific locus fails the build. Call sites choosing a locus stay legal.
Loci being *open but discrete* makes the registry a closed set per version — adding a locus becomes a
language update the compiler walks you through, with **exactly one** place to walk to.
