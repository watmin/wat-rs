# DESIGN STONE — 255.1c-retire-kernel-drop · a verb that was never alive

## The finding, and how it was proven

`:wat::kernel::drop` is **unreachable from the wat language**, and the registry carve proved it.

```
:wat::core::drop      38 corpus call sites   the seqable — take/drop-while's sibling. ALIVE.
:wat::kernel::drop     0 corpus call sites   the handle no-op. THIS ONE.
```

**Born `ce8ec656`, 2026-04-19** — *"try-recv + drop: non-blocking poll and scope-based close marker."*
`git log -S':wat::kernel::drop'` across the **entire history** of `wat/`, `wat-tests/` and
`wat-scripts/` returns **empty**. It was not retired by a migration and not orphaned by a rewrite: it
has never had a caller, in four months.

**Its co-born sibling `try-recv` is already gone** from the runtime. `drop` is the survivor of a
retired pair.

**And the world it was minted for is gone.** Raw `Sender`/`Receiver` were the channel surface in arc
170 slice 1c (`3c737ee5`); Stones C1/C2 replaced them with typed `ThreadPeer<I,O>` /
`ProcessPeer<I,O>`, and the Peer became the handle a user holds. `drop` was the *scope-based close
marker for raw channel ends* — it never followed the migration, and its argument type stopped being
constructable when the Peers landed. `:wat::kernel::Channel<T>` (`wat/kernel/channel.wat:49`) is a
**typealias** naming the pair's type; **no verb builds one.** `close`, on Peers, is the successor.

## ★ WHY THIS IS NOT "zero callers ⇒ delete"

This project's standing rule is that **zero consumers is not evidence of deadness** —
`[[feedback_no_consumers_does_not_mean_dead]]`, earned when `insert-all` turned out to be good
capability nobody had adopted yet. A count would have proven nothing here either.

**What proved it was a REACHABILITY argument, produced by a gate.** `purity_mandated_examples` holds
that a `Pure`+`Deterministic` row owes a **runnable** `@example`. Home #7 declared `drop` honestly —
it is a no-op, so Pure+Deterministic is right — and then no example could be written, because *its
argument cannot be constructed*. The missing example was the symptom; unreachability was the defect.

So the discriminator is: **`insert-all` COULD be called and wasn't. `drop` CANNOT be called.** The
first is unadopted capability; the second is not capability at all.

## ⛔ THE LOAD-BEARING HAZARD — TWO FUNCTIONS NAMED `infer_drop`

```
src/check.rs:9448                   fn infer_drop(…)   ← THE KERNEL ONE. Delete this.
src/collection/infer.rs:1069        infer_drop(…)      ← THE SEQABLE ONE. 38 callers. DO NOT TOUCH.
```

and two dispatch arms that differ only by qualification:

```
check.rs:4253  ":wat::kernel::drop" => infer_drop(…)                              ← delete the arm
check.rs:4431  ":wat::core::drop"   => crate::collection::infer::infer_drop(…)    ← LEAVE IT
```

`collection/infer.rs`'s `infer_drop` type-checks `(:wat::core::drop xs n)` — arc **118.2a**, the lazy
`Seqable<T> × i64 → Stream<T>` path that arc 118 spent four months building and inscribed this
morning. **Deleting by name would kill it.** Same name, different subject — the exact class recorded
as `[[feedback_an_adjacent_implementation_is_not_the_subject]]`.

## Blast radius

```
src/runtime.rs        the literal arm (+ its held-back comment) and `eval_kernel_drop`'s body
src/check.rs:4253     the `:wat::kernel::drop` inference arm
src/check.rs:9448     the LOCAL `fn infer_drop` and its doc
src/check.rs:19481    a comment referencing it
wat/runtime-meta.wat  `:Resource`'s prose — it NAMES `drop` in its member list (`:159`) and carries
                      the parenthetical "`drop` is a documented NO-OP…" (`:161`). Both go.
src/intrinsic/kernel_resource.rs   the module doc's held-back explanation (17 mentions)
```

**NOT a `.wat` corpus migration** — zero corpus call sites, so no wat-fix codemod is required (R21
governs *structural rewrites across many `.wat` files*; this is one prose edit in one file).

**MUST NOT TOUCH:** `src/collection/infer.rs`, `check.rs:4431`, and anything spelled
`:wat::core::drop`.

## The one contract decision, pinned

**Retire the verb, not the machinery beneath it.** `Value::wat__kernel__Sender`/`Receiver` and the
crossbeam plumbing stay — they are how peers are built internally. This stone removes the *wat-facing
verb* that nothing can call, and nothing else.

## Adjacent findings — NAMED, NOT FIXED

- **`:wat::kernel::Channel<T>` is a typealias for a pair no verb constructs.** It may be orphaned by
  the same migration that stranded `drop`. Its own stone; do not touch it here.
- **`:wat::kernel::close` has zero direct corpus call sites** (pattern positive-controlled before the
  zero was quoted). It is a **different case**: its `Peer` argument IS constructable, so `close` is
  *exercisable but unexercised* — unadopted, not unreachable. Conflating the two is precisely how
  good capability gets deleted. Not this stone's business.

## Acceptance

The registry gains nothing; the language loses a verb no program could ever have written. `:Resource`
goes from a fifteen-member prose claim to a **fourteen-member one that is true** — and home #7's
fourteen carved rows become the whole population.
