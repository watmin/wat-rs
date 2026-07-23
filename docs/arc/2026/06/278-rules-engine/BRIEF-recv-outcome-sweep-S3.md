# BRIEF — the recv' OUTCOME WALL sweep (arc 278 S3): wrap the corpus recv' sites in bare `match` over `RecvOutcome`

> **The wall (R53 `VERBO MEO CAPTVS`):** `recv'` no longer returns the bare message `O` and raises on close/crash.
> It returns a **matchable** `:wat::kernel::RecvOutcome<O>` — a masked failure is now structurally unrepresentable.
> Every direct `(recv' p)` use in the corpus is a type error until wrapped. This brief is the mechanical wrap.

## The enum (grounded — `types.rs:1149`)
```
:wat::kernel::RecvOutcome<O> ::= ::Message[msg <- O]        ; a message arrived — the value you used to get
                               | ::Closed[]                 ; a CLEAN EOF (peer done, no error)
                               | ::Lost[cause <- :wat::kernel::Failure]   ; abnormal loss — the reason is HERE
```
`(:wat::kernel::Failure/message cause)` → the reason String. `eprintln`/`epprintln` are **divergent-return**
(`∀R`), so they stand as a match arm without constraining the result type.

## The task, in one sentence
For each broken `.wat`: a `(recv' p)` whose result was used directly (bound + used, or nested) becomes
`(match (recv' p) ((RecvOutcome::Message m) <the-old-use-of-the-result-with m>) (<Lost arm>) (<Closed arm>))`.
The consumer that pinned `recv'`'s type still pins it **through the `::Message` binding `m`** — inference is unchanged.

## The exemplar (PROVEN green — `tests/comms/probe_arc258_recv_infers_from_consumer.wat`)
```clojure
;; BEFORE (broken — addr is now RecvOutcome<Address'>, not Address'):
;;   addr (:wat::kernel::recv' svc)
;;   c    (:wat::kernel::connect' addr)
;; AFTER (owner role — the test is the final caller; surface the cause loudly on Lost):
     r    (:wat::kernel::recv' svc)
     addr (:wat::core::match r
            ((:wat::kernel::RecvOutcome::Message m) m)
            ((:wat::kernel::RecvOutcome::Lost cause)
              (:wat::kernel::eprintln (:wat::kernel::Failure/message cause)))
            (:wat::kernel::RecvOutcome::Closed
              (:wat::kernel::eprintln "recv': svc closed before sending the address")))
     c    (:wat::kernel::connect' addr)
```
`m` flows into `connect'` exactly as `addr` did → `O` binds to `Address'`; the divergent `eprintln` arms fit any result type.

## The FOUR roles — read the CO-LOCATED `.rs` (same basename) to classify
The role is decided by **what the test asserts**, not by the `.wat` alone. Read `<file>.rs` (the `//!`/`///` docs
+ the assertions) first.

1. **OWNER / final-caller** (DEFAULT for tests — the exemplar): the test dials, receives a reply, uses it.
   `::Message m` → the old use of `m` · `::Lost cause` → `(:wat::kernel::eprintln (:wat::kernel::Failure/message cause))`
   · `::Closed` → `(:wat::kernel::eprintln "recv': <peer> closed unexpectedly")`. (Surface the reason — the law.)
2. **STREAM-loop reader** (the test reads N messages in a recursion/loop): `::Message m` → process `m` + recurse ·
   `::Closed` → the clean-exit value (done; the loop's terminal) · `::Lost cause` → `eprintln` the cause.
3. **CLIENT** (rare in tests — a defservice-internal reply read): `::Lost` → reason-free
   `(:wat::kernel::assertion-failed! "…peer lost…" :wat::core::None :wat::core::None)`; `::Message` → inner reply match.
4. **DEATH-PATH-ASSERTING** — ⛔ **STOP, do NOT wrap; surface the file to the orchestrator.** If the co-located
   `.rs` asserts the peer **DIES/crashes** (the old model: *"the child calls assertion-failed! which RAISES; the test
   asserts the raised error"*), then `::Lost` is the **expected, asserted** outcome — a SEMANTIC migration (raise→
   Lost-match), not a mechanical wrap. These are the orchestrator's (R53's payload). Names that smell death-path:
   any `.rs` whose assertion is about a crash/`Failure`/`PeerDeath`/a raised error carrying structured fields.

## Rooms (your dir) — wrap every broken `recv'` site in each
You are given a directory and its list of broken files. In each: find every direct `(recv' p)` use (grep
`recv'`), classify by the co-located `.rs`, wrap per the role. Nested/`let`-bound/tail-position all follow the
exemplar shape (bind `r`, match, bind the result to the `::Message` arm's value). A file may have MULTIPLE recv'
sites — wrap each.

## The RED gate (per file — run it yourself before reporting)
`target/release/wat --check <file.wat>` → **exit 0**, AND the file's error output no longer contains
`RecvOutcome`/`recv'`. If a file has a NON-recv' red (e.g. `ProcessPeer` kwargs-flip, `select'`) leave that red as
is — it is a separate WIP thread; your gate is only "no recv'/RecvOutcome error remains." Report per file:
`GREEN` (wrapped, --check clean) · `STOP:death-path` (surfaced, untouched) · `STOP:<reason>` (ambiguous — surfaced).

## STOP triggers (surface, do not improvise)
1. The co-located `.rs` asserts the peer DIES → **STOP:death-path** (orchestrator handles the semantic migration).
2. The `::Message` binding's use is ambiguous (you cannot tell what the old result fed) → **STOP:ambiguous**.
3. `--check` still shows a `RecvOutcome`/`recv'` error after your wrap → **STOP:wrap-failed** (report the error).

## Weigh
The orchestrator re-runs `--check` on every file you touch and reads the diff. Do NOT run the full test suite; the
per-file `--check` is your gate. The binary is `target/release/wat` (already built — do not rebuild).
