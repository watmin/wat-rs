# BRIEF — complete parametric surfaces (arc 170 C2, Gaps 1+2)

> **A substrate type-checker stone that unblocks the C2 `bracket/uses` macro.** The arc-170 C2
> parametric-surface support (`293.4b`/`4e-pre.ii`/`K1b`) landed **half-built**: it handles the two
> *concrete* cases and neither *abstract* one. The C2 consumer (the `Dialable`-checker) surfaced it —
> `ALIVS ARGVIT`. Two guarded additions to `src/check.rs`, both extending code already there.
> **Executor: sonnet shadowdancer, weighed by the orchestrator's own re-run.**

---

## The two gaps (both PROVEN by the RED gate)

The RED gate `scratchpad/probe-gaps12-red.wat` fails **right now** with exactly these two errors, and
must **freeze clean** after this stone (its content is reproduced in the committed test below):

1. **Gap 2 — `ReturnTypeMismatch`**: `(Dialable/coord d)` where `d : Dialable<probe::Echo::Op,probe::Echo::Reply>`
   (an abstract parametric-surface param) produces `Address'<S,R>` (the surface's *raw* type params),
   not `Address'<probe::Echo::Op,probe::Echo::Reply>`.
2. **Gap 1 — `TypeMismatch`**: a concrete `echo'::Handle` does not satisfy a `Dialable<probe::Echo::Op,probe::Echo::Reply>`
   **param** — `assignable` has no `(concrete-Path actual, parametric-surface expected)` rule.

## The contract (pinned)

**Zero regression.** Monomorphic surfaces (`s.type_params` empty) and concrete-satisfier resolution
must be **byte-for-byte unchanged** — both fixes are new branches guarded so the existing paths never
change. And **soundness, not permissiveness**: after the fix, `echo'::Handle` satisfies
`Dialable<Echo::Op,Echo::Reply>` but must STILL NOT satisfy `Dialable<Kv::Op,Kv::Reply>` (else the whole
swap-gate erases). The negative test below is the load-bearing proof of that.

---

## Read in order (the rooms)

1. **`scratchpad/probe-gaps12-red.wat`** — the RED gate. Run `./target/release/wat --check` on it: 2
   errors now (Gap 2 line 21, Gap 1 line 26). This is your target — it must freeze clean when done.
2. **`wat/capability.wat:44–46`** — `(:wat::core::defsurface :wat::capability::Dialable<S,R> :nature :Struct
   :features [(coord [self <- Dialable<S,R>] -> :wat::kernel::Address'<S,R>)])`. The parametric surface;
   `coord`'s declared return is `Address'<S,R>` (the surface's own type params). The per-service
   auto-emit registers the FULL-ARGS edge `<fqdn>::Handle <: Dialable<Op,Reply>`.
3. **`src/check.rs:5786–5928`** — the surface-method call-site check. **Gap 2 lives at 5852–5871**: the
   return is resolved by looking up a *concrete* satisfier scheme (`<concrete>/coord`, line 5860); an
   abstract `Dialable<A,B>` receiver finds none → `unwrap_or` (5870) falls back to the raw
   `member_ret_raw` = `Address'<S,R>`. The `rename`/mapping machinery to reuse is right below at
   **5883–5908**.
4. **`src/check.rs:15333` (`assignable`)** — **Gap 1**. Branches exist for `(Parametric actual, Path
   expected)` (15345 Peer'; 15367 extend-type edge) but **none for `(Path actual, Parametric expected)`**.
   Mirror the 15367–15372 branch with roles flipped.
5. **`src/check.rs:15298–15319` (`nature_floor_ok`)** — the nature-floor check every extend-type-edge
   satisfaction must clear; call it from the new Gap-1 branch (as the existing branches do).

---

## The fixes (fill them; reuse the machinery that's already there)

### Gap 2 — `check.rs:5852–5871`: instantiate the return from an abstract parametric-surface receiver
Today: if `s.type_params` non-empty, look up `<recv_concrete>/<method>`; on miss, fall back to raw.
Add, in the miss path (before the raw fallback): if the receiver type reduces to the surface itself
parametrized — `TypeExpr::Parametric { head == <surface fqdn>, args }` with `args.len() == s.type_params.len()`
— build `mapping = { s.type_params[i] → args[i] }` and `rename(&member_ret_raw, &mapping)` (and the
`extra_param_types_raw`). This is the *same* `rename` + `HashMap<String,TypeExpr>` shape already at
5885–5908; you're substituting the surface's own type-params from the receiver's concrete args instead
of from a concrete satisfier's binding. Guard: only when the receiver IS that surface parametrized;
every other receiver keeps the current fallback (byte-identical).

### Gap 1 — `assignable` (`check.rs:15333`): a concrete type satisfies a parametric-surface param
Add a branch for `(TypeExpr::Path ap, TypeExpr::Parametric { head, args })` where `head` names a
`Surface`: return `is_subtype(ap, &format_type(&e)) && nature_floor_ok(&a, &format_type(&e)... , types)`
— i.e. the FULL-ARGS extend-type edge `echo'::Handle <: Dialable<Echo::Op,Echo::Reply>` exists AND
clears the nature floor. Mirror the existing edge branch at 15367–15372 (which does the same for
`(Parametric actual, Path expected)`), roles flipped. **CONFIRM FIRST (do not assume):** that
`is_subtype(ap, "wat::capability::Dialable<probe::Echo::Op,probe::Echo::Reply>")` actually finds the
auto-emitted edge — grep how the per-service `extend-type <Handle> Dialable<Op,Reply>` edge is keyed
(`register_extend_type_surface_impls` / the `typesub` table); if it's keyed by the full parametric
string, the branch is a one-liner over `is_subtype`. If the edge is keyed differently, STOP and report
the actual key shape.

---

## The gate — committed tests (promote the RED probe)

`tests/types/probe_arc170_parametric_surface_param.rs` + fixtures:
- **POSITIVE** (`..._ok.wat`, = the RED gate content): the abstract-`Dialable` path freezes clean —
  `(defn takes-dialable [d <- Dialable<Echo::Op,Echo::Reply>] -> Address'<Echo::Op,Echo::Reply> (Dialable/coord d))`
  + a caller passing a raw `echo'::Handle`. `startup_from_file` → `Ok`.
- **NEGATIVE — the soundness proof** (`..._wrong_param.wat.bad`): a `takes-dialable [d <- Dialable<Kv::Op,Kv::Reply>]`
  called with an `echo'::Handle` → `startup_from_file` → `StartupError::Check` `TypeMismatch` (STRUCTURAL:
  `expected` `Dialable<Kv…>`, `got` `echo'::Handle` — or the satisfaction-precise message). This proves
  the completion is PRECISE — a handle satisfies only its own parametric surface, so the swap-gate holds.
- Structural asserts (match the error enum; no `contains`/`starts_with`; no inlined wat form in a `.rs`
  string — drive via `startup_from_file` on the fixture, like `probe_arc170_wrong_service_compile_error.rs`).

## STOP triggers (rejection — ship nothing, surface the gap)

1. The auto-emitted `<Handle> Dialable<Op,Reply>` edge is NOT keyed by the full parametric string (so
   `is_subtype` can't find it) → STOP, report the actual key shape (the fix location shifts).
2. The Gap-2 branch changes ANY monomorphic-surface or concrete-satisfier result (a floor regression) →
   STOP; the guard is wrong.
3. The negative soundness test does NOT fail (a handle satisfies the wrong `Dialable<A,B>`) → STOP; the
   completion is permissive, not precise — the swap-gate would be dead.

## How to work / expectations

- `cargo build --release`; `cargo nextest run` (targeted then the full floor). Run everything
  **FOREGROUND**; never `&`/background. A mid-edit rust diagnostic is a PHANTOM — a clean build + a suite
  that ran N tests compiled.
- Do NOT commit; leave uncommitted (this stone commits on its own once weighed — it is independent of
  the in-flight Strike-1 tree, which stays intact).

| what | command | expected |
|---|---|---|
| RED gate now fails | `./target/release/wat --check scratchpad/probe-gaps12-red.wat` | 2 errors (before your change) |
| build | `cargo build --release` | clean |
| positive: abstract-Dialable freezes clean | `cargo nextest run -p wat -E 'test(parametric_surface_param)'` (ok) | PASS |
| negative: wrong param still rejected | same (wrong_param) | PASS — TypeMismatch (soundness) |
| full floor | `cargo nextest run --release` (FOREGROUND) | prior floor + these; 0-new (modulo the 1 known `no_inlined_wat`) |

Report: the `check.rs` diff (both branches), the confirmed edge-key shape, the RED-gate before/after,
the scorecard with real results, the full Summary line, any STOP.
