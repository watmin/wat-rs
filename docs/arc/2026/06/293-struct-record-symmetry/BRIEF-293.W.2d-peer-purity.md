# BRIEF — 293.W.2d: PEER-TYPE PURITY — the wall becomes ZERO runtime code

> **Executor: one sonnet LEAF.** Orchestrator drew this (four-questions ratified **R**) + weighs the kill forced-clean.
> Work ONLY in `wat-rs/`, NEVER worktrees. Commit nothing — leave the tree green for the orchestrator to weigh.

## The work (one paragraph)
Make the wire wall **structural** so the **2a/2c runtime+gate code is annihilated** (the wall ends as zero runtime
code). The mechanism (option R, four-questioned over the DESIGN's literal `ConnPeer'` rename): **`Peer'<I,O>` keeps its
name and becomes the wire-capable peer — its `I,O` must be `:Pure` (well-formedness, reusing `is_pure_type`)** — and a
**new `ThreadSelfPeer'<I,O>` (any I/O, in-locus)** is the escape hatch for thread self-peers that carry impure values
(reply-`Sender`s, etc.). Then bare-`Peer'` `send'` is statically pure-safe; impure peers are a distinct in-locus type
that can never unify into a wire slot; the ops (`send'`/`recv'`/`select'`/`poll'`) go **purity-blind**; and the **2a
runtime guards + the 2c `send'`-site gate DELETE**. `make-channel` (always thread-tier/crossbeam) **drops its purity
gate** → unlocks the ledgered `:svc::Request` ignore. `NonPortableCapture → ImpureCapture` (tier-aware).

## Read in order (the design + the rooms)
1. `293/DESIGN-293.W § 293.W.2b`/contract + the breadcrumb dep-order — the wall's purely-compile-time scope.
   **NOTE:** the DESIGN's literal "`ConnPeer'`/`ThreadSelfPeer'` rename" is SUPERSEDED by **R** (this brief) —
   four-questioned: keeping bare `Peer'` as an any-I/O parent leaves `send'` ambiguous and the runtime guard
   undeletable; **R** (Peer'=pure + ThreadSelfPeer') is the only shape that achieves zero-runtime-code.
2. The peer-io extractor + the prior-self spec: `src/check.rs:11446–11488` (`project_peer_io`; the comment at
   11476–11485 IS the 2d statement — "statically gateable once a type-level tier split exists").
3. The 2c gate to DELETE: `src/check.rs:11605` (`infer_send_prime`), the `is_wire` purity branch `11663–11690`.
4. The 2a guards to DELETE: `src/edn_shim.rs:892` (`StructOnWire` `EdnReadErrorKind`) + `decode_trusted_wire`
   (`2734`); `reject_non_portable_on_wire` (grep it; the outbound send guard).
5. `make-channel`'s purity gate to DROP: `src/check.rs:10573` (`if !is_pure_type(&t, …)`).
6. `NonPortableCapture`: `src/closure_extract.rs:87` (+ Display `108`, + construction sites).
7. The `Peer'<…>` blast radius (the crawl): `wat/spawn.wat:130/170/195/246/249`, `wat/bracket.wat:21/119`,
   `wat/test.wat:798`, `wat/service.wat:327/339/353/384/1098` (macro type-string builders); `src/check.rs`
   producers `self-peer` (5206), `connect'`/`accept'` (5219/5220), `peer-pair'` (10701), `socket-pair'` (10743),
   `infer_select_prime` (11879), `infer_poll_prime` (12016).
8. The IGNORE LEDGER: `294/CLOSE-SEQUENCE § THE IGNORE LEDGER` row 1 (`:svc::Request` / `deftest_svc_test_svc_assert_state`).
9. Prior comparable: **293.W.2b** (`76d1d890`, the purity strike) — same `is_pure_type` predicate, one tier up.

## The categorization (THE load-bearing decision — get this right)
Every bare `Peer'<I,O>` site is one of:
- **WIRE / wire-capable → stays `Peer'`** (gains the pure-`I,O` well-formedness; already satisfies it): `connect'`,
  `accept'`, `peer-pair'`, `socket-pair'` producers; `ServiceEvent.:Connection [peer <- Peer'<I,O>]`; the service
  `Vector<Peer'<R,S>>` collection; `Launched.handle <- Peer'<Sh,Lu>` (the lineage peer — Sh=Admin, Lu=Launched, both
  pure data).
- **IN-LOCUS thread self-peer → `ThreadSelfPeer'`** (any I/O): the `self-peer` producer (`check.rs:5206`); the thread
  program self-peer sigs `wat/spawn.wat:195` (`prog <- [Peer'<S,R> :-> nil]`), `wat/bracket.wat:21/119`,
  `wat/test.wat:798`; `poll'`'s `self` arg (accept BOTH `Peer'` and `ThreadSelfPeer'` for self).
- **macro type-string builders** (`wat/service.wat`): the defservice macro builds `"wat::kernel::Peer'<…>"` strings —
  these construct CONNECTION-peer types (wire) → keep `Peer'`. (Verify each is a client/connection peer, not a self-peer.)

## Decomposition (build between each; the gate is the meter)
### A — mint `ThreadSelfPeer'<I,O>` (the in-locus escape hatch)
Register the type (mirror how `Peer'`/`Thread'`/`Process'` are registered; it is a 2-param peer). It does **NOT** derive
`Peer'` (Peer' is pure; ThreadSelfPeer' is any-I/O — deriving would violate the bound). The ops accept it directly (step C).

### B — `Peer'<I,O>` well-formedness: `I,O` must be `:Pure`
Add an `is_pure_type` check on `Peer'`'s two type args wherever a `Peer'<I,O>` type is formed/validated — minimally the
producers (`connect'`/`accept'`/`peer-pair'`/`socket-pair'`) and the type-validation path for the `wat::kernel::Peer'`
head. A `Peer'<:Impure,…>` is a hard type error naming the impure arg ("a wire peer carries only pure data; use
`ThreadSelfPeer'` for an in-locus peer that holds resources"). **STOP-1:** if `Launched.handle`'s `Sh`/`Lu` can be
impure (i.e. an admin/launched type that holds a resource), STOP — the lineage handle would need tier-awareness, a
deeper issue; surface it.

### C — the ops go purity-blind + accept the full peer family
`project_peer_io` (`check.rs:11466`) and `infer_select_prime`/`infer_poll_prime`: add `wat::kernel::ThreadSelfPeer'`
(and keep `Peer'`/`Thread'`/`Process'`) to the accepted peer heads. The ops do **NO** purity check — the peer TYPE now
guarantees it (Peer'/Process' pure by well-formedness; ThreadSelfPeer'/Thread' in-locus, any).

### D — DELETE the 2c send'-site gate
`infer_send_prime` (`check.rs:11605`): remove the `is_wire` purity branch (`11663–11690`) and the `is_wire` return from
`project_peer_io` (it becomes a 2-tuple `(I,O)`). `send'` now just unifies the payload with the peer's `I` — and since a
wire peer's `I` is pure by construction, an impure payload to a wire peer is an ordinary unify error. No special gate.

### E — DELETE the 2a runtime guards
Remove `reject_non_portable_on_wire` (the outbound `send'` guard) + its call sites; remove the `StructOnWire`
`EdnReadErrorKind` + the inbound `decode_trusted_wire` reject of a bare top-level impure value. Retirement-table the
heads. **STOP-2:** if `decode_trusted_wire` rejects bad BYTES (untrusted-input defense), that is OUT OF SCOPE (the
user's validation problem) — but the STRUCT-on-wire reject specifically is the dead 2a guard; delete only that.

### F — `make-channel` drops its purity gate
`check.rs:10573`: remove the `is_pure_type` rejection. `make-channel` is thread-tier (crossbeam, in-process) → a thread
channel carries impure fine. **STOP-3:** verify `make-channel` is ONLY thread-tier (no wire `make-channel`); if a wire
channel constructor routes through here, surface it (don't blindly drop).

### G — `NonPortableCapture → ImpureCapture` (tier-aware)
Rename in `closure_extract.rs` (the variant + Display + construction sites + the `binding_name` comment). Make it
tier-aware: a THREAD-spawn closure may capture impure (shared memory); only a PROCESS/remote-spawn closure rejects an
impure capture. (If the closure-extract path already knows its spawn tier, gate on it; if not, surface what it has.)

### H — un-ignore `:svc::Request` + the ledger row
With make-channel's gate gone, `deftest_svc_test_svc_assert_state` passes. Remove the `(:wat::test::ignore …)` +
the `// ⛔ IGNORE-LEDGER(293)` marker in `wat-tests/service-template.wat`; delete the ledger row in `CLOSE-SEQUENCE`;
confirm the test is GREEN (not skipped). **The IGNORE LEDGER must end EMPTY.**

### The RED probe (write FIRST, verify it disconfirms, then build)
`tests/comms/probe_arc293_W2d_peer_purity.{rs,wat}`: a `send'` of an **impure** value over a **wire** peer
(a `connect'`'d `Peer'` or a `Process'`) is a **COMPILE** error (structural). RED at the 2b HEAD: it COMPILES (the 2c
gate only covers `Process'`, not bare `Peer'`; the impure-over-wire is caught only at RUNTIME) → so the probe's
"expect a compile error" assertion FAILS at HEAD. GREEN after 2d: the impure arg can't unify a pure-`I` wire peer →
compile error. Also assert the POSITIVE: a `ThreadSelfPeer'` carrying impure I/O type-checks (in-locus), and a thread
`make-channel` of an impure payload type-checks.

## Blast radius (bounded)
`src/check.rs` (project_peer_io / send' / select' / poll' / the producers / make-channel gate), `src/edn_shim.rs`
(StructOnWire + decode reject), `src/closure_extract.rs` (rename), `src/types.rs` (register ThreadSelfPeer'),
+ the ~10 `wat/` peer sites + `wat-tests/service-template.wat`. NO new concept beyond `ThreadSelfPeer'`. NO change to
the purity axis (293.W.2b) or the holder.

## STOP triggers (numbered above): impure lineage `Handle` I/O (B/STOP-1); a bad-bytes vs struct-on-wire distinction in
decode (E/STOP-2); a non-thread `make-channel` (F/STOP-3). On any STOP: halt, leave the tree building, surface the gap.

## Gate + floor
`cargo nextest run --release` → **0 failed, and the IGNORE LEDGER is EMPTY** (the `:svc::Request` test now GREEN, not
skipped → 93 skipped, not 94). `cargo build --release` clean. Read your own diffs end-to-end (deleting runtime code —
confirm nothing live depended on the deleted guards beyond the wall).

## EXPECTATIONS (the scorecard — fixed before the strike)
| what | command | expected |
|---|---|---|
| the 2a guards are GONE | `grep -rn 'reject_non_portable_on_wire\|StructOnWire' src/` | 0 (annihilated) |
| the 2c gate is GONE | `grep -n 'is_wire' src/check.rs` (in infer_send_prime) | 0 (the purity branch deleted) |
| ThreadSelfPeer' exists | `grep -rn "ThreadSelfPeer'" src/ wat/` | present (type + the migrated sigs) |
| impure-over-wire is a COMPILE error | `probe_arc293_W2d_peer_purity` | RED→GREEN (compile-rejected) |
| :svc::Request unlocked | `cargo nextest run --release deftest_svc_test_svc_assert_state` | PASS (not skipped) |
| ImpureCapture | `grep -rn 'NonPortableCapture' src/` | 0 |
| floor + ledger empty | `cargo nextest run --release` | 0 failed; **93 skipped** (the one ignore is gone) |

Runtime estimate: 60–90 min (a type-system change + guard deletion + the producer categorization cascade). Trap-door:
the `Peer'`-pure well-formedness may RED legitimate pure-but-generic `Peer'<I,O>` sites where `I`/`O` are unresolved
type vars — `is_pure_type` is conservatively false for an unresolved var; gate the well-formedness on RESOLVED types
only (mirror `infer_send_prime`'s existing `!matches!(resolved, Var)` guard at `check.rs:11669`).
