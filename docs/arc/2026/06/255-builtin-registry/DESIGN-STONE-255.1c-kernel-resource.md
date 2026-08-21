# DESIGN STONE — 255.1c-kernel-resource · HOME #7: fifteen verbs, and the taxonomy's first real load test

## The population — `:Resource`'s prose names all fifteen, and all fifteen are still literal arms

```
runtime.rs:6746  drop                 → eval_kernel_drop            body 26662
runtime.rs:6760  HandlePool::new      → eval_handle_pool_new        body 26914
runtime.rs:6761  HandlePool::pop      → eval_handle_pool_pop        body 26983
runtime.rs:6762  HandlePool::finish   → eval_handle_pool_finish     body 27040
runtime.rs:6779  pipe                 → crate::io::eval_kernel_pipe body src/io.rs:1573  ⚠ NOT runtime.rs
runtime.rs:6794  spawn-thread         → (inline block)
runtime.rs:6797  spawn-process        → (inline block)
runtime.rs:6805  after                → eval_kernel_after           body 32923
runtime.rs:6814  close                → eval_peer_close_prime       body 31692
runtime.rs:6819  signal               → eval_signal                 body 31858
runtime.rs:6842  listener             → eval_listener_prime         body 26070
runtime.rs:6843  connect              → eval_connect_prime          body 26256
runtime.rs:6849  accept               → eval_accept_prime           body 26440
runtime.rs:6853  allow                → eval_allow_prime            body 26490
runtime.rs:6854  deny                 → eval_deny_prime             body 26569
```

**Three more table rows dissolve.** These come from the decomposition table's `networking`
(`listener`/`connect`/`accept`), `handles/capability` (`HandlePool::*`, `allow`, `deny`),
`concurrency` (`spawn-thread`/`spawn-process`/`after`/`close`) and `misc` (`pipe`/`drop`/`signal`).
The table has now been wrong in **every stone that tested it** — homes #4, #5, #6 and now #7. It is
not a map; the categories are.

## ★★ THE POINT — this is the builder's precedent-gathering instrument, not just a carve

> **Builder, 2026-08-19:** *"we continue with the names we have as seek failures to classify as we
> move forward."*

`intueri`'s verdict on the taxonomy was recorded and **HELD** (`NOTE-intueri-on-Category-HELD-…`) on
the ruling that a naming argument in the abstract is taste, while a verb that cannot be honestly filed
is data. **Fifteen bodies is the largest single sample the taxonomy has ever faced.**

So this stone has TWO deliverables, and the second is the one that matters more:

1. Fifteen verbs registered.
2. **A strain report** — every verb that had to be *argued* into `:Resource` rather than landing in it.

**A verb that fits only after a paragraph of justification is a FINDING, not a success.** The stone
must not let that be smoothed into a clean scorecard, because the smoothing is exactly how a taxonomy
ships wrong and stays wrong.

### The four strain candidates, named at draw time so the rider cannot quietly resolve them

`:Resource`'s axis is *"acquires, releases, or ADMINISTERS a handle whose lifetime is tracked outside
value scope."* Four of the fifteen test that sentence:

- **`allow` / `deny`** — grant and revoke a CAPABILITY. Is a capability a handle? The prose says
  `:Mutate` was refused for these, which settles what they are NOT, not what they are.
- **`pipe`** — CONSTRUCTS a reader/writer pair. Acquiring implies taking custody of something that
  existed; `pipe` makes one. Is construction acquisition?
- **`after`** — SCHEDULES a timer. The handle is time itself, which no one holds.
- **`drop`** — ⚠ **a documented NO-OP.** Its own prose: *"`drop` is a documented NO-OP — it does not
  force teardown while other references remain."* A verb that administers nothing, filed under a
  category about administering. **A rider deriving from the name alone will get this wrong.**

If any of the four needs more than a sentence, that is the precedent the builder asked for. Report it;
do not resolve it.

## ★ The gate covers TEN of fifteen not at all — and the split is not arbitrary

```
gate LIVE  (5)   pipe · drop · HandlePool::{new,pop,finish}          plain registered TypeSchemes
gate SKIPS (10)  listener · connect · accept · after · close ·        bespoke infer_list arms
                 signal · spawn-thread · spawn-process · allow · deny (check.rs:4003–4245)
```

`doc_arg_ret_types_match_checker_scheme` opens `None => continue`, so the ten with bespoke inference
are skipped exactly as home #5's five were. **The split tracks TYPE COMPLEXITY, not importance:** the
ten are precisely the verbs with parametric or projective types (`peer<I,O>`, `Listener<S,R>`, the
capability types) — the ones hardest to get right are the ones nothing checks.

This home is the first with **mixed** coverage (home #5: none; home #6: all), so it is the first place
a rider could mistake a green gate for whole-home verification. **Five rows are checked. Ten are
documentation.** Each of the ten names its `infer_*` arm as the authority, per the `readln'` shape.

**No stub schemes.** Same rejection as home #5: a stub existing only to be agreed with is a gate
reading a copy of the truth.

## The one contract decision, pinned

**Every row's Category is derived from its own body, and a verb that does not fit is reported, not
filed anyway.** `:Resource` is a claim about fifteen bodies; fifteen body-reads are the check. This is
the stone where the taxonomy is most likely to break, and breaking it here is a **success**.

## Blast radius

```
NEW   src/intrinsic/kernel_resource.rs
EDIT  src/intrinsic/mod.rs   one `mod kernel_resource;` line
EDIT  src/runtime.rs         delete 15 literal arms (+ replacement comments); widen delegates
```

⚠ `pipe`'s body is `crate::io::eval_kernel_pipe` (`src/io.rs:1573`) — already `pub`, no edit needed
there, but a rider assuming every body is in `runtime.rs` will hunt for it. `spawn-thread` and
`spawn-process` are **inline blocks**, not single delegate calls — they need their bodies lifted to
named `pub(crate)` fns or wrapped as-is; the rider reports which it did and why.

No `check.rs`. No `wat/runtime-meta.wat`.

## ⚠ STANDING ORCHESTRATOR STEP — the goldens, and this is the largest shift yet

Fifteen arms is the biggest `runtime.rs` delta of the campaign. The five `.edn` diagnostics fixtures
will move. Procedure unchanged (`DESIGN-STONE-255.1c-kernel-error.md` § standing step): measure
`git diff --numstat`, confirm the structural hunks precede the pinned site and the `fn`→`pub(crate)`
hunks are zero-delta at old − D, confirm `:col` unchanged, bump, verify by floor. The rider is told
its scoped filter cannot see `tests/diagnostics/` so it reports a scoped run, not a floor.

## Progress meter

72 → 87 registered forms. Fifteen arms leave `runtime.rs` — and after this the kernel tier's literal
dispatch is down to the genuine remainder (`here`, `raise!`, `assertion-failed!`, `fn-forms`,
`call-site`, `macro-call-site`, `serve-dispatch-op`, `retag-op`, `address-wire?`,
`require-wire-address`).
