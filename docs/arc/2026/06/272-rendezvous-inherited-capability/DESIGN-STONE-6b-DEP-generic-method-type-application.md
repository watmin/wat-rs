# DESIGN — DEP (arc-232 follow-on): generic-method type-argument application

> Opened 2026-06-16. Grounded against HEAD `611d68e3`. The blocking dependency 6b-ii-β surfaced
> ([DESIGN-STONE-6b-process-launch.md] § "The blocking dep"). Block-and-build it before 6b-ii-β
> ([[feedback_deferred_dep_becomes_necessary_block_and_build]]). Lineage: arc-232 defprotocol →
> arc-246 generic protocol methods (multi-param names parse + dispatch) → arc-267 parametric bounds →
> THIS (apply explicit type-args at a method call + flow type-params into the body's intrinsics).

## Why (the forcing function)

For the `Host/launch` protocol to give a **constant interface across thread/process/remote** (the
narrow-waist / zero-central-edit-per-transport requirement), each tier's `launch` impl must mint its
listener generically: `launch<S,R,St>` whose body calls `(listener' self :S :R)`. That requires two
things wat cannot do today:

1. **Call a generic method with explicit type-args.** `(:P/m<T1,T2> recv args…)` — today resolves as
   `unknown callee` (the probe). Generic *fns* (`foldl<T,Acc>`) accept this; methods don't.
2. **Flow a method's type-params into its body as type-args to an intrinsic.** Inside the impl,
   `(listener' self :S :R)` must resolve `:S`/`:R` to the *instantiated* types, not the literal
   `Path(":S")` (the 4a-probe failure mode).

## Disconfirming probe (RED at HEAD — already run)

`/tmp/tparam_probe.wat` (formalize as `tests/probe_arc232_generic_method_type_application.rs`):
```wat
(:wat::core::defprotocol :user::Mk
  (mk<S,R> [self <- :user::Mk] -> :wat::spawn::Bound<S,R>))
(:wat::core::extend-type :wat::spawn::ThreadOpts :user::Mk
  (mk [self] (:wat::kernel::listener' self :S :R)))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [b (:user::Mk/mk<wat::core::i64,wat::core::i64> (:wat::spawn::thread))] nil))
```
RED: `unknown callee: :user::Mk/mk<wat::core::i64,wat::core::i64>`. GREEN target: it type-checks + runs,
the `(listener' self :S :R)` body minting a `Bound<i64,i64>`.

## The two seams (grounded)

- **Seam 1 — call-head resolution.** `is_resolvable_call_head` (runtime.rs ~1469 region;
  `preregister_protocol_names` feeds it) finds the protocol by the stem before the last `/`, but the
  method name `mk<i64,i64>` isn't matched to the registered method `mk` — the `<…>` suffix isn't
  stripped, and the type-args aren't captured. Fix: strip the `<type-args>` suffix to match the method
  name AND carry the parsed type-args to the dispatch/inference. Mirror however generic-fn call heads
  already do this (the `foldl<T,Acc>` path).
- **Seam 2 — type-param substitution in the method body.** Generic-fn instantiation lives in
  `src/check/env.rs`; a generic fn body's type-params resolve to the call's instantiated types. The
  defprotocol-method check path must build the same substitution environment so the impl body's `:S`/`:R`
  (and `Bound<S,R>` return) resolve to the instantiated types. Runtime dispatch is at `runtime.rs:4930`
  (arc-232 232.3) — confirm it carries the type-args to the right `listener'` runtime arm (`listener'`
  dispatches on the concrete host VALUE, so the runtime may already be fine; the gap is the CHECKER).

## Decomposition

- **DEP-i** — formalize the RED probe (`probe_arc232_generic_method_type_application`), verify RED at HEAD.
- **DEP-ii** — Seam 1: method call-head resolves `:P/m<T,T>` (strip suffix + capture args). Re-run probe;
  expect it to advance from "unknown callee" to a type-param-substitution error (Seam 2).
- **DEP-iii** — Seam 2: type-param substitution into the method body; probe GREEN.

## Out of scope = rejected

- **Type-inference of the method's type-args from value args** (so the `<T,T>` could be omitted): the
  reshaped `launch` has NO value arg carrying `S`/`R` (state0 carries `St` only), so explicit type-args
  are required here. Inference is a separate, later convenience — CUT.

Pairs [[feedback_deferred_dep_becomes_necessary_block_and_build]] + arc-232/246/267 + DESIGN-STONE-6b-process-launch.
