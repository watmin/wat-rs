# DESIGN — Stone 259.S3.4 — per-runner identity & setup (worker-id + per-runner state)

## The goal (unchanged)

Give the brackets pool **per-runner setup**: a resource allocated ONCE when a runner
starts and reused across every item that runner pulls (the canonical case: a DB handle /
connection, opened once per worker, not once per item), plus **which worker am I**
(an index 0..N-1). This is Ruby `Parallel`'s thread-local / `start`+`finish` idiom.

## What the breadcrumb specified — and why grounding broke it

The breadcrumb design: **`bracket::Env <: program::Env`** + a `wat.worker-id` field +
a bracket init-fn whose result lands in `user.bracket`, all read ambiently via the env.
Two disk facts kill it as written:

1. **wat record dispatch is nominal-EXACT (Stone S-B.1, `runtime.rs:11698`): no
   parent-walk.** Only the ROOT `:wat::Record` is an umbrella that accepts any record.
   A slot/return typed `:wat::program::Env` accepts ONLY that exact class. So a
   `bracket::Env` would NOT be accepted where `program::Env` is expected, and
   `program::Env`'s accessors would not fire on it. The `<:` Liskov substitution the
   design assumed is not how wat records work — and changing that is re-litigating the
   deliberate nominal-exact design (a major arc, not this stone).
2. **There is no wat-level "install an ambient env" verb.** Env install is Rust-only
   (`install_program_env` at the seam + `spawn_thread_peer`). A wat-spawned runner cannot
   install a `bracket::Env` for its work-fn to read. Delivering the ambient model needs a
   NEW Rust scoped-install verb + a second thread-local + a `(:wat::bracket::env)` verb +
   a bracket-runner-loop variant. Heavy substrate.

**Deeper finding — worker-id was mis-categorized.** The escape-hatch env (`wat.*`) is for
**kernel facts** — pid, tid, cpu-count: things only the kernel knows about THIS execution.
`worker-id` is **bracket-assigned** (the coordinator hands out 0..N-1) — application data,
not a kernel stamp. Putting it in the `wat.*` env conflated bracket-domain with
kernel-domain. It never belonged in the escape hatch.

## The two paths

### Path A — Ambient (the breadcrumb's model, repaired by nesting)

A separate `bracket::Env` record `[wat.worker-id, user.bracket]` (NOT a `program::Env`
subtype — nesting, since subtyping is nominal-exact), a second thread-local `BRACKET_ENV`,
a `(:wat::bracket::env)` accessor verb, and a NEW Rust scoped-install verb
`(:wat::bracket::with-env env body)` so a wat runner can install it for its loop's extent.
Plus a bracket-runner-loop that runs the init-fn once, installs the bracket::Env, then loops.

- Pros: keeps worker-id "ambient" (a work-fn deep in a call tree reads it without threading).
- Cons: a new record + a new thread-local + TWO new Rust verbs + a runner variant + the
  coordinator assigning worker-ids; and it cements worker-id (application data) into the
  kernel-fact escape-hatch surface. Substantial substrate for a convenience.

### Path B — Closures (RECOMMENDED — pure wat, zero substrate)

Per-runner state is a **closure**. The user supplies `worker-init : i64 -> (I -> O)` — a
function that, given the worker index, returns that runner's work-fn (closing over the
per-runner resource it just allocated). The coordinator calls `(worker-init i)` ONCE per
runner i; that runner uses the returned closure for all its items.

```clojure
(:wat::bracket::map-worker (:wat::spawn::thread) items
  (:wat::core::fn [worker-id <- :wat::core::i64] -> :wat::core::Fn(I)->O
    (:wat::core::let [db (open-conn worker-id)]      ;; per-runner setup — ONCE
      (:wat::core::fn [item <- :I] -> :O             ;; per-item work — closes over db
        (query db item)))))
```

The per-runner resource (`db`) lives in the closure, allocated once in the outer fn,
reused across items by the inner fn. `worker-id` is the outer fn's argument. ZERO new
records, thread-locals, Rust verbs, or nominal-exact conflict — and it puts worker-id
where it belongs (bracket-domain data the bracket hands to the worker).

**Confirmed buildable:** wat closures return closures (`holon.wat:65` returns a
`Fn(f64)->bool`); `Fn(i64)->Fn(I)->O` types are expressible (stream.wat HOFs,
spawn's `Fn()->Record`).

**`map-worker` is the general engine; `map` becomes a thin wrapper.**
The shipped `brackets/map` is exactly `map-worker` with a constant worker-init that
ignores the id:

```
(:wat::bracket::map host items work-fn)
  ≡ (:wat::bracket::map-worker host items (fn [_wid] work-fn))
```

So the coordinator's runner-spawn changes from "all runners share `wf`" to "runner i gets
`(worker-init i)` wrapped for the index round-trip" — a one-line change in the spawn map.
`brackets/map` / `brackets/each` become thin wrappers over `map-worker` / `each-worker`
(constant worker-init). DRY; the proven coordinator is reused, not duplicated.

## Path B decomposition (if chosen)

- **S3.4a** — generalize the coordinator: runner i's work-fn = `(worker-init i)` (called
  once per runner); add `brackets/map-worker<I,O> [host items worker-init:Fn(i64)->Fn(I)->O]`.
  Re-express `brackets/map` as the constant-worker-init wrapper. Probe: per-runner setup
  runs ONCE per runner (not once per item) + worker-id is the runner index.
- **S3.4b** — `brackets/each-worker` + re-express `brackets/each`. (Thin, like S3.3.)
- Names are placeholders — intueri cast at build (`map-worker`? `map-init`? `map/worker`?).

## The one thing Path B does NOT give

A work-fn buried deep in a call tree cannot read worker-id "from the ambient" without it
being captured/threaded — it must close over it (or take it as an arg). That is the only
capability Path A's ambient model adds, and it costs the whole substrate stack above.
Given worker-id is bracket-domain (not a kernel fact), threading/closing it is honest, not
a workaround.

## The four questions — the decision (protocol-mandated)

Run on each candidate, flat YES/NO, atomic.

**Path B — closures (`worker-init : i64 -> (I->O)`)**
- **Obvious? YES** — "outer fn = per-worker setup, inner fn = per-item work" is the standard
  closure-as-state idiom; the form documents itself (the nesting of fns IS the nesting of
  lifetimes).
- **Simple? YES** — one concept (work-fn produced per-runner by a HOF); zero new substrate;
  coordinator change is one line; `brackets/map`/`each` become thin wrappers over the
  general `map-worker`/`each-worker`.
- **Honest? YES** — delivers per-runner setup + worker-id truthfully; the one limit
  (deeply-nested fn must close over worker-id, not read it ambiently) is surfaced; puts
  worker-id where it lives (bracket-domain), no nominal-exact lie.
- **Good UX? YES** — common case stays the flat `(brackets/map host items work-fn)`; the
  per-runner case writes exactly the closure that expresses it.

**Path A — ambient (`bracket::Env` + thread-local + verbs)**
- **Obvious?** borderline-NO (two ambient env layers + install timing).
- **Simple? NO** — braids SIX new pieces (record + thread-local + accessor verb + Rust
  scoped-install verb + runner-loop variant + coordinator worker-id assignment), coupled.

## Verdict — Path B

The compass disqualifies Path A on **Simple** (Obvious + Simple + Honest must hold before
UX is even weighed); Path B passes all four. The one capability Path A buys — reading
worker-id from deep ambient without threading — costs the entire substrate stack, and
worker-id is bracket-domain data that *should* be threaded/closed, not stamped into the
kernel-fact escape hatch. **Build Path B.**
