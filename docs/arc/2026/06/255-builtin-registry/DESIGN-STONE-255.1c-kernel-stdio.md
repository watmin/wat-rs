# DESIGN — STONE 255.1c-kernel-stdio: HOME #3, and the registry's first EFFECTFUL rows

## Why this slice, and why it is 6 arms and not 49

**`:wat::kernel::` is not a family. It is a tier**, and carving it whole would braid seven independent
domains into one stone. Measured at HEAD — 49 arms:

| concern | arms |
|---|---|
| **stdio** | `println` `pprintln` `eprintln` `epprintln` `readln'` `read-frame` — **`5704–5714`, contiguous** |
| concurrency | `spawn-thread` `spawn-process` `send` `try-send` `recv` `close` `select` `poll` `after` |
| networking | `listener` `connect` `accept` `peer-pid` `peer-process` `peer-wire?` |
| signals | `sigusr1?` `sigusr2?` `sighup?` `reset-sigusr1!` `reset-sigusr2!` `reset-sighup!` |
| errors | `LociDiedError/{message,to-failure}` · `Failure/{message,location}` |
| handles / capability | `HandlePool::{new,pop,finish}` · `allow` · `deny` |
| misc | `drop` `here` `pipe` `raise!` `assertion-failed!` `fn-forms` `call-site` `stopped?` `serve-dispatch-op` `retag-op` |

Each row is a different reason to change, a different test surface, and in several cases a different
module. **This stone takes the stdio row only.** The rest are their own stones and are named here so
the cut reads as a decomposition, not an omission.

## ★ THE POINT: the purity axis has never been falsifiable

Every registered row today is **48 `Pure` / 2 `Preserving` / 0 `Effectful`.** That is exactly where
determinism sat before home #2 — a set whose every member takes the same value cannot falsify the
contract (R59 `NISI FRANGAS, NIHIL PROBAS`).

stdio is effectful by definition: these six write to fd 1/2 or read fd 0. **They are the registry's
first `Effectful` rows**, and they turn a decorative column into a measured one.

And there is a gate waiting for them: **`pure_declared_matches_is_effectful_op`
(`src/intrinsic/mod.rs:601`)** cross-checks a row's declared purity against
`runtime::is_effectful_op`. That function (`runtime.rs:25164`) classifies **by prefix** —
`head.starts_with(":wat::kernel::")` ⇒ effectful — so it already has an opinion about all six. The
cross-check has never once seen a row it could disagree with. After this stone it has.

## The one contract decision, pinned

**`@Purity Effectful` on all six, and the cross-check must AGREE without either side being edited to
make it agree.**

If a declared `Effectful` disagrees with `is_effectful_op`, that is a finding to report — not a
prompt to adjust the declaration until the test passes. The whole value of the cross-check is that
two independently-derived answers meet; making one copy the other destroys it.

## What the hoist bought this stone

`255.1c-guard` moved the registry check above the literal table, so **a missed arm deletion is now
dead code that clippy names**, rather than a silently-shadowed registration that leaves dispatch
unchanged while looking finished. Three kernel arms (`serve-dispatch-op` ×2, `retag-op`) sit at
`3907`/`4994`/`5014` — above where the guard used to be. They are **out of this stone's scope**, but
before the hoist their existence made any kernel carve hazardous. It no longer does.

## Rooms the brief must send the rider to — the bodies are NOT inline

Home #2's brief was wrong about this and cost a delta: it assumed handler bodies lived inline in
`runtime.rs` because home #1's did. **Measured for this slice:** all six stdio arms delegate to
`crate::services::` —

```rust
":wat::kernel::println" => crate::services::eval_kernel_println(args, list_span, env, sym).map_err(Into::into),
```

so `src/services/` is necessarily in the blast radius, exactly as `src/time.rs` was for home #2.
`src/kernel/` exists but holds `address.rs`/`listener.rs`/`peer.rs`/`spawn.rs` — the concurrency and
networking side, **not** stdio. A rider sent to `src/kernel/` looking for `println` finds nothing.

## The hazard this family has and `time` did not

stdio routes through the **service tier**, and this project has a documented three-rule
thread-vs-process classification (recovery doc FM 7-ter): a body that calls `println`/`readln` shares
the parent's fd 0/1/2 in a thread context and needs a process boundary to be captured.

Registration must not change routing — the handler fn is the same fn, reached through the registry
instead of a literal arm. But it is the first carve where "the handler is the same" is a claim worth
checking rather than an obvious truth, so it gets a STOP.

## Out of scope — affirmatively cut

The other six kernel concerns above. The blanket-accept (`255.1b-iv`). `core::i64`/`core::f64`.
`rete/purity.rs`'s divergence (`255.3`, already recorded). No `@Category` variant is minted — if
none of the six existing ones fits an stdio verb, that is a STOP and a builder ruling, exactly as
`Clock` and `Arithmetic` were.

## Progress meter

47 registered production names → **53**. Six arms leave `runtime.rs`. The honest claim is not that
the megafile shrank; it is that **the registry carries a row whose purity is not `Pure`, and a
cross-check that could always have disagreed finally had the chance.**
