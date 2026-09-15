# NOTE — the checker may admit what the runtime refuses (a LEAD, not a finding)

**Filed 2026-09-15.** ⛔ **UNMEASURED.** One instance is confirmed; the class size is unknown and the
census is not cheap. Filed so it is not rediscovered from scratch.

## The one confirmed instance

`a-peer-wait-can-be-bounded-too` (`7686bea24`) widened `:wat::kernel::recv-by-deadline` to accept a
`Peer`. The surprise was in its SCORE: **`src/check.rs` needed a comment only.**

> *"`infer_recv_by_deadline` already used `project_peer_io` (Peer was a **check-time admit** and a
> **runtime refusal**)."*

So the wall measured the day before —

```
recv-by-deadline: expected owner handle ((Thread :- [I O]) | (Process :- [I O])),
                  got :wat::kernel::Peer
```

— **contradicted the primitive's own type inference.** The checker said a `Peer` was fine; only the
runtime refused. The stone therefore did not add a capability. **It closed a divergence.**

⭑ And note what that cost while it stood: `child-main`'s `TimedOut` arm was written, type-checked, and
**dead**, so a slow owner hung every service at birth — a hang the type system had implicitly promised
was a bounded wait.

## Why this is a class worth censusing, and why it is not cheap

`conferre`'s whole subject is spec/implementation divergence: *"where they disagree, one of them is
wrong."* Here the two disagree about a **primitive's domain**, and the disagreement is invisible until
someone drives the refused shape at runtime.

```
RuntimeErrorKind::TypeMismatch construction sites in src/   583
```

⚠ **Most of those 583 are legitimate** — a value genuinely of the wrong type at runtime, where the
checker could not have known. The interesting subset is only where **the checker's inference ADMITS a
shape the runtime REFUSES**, and separating them means comparing each guard against the inference for
its own op. **That is a real census, not a grep.**

⭑ Three sites do sit in the same family as the confirmed one — `select`'s tier guards
(*"expected Thread"*, *"expected Process"*, *"expected Peer"*), which arc 109's NOTE already recorded as
*"peers[1] has wrong tier (expected Process)"*. **Whether the checker admits those mixed sets is
unmeasured.**

## The shape a census would take

For each kernel primitive with a bespoke inference in `check.rs` (`infer_recv_prime`,
`infer_recv_by_deadline`, `infer_serve_dispatch_op`, `infer_close_prime`, … — they are grouped around
`check.rs:4676`), compare the **types its inference admits** against the **shapes its runtime guard
refuses**. A divergence is either a missing check-time refusal or an unnecessary runtime one.

⛔ **And note which way the confirmed instance went:** the runtime was *stricter* than the checker, so
the divergence surfaced as a **crash** rather than as an unsound admission. A sweep should expect both
directions and say which it found.

## Why it is filed rather than drawn

The builder's standing mandate is finding service-to-service ungraceful crashes, and this is a source of
exactly those — **a crash the compiler implied could not happen.** But the class size is unknown, and
this session has repeatedly shown that a census drawn before its instrument is validated produces a
number nobody can rule on (2276 → 51; 679 → 67). **Whoever takes this should build the comparison for
two or three primitives first and report the hit rate before sweeping all of them.**
