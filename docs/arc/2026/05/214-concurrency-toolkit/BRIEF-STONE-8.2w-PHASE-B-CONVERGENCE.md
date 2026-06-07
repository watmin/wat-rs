# BRIEF — Stone 8.2w Phase B: the vigilia convergence sweep

> The trio-completion FULL VIGILIA (14 wards) reported: 4 L1 + ~20 L2 across
> the home. Every finding below was weighed by the orchestrator against the
> live tree; every item is FIX or an earned rune — nothing is deferred. The
> stamp lands when this sweep is green and the orchestrator's re-greps converge.

## Required reading
1. `docs/ZERO-MUTEX.md` (the doctrine the home embodies).
2. `src/services/` — all four files, whole.
3. This brief's per-item citations are exact; trust them after confirming each
   site reads as described.

## The work (by file, then tests; every site enumerated)

### A. `src/services/peer.rs`

A1 (perspicere). Mint at top, after the imports:
```rust
/// Reply sender routed back to a registered caller — Ok(R) means the
/// handle call COMPLETED; Err carries the failure the caller surfaces.
pub type ServiceReplySender<R> = crate::comms::thread::Sender<Result<R, String>>;
/// The loop-owned routing table: thread-id → that thread's reply sender.
type ReplyRegistry<R> = std::collections::HashMap<ThreadId, ServiceReplySender<R>>;
/// The sender half of a service peer's input channel.
pub type ServiceInputSender<R> = crate::comms::thread::Sender<ServiceMsg<R>>;
```
Use them at: ServiceMsg::Register's payload (~:27), ServicePeer.input_tx
(~:36), the reply_registry local (~:83-86).

A2 (circumspicere F1 — the EDN-only contract, sites honored now; the
mechanical gate stays tracked at
docs/arc/2026/04/109-kill-std/NOTE-edn-only-rust-stdio-enforcement.md).
The three bare-string diagnostics (~:108-110, ~:117-121, ~:156-159) become
single-line TAGGED EDN on fd 2:
```rust
eprintln!("#wat.substrate/Diag{{:site \"{}-peer\" :msg \"Req field[0] is not i64\"}}", service_label);
eprintln!("#wat.substrate/Diag{{:site \"{}-peer\" :msg \"Req is not a Struct\"}}", service_label);
eprintln!("#wat.substrate/Diag{{:site \"{}-peer\" :msg \"handle failed\" :error {:?}}}", service_label, format!("{}", e));
```
(Exact field spelling: `#wat.substrate/Diag{:site ... :msg ...}` — one line,
parseable EDN; the `{:?}` on the pre-formatted error string gives a quoted,
escaped EDN string.)

### B. `src/services/client.rs`

B1 (exigere L1+L2 — the 1f-era future-tense docs now LIE; rewrite to
present/past truth):
- Module-section doc ~:7-9 ("slices 1f-β/γ/δ ship...") → past tense: they
  shipped; the orchestrator (spawn-thread eval arm, runtime.rs) populates
  ThreadIO in production; tests populate it directly.
- ~:13 ("later slices call these...") → present: "The spawn-thread/reap
  orchestrator (runtime.rs) calls these in production; tests call them
  directly."
- ~:32 ("Slice 1f-γ will populate...") → present: "Allocated by
  `next_thread_id`; the spawn orchestrator assigns one per thread."
- ~:35-37 (ThreadIO doc "Populated by ... (slice 1f-γ); for slice 1f-α...")
  → present: "Populated by the spawn orchestrator via
  `register_thread_with_services`; tests populate via `install_thread_io`."
- ~:103-105 ("Slice 1f-γ will call this...") → present tense.
- ~:114-116 ("Slice 1f-γ calls this when reaping...; slice 1f-α tests...")
  → present, no slice numbering.
- ~:181 ("0 is reserved as a 'no thread' sentinel for future use") →
  present fact: "0 is never allocated (the counter starts at 1); no current
  consumer reads 0 as a sentinel."

B2 (intueri). `RuntimeServices` doc ~:141: drop the "Three-Sender carrier
per BRIEF Q5 + Q-carrier." opening; replace with: "Holds the three
universe-resident service peer input channels; accessed via
`sym.runtime_services()` rather than ThreadIO so the peers' lifetimes tie
to `Arc<RuntimeServices>`, not to every per-thread cell."

B3 (intueri + struere — the double-name collapses). DELETE
`uninstall_ambient_stdio`; `take_ambient_stdio` is the one name
(Option::take semantics). Its doc absorbs the test-cleanup note ("tests
call this between rows to keep the reused worker thread's cell clean").
Sweep ALL callers (src/spawn.rs, src/process_stdio.rs, ~20+ test files —
`cargo check --all-targets` names every site; grep
`uninstall_ambient_stdio` to zero).

B4 (sequi rune — earned, not a suppression):
```rust
// rune:sequi(performance-counter) — uniqueness-only id allocator; no domain
// state crosses threads through the counter (the allocated tid travels
// VISIBLY in Register/Req messages); threading an AtomicI64 through every
// spawn-site signature trades real legibility for monadic purity. Documented
// bound: ZERO-MUTEX.md § honest caveats (hot atomic counters).
```
placed above `static NEXT_THREAD_ID` (~:186).

B5 (temperare L3 + secare corroboration): `fetch_add(1, Ordering::SeqCst)`
→ `Ordering::Relaxed` (~:190) with a one-line WHY: uniqueness-only, no
happens-before required.

B6 (perspicere). Mint:
```rust
/// Receiver of write-acks from a write-service peer (stdout/stderr).
pub type WriteAckRx = crate::comms::thread::Receiver<Result<(), String>>;
/// Receiver of read-replies from the stdin peer (the line, or the error).
pub type ReadReplyRx = crate::comms::thread::Receiver<Result<String, String>>;
```
Use at ThreadIO fields (~:65, ~:74, ~:89) and drop the now-redundant
turbofish at the three `pair::<...>()` sites (~:217, ~:233, ~:244 — the
binding annotations carry the type). RuntimeServices fields (~:157, ~:162,
~:167) become `ServiceInputSender<String>` / `ServiceInputSender<()>`
(import from peer).

B7 (perspicere runes). Above both thread-local cells (~:100, ~:314):
```rust
// rune:perspicere(intentional-structure) — RefCell<Option<T>> is the
// canonical thread_local interior-mutability idiom; the structure shows the
// reader exactly how borrow_mut/take interact.
```

B8 (conformare — the #189 span-debt closes in-home).
`with_thread_io` (~:127) gains `span: &Span`; the ServiceNotRunning arm uses
`span.clone()` instead of `Span::unknown()`. All three verbs pass
`list_span`.
`register_thread_with_services` (~:205) gains `caller_span: &Span`; its
three ChannelDisconnected sites use `caller_span.clone()`. Callers: the
spawn-thread eval arm in src/runtime.rs (~:19190 area) passes its in-scope
span/list_span; the freeze.rs boot (~:333) passes `&Span::unknown()` with a
one-line comment: "boot-time registration — no user form exists; genuinely
spanless-by-domain."

### C. `src/services/verbs.rs`

C1 (solvere — un-duplicate require_one_arg): make
`src/edn_shim.rs`'s `require_one_arg` `pub(crate)`; DELETE the verbs.rs copy
(~:16-31); `use crate::edn_shim::require_one_arg;`. (If the two bodies
differ in any way, STOP-1.)

C2 (solvere — the Req schema, one source of truth):
```rust
/// Build a write-service Req {thread-id, line} — THE positional contract
/// the peer's field[0] extraction and the wat defstructs share.
fn build_write_req(type_name: &str, thread_id: ThreadId, line: String) -> Value
```
println (~:65-71) and eprintln (~:126-132) call it. stdin's single-field Req
(~:241-246) stays distinct.

C3 (temperare — the clones die): `line.clone()` at ~:69 and ~:130 become
moves (bind `line` into the closure / pass by value into build_write_req —
nothing uses `line` after).

C4 (conformare — the 14 Span::unknown() sites): every error production in
the three eval arms threads `list_span.clone()` — the closure captures
`list_span` by reference. Sites enumerated by the conformare cast: println
~:59-62/75-77/82-85/86-88; eprintln ~:120-123/136-138/143-146/147-149;
readln ~:235-238/250-253/262-265/267-270/273-276/277-284. (The exact line
numbers will shift as you edit — the INVARIANT is: zero `Span::unknown()`
remains in verbs.rs; grep is the gate.)

### D. `src/freeze.rs`

D1 (circumspicere F1): the three join-error eprintln sites (~:193-196,
~:202-205, ~:211-214) become tagged EDN:
`#wat.substrate/Diag{:site "process-runtime-drop" :msg "<service> service peer join error" :error "..."}`
(same one-line EDN shape as A2).

D2: the stdin `reply_of` closure + spawn sites: update for B8's signature
change if touched; pass spans per B8.

### E. `src/services/mod.rs`

E1 (circumspicere F4 — the over-promise softens): contract §4's "The
canonical rig is `MiniUniverse`..." → "The reference rig is `MiniUniverse`
in tests/wat_arc170_slice_1f_alpha_helpers.rs — new service tests either
reuse it or rebuild its true-universe shape (live loop + fd-backed resource
+ real Register exchange); the doctrine's enforcement is review-time (the
brief-authoring checklist)."
E2: re-export the new aliases (`ServiceReplySender`, `ServiceInputSender`,
`WriteAckRx`, `ReadReplyRx`) in the flat pub-use block.

### F. `wat/kernel/services/stdin.wat`

F1 (exigere): line ~24's "(see the #[ignore]'d probe_diag_typealias_leniency
nursery probe, banked for arc 255)" gains the rune form on its own comment
line: `;; rune:exigere(attested-arc) — arc 255 (docs/arc/2026/06/255-builtin-registry/DESIGN.md); the leniency probe un-ignores when 255 makes undeclared type keywords check errors`.

### G. `docs/arc/2026/05/214-concurrency-toolkit/DESIGN-SLICE-8-SERVICES-UNIVERSE-RESIDENT.md`

G1 (conferre): the Stone 8.1 bullet's "rewritten as the pure tagged loop
over `spawn-program' :thread`" → "rewritten as the pure handle fn driven by
the Rust service loop (`spawn_service_peer`, src/services/peer.rs — the
8.1w/8.2w lift made the loop universe-resident in Rust; the wat half is the
handle alone)". DESIGNs are living docs.

### H. New tests (the invariant gates — circumspicere F2 + F3)

H1 (F2 — THE FIELD-ORDER GATE, the cast's best find): a new test in
`tests/nursery/` (e.g. `gate_arc214_service_record_field_order.rs`):
freeze a skeleton world; via the symbol table's type registry, read the
three `*Service::Req` defstructs + `StdInService::Rep`; assert
field[0].name == "thread-id" for all three Reqs AND field[1].name == "line"
for StdInService::Rep + StdOutService::Req/StdErrService::Req's field[1] ==
"line". The test's doc names the coupling: "the peer's positional
extraction (peer.rs field[0]; freeze.rs stdin reply_of field[1]) and the
wat defstruct order are ONE contract; a reorder must be a red build, never
a silent mis-route." (Discover the type-registry read API from
sym.types() / the TypeDef shape — check src/types.rs; if field names are
not introspectable at that surface, STOP-2 and report what IS available.)

H2 (F3 — the guard arms): two tests (same file or alpha helpers): spawn a
peer with a real handle; send `ServiceMsg::Req(Value::i64(99))` (not a
Struct), then Register + a VALID Req and assert the valid one still
round-trips (the loop survived the continue); repeat with a Struct whose
field[0] is a String. Assert no hang (the valid Req's reply arrives).

## Gates

1. `cargo test --release --lib -p wat` → 0 fail.
2. `cargo test --release --test nursery` → no new reds beyond the 4 parked
   arc-255 (the new gate tests GREEN).
3. `cargo test --release --test wat_arc170_slice_1f_alpha_helpers` → green.
4. `cargo check --all-targets` → 0 errors (the double-name sweep complete).
5. `cargo clippy --release --lib -p wat` → zero findings in src/services/.
6. Greps at zero: `Span::unknown` in src/services/verbs.rs;
   `uninstall_ambient_stdio` anywhere; `BRIEF Q5` in src/services/.

## STOP triggers (rejection criteria)
- STOP-1: edn_shim's require_one_arg differs in behavior from the verbs copy.
- STOP-2: the type registry does not expose defstruct field names (H1).
- STOP-3: any test outside the known baseline reds for an untraceable reason.

## Constraints
- Commit NOTHING — the orchestrator scores, then commits.
- The probe files are read-only ground truth.
- Work only in /home/watmin/work/holon/wat-rs/.
