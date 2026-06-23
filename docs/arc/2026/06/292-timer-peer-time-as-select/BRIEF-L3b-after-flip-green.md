# BRIEF — arc 292 L3-β: flip `after` to PeerKind + wire the process timer through select' (→ GREEN)

**You are a LEAF executor.** Do this ONE integration across the named rooms. Do NOT spawn
subagents. Do NOT redesign — the shape is fixed below. If a piece needs a file/мechanism
not named here, STOP and report. The tier-open type machinery (`Timer'<O>` + the `unify`
fusion) already SHIPPED (`b958732d`); you consume it.

## The work (one paragraph)
`(:wat::kernel::after <PeerKind> <Duration> <msg>)` is the locked surface: arg0 is a
`:wat::program::PeerKind` (`:thread`|`:process`), the timer is a selectable IN the `select'`
vector, and `after` returns the tier-open `Timer'<O>` (already in the checker). Flip `after`'s
eval + infer from the old spawn-locus to PeerKind; add a `Timer` variant to `ProcessSelectable`
and wire it through the four process-peer runtime arms (rustc's non-exhaustive-match errors
will list them); rewrite the timer probes to the new form and make them GREEN.

## Rooms — read in order
1. `wat/program.wat:14-16` — `:wat::program::PeerKind` (`:thread`|`:process`). This is arg0's type.
2. `src/kernel/spawn.rs` — `enum ProcessSelectable { Spawned(ProcessPeerBundle) }` (from L1, ~line 329). ADD `Timer(crate::comms::process::Receiver<String>)`.
3. `src/comms/process.rs` — `pub fn timer(duration, msg_frame: Frame) -> io::Result<Receiver<String>>` (L2, SHIPPED). You CALL this for the process timer.
4. `src/runtime.rs:24996-25130` — `eval_kernel_after`. Today it matches arg0 `class_fqdn == "wat::spawn::ThreadOpts"`/`"ProcessOpts"`. The thread branch already builds a crossbeam timer peer (~25085-25125). FLIP to match the `PeerKind` VALUE.
5. `src/runtime.rs` — the 4 process-peer downcast arms (send' :24010, recv' :24217, close' :24430, select' :24700). Adding `ProcessSelectable::Timer` makes their matches non-exhaustive → rustc lists each; handle per §"the 4 arms".
6. `src/check.rs:11118-11193` — `infer_kernel_after`. Today returns `Thread'<nil,O>` always. CHANGE: check arg0 conforms to `:wat::program::PeerKind`; return `Timer'<O>` (O = inferred msg type). No tier dispatch.
7. `src/services/client.rs` (`current_program_env`, the `(:wat::program::env)` arm) + `wat/program.wat` — find how to read `wat.peer-kind` off the env for the idiom probe (likely `(:wat::program::Env/wat.peer-kind (:wat::program::env))` — VERIFY the real accessor).
8. `wat-tests/timer-after.wat`, `wat-tests/timer-after-process.wat`, `wat-tests/timer-tier-open.wat` — the probes to rewrite + un-ignore.

## The 4 process-peer runtime arms (when `ProcessSelectable::Timer(rx)` is added)
- **select' (`:24700`)** — today builds parallel `output_rxs` + `err_rxs` from `Some(ProcessSelectable::Spawned(bundle))`. Change `err_rxs` to `Vec<Option<&Receiver<String>>>`: `Spawned` → push `(&bundle.peer.output, Some(&bundle.err))`; `Timer(rx)` → push `(rx, None)`. On `Recv { result: Err(e) }`: if `Some(err)` → `classify_peer_error(&e, err)` (as today); if `None` (a timer) → `ServiceEvent::Closed` (a timer never crashes — no err channel). On `Ok(edn_str)` → `ServiceEvent::Message` (unchanged).
- **send' (`:24010`)** — `Timer(_)` → clean RuntimeError "cannot send to a timer peer (timers are select'-only)".
- **recv' (`:24217`)** — `Timer(_)` → clean RuntimeError "recv' on a timer peer is not supported; place it in a select' set".
- **close' (`:24430`)** — `Timer(_)` → clean RuntimeError "close' on a timer peer is not supported (it is consumed by select')". (Drop-on-error is fine; do NOT call `.peer.wait()`.)

## eval_kernel_after — flip to PeerKind
Replace the arg0 `class_fqdn` ThreadOpts/ProcessOpts match with a match on the `PeerKind`
enum VALUE (arg0 evaluates to a `:wat::program::PeerKind` — an `EnumValue` with variant
`thread`/`process`; check how enum values are represented, mirror an existing enum-variant
match):
- `:thread` → build the crossbeam thread timer (REUSE the existing thread-branch code that
  builds `kernel::peer::Thread{ output: thread::timer(dur,msg), ... }` + wraps as
  `THREAD_PEER_TYPE_PATH` — it already exists for the old ThreadOpts path).
- `:process` → encode `msg` to a wire frame (`value_to_edn_with(&msg, sym.types())` + `'\n'`,
  the same framing `comms::process` uses), call `comms::process::timer(std_dur, frame)`, wrap
  the `Receiver<String>` as `ProcessSelectable::Timer(rx)`, wrap in
  `Arc<ThreadOwnedCell<Some(ProcessSelectable::Timer(rx))>>`, `make_rust_opaque(PROCESS_PEER_TYPE_PATH, cell)`.
- non-PeerKind arg0 → clean TypeMismatch "expected :wat::program::PeerKind (e.g. :wat::program::PeerKind::process)".

## infer_kernel_after — accept PeerKind, return Timer'<O>
- arg0: infer; assert assignable to `:wat::program::PeerKind` (else TypeMismatch).
- arg1: Duration (unchanged).
- arg2 (msg): infer → O.
- return `TypeExpr::Parametric { head: "wat::kernel::Timer'", args: vec![O] }`.

## Probes — rewrite + un-ignore (the GREEN gate)
- `timer-after.wat` (thread): arg0 `(:wat::spawn::thread)` → `:wat::program::PeerKind::thread`; Vector elem-type ascription → `:wat::kernel::Timer'<wat::core::keyword>` (or drop the ascription if inference suffices). Keep it GREEN (un-ignored — it was green before; keep it green).
- `timer-after-process.wat`: arg0 → `:wat::program::PeerKind::process`; same ascription; REMOVE its `:wat::test::ignore`.
- `timer-tier-open.wat`: REMOVE its `:wat::test::ignore` (it's already in the new form).
- **Idiom probe (best-effort):** add a probe using the env-grab `(:wat::kernel::after (<read wat.peer-kind off (:wat::program::env)>) (Millisecond 50) :tick)`. If the accessor is clean, make it green. If reading `wat.peer-kind` off the env is awkward/needs a new verb, STOP and report the accessor situation (do NOT mint a new convenience verb without surfacing).

## Blast radius
`kernel/spawn.rs` (Timer variant), `runtime.rs` (eval after + 4 arms), `check.rs` (infer after), the 3-4 probe `.wat` files, possibly `comms/process.rs` only if `Receiver` needs a tiny accessor (report if so). NO new files. The `unify`/`Timer'` machinery is DONE — don't touch it.

## STOP triggers
1. If the env `wat.peer-kind` read needs a new verb/mechanism, STOP + report (idiom probe is best-effort).
2. If wiring `ProcessSelectable::Timer` needs `comms/process.rs` changes beyond a trivial accessor, STOP + report.
3. If a probe stays RED for a reason not in this brief, STOP + report the exact checker/runtime error.

## Gate (run yourself; report real output)
```
cargo build 2>&1 | tail -8
touch tests/test.rs
cargo test --test test -- after_delivers after-delivers tier_open tier-open 2>&1 | grep -E "timer|after|tier|test result" | head
cargo test --no-fail-fast 2>&1 | grep -E '\.\.\. FAILED$' | sort -u | wc -l    # floor must stay ~218
```
HARD GATE: the thread + process literal-PeerKind probes + the tier-open probe run **green**;
floor unchanged (~218, the stdlib flap ±1). Report the idiom-probe outcome separately.

## Report back (raw facts — do NOT commit, I weigh)
1. `git diff --stat`.
2. The timer probes' result lines (thread / process / tier-open — must be ok).
3. The idiom (env-grab) probe outcome (green / STOPped-with-accessor-report).
4. Total FAILED count vs ~218 + whether any NON-timer test changed status.
5. Any STOP trigger hit.
