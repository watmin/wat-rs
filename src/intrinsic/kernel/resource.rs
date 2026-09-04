//! `:wat::kernel::` resource intrinsics — arc 255 home #7
//! (255.1c-kernel-resource). Fourteen verbs — `HandlePool::new`,
//! `HandlePool::pop`, `HandlePool::finish`, `pipe`, `spawn-thread`,
//! `spawn-process`, `after`, `close`, `signal`, `listener`, `connect`,
//! `accept`, `allow`, `deny` — all `@Category Resource`: `:Resource`'s
//! whole population (`wat/runtime-meta.wat`, the `:Resource` prose,
//! ~line 156-162): *"Acquires, releases, or ADMINISTERS a handle whose
//! lifetime is tracked outside value scope … NOT what data moves through
//! the handle (that is `:Message`), NOT where the handle came from."*
//! `drop` — the fifteenth-named, never-registered candidate this doc
//! used to carry a long held-back analysis for — was **retired**, not
//! carved: stone 255.1c-retire-kernel-drop found it unreachable from
//! wat (zero corpus callers in four months; its only accepted argument
//! types have no live constructor). This home's fourteen rows are now
//! `:Resource`'s whole population.
//!
//! All fourteen delegate to the SAME `crate::runtime::eval_*` fn (or, for
//! `pipe`, `crate::io::eval_kernel_pipe`; for `spawn-thread`/`spawn-process`,
//! `crate::kernel::spawn::eval_kernel_spawn_*_prime`) that already existed
//! as a literal-match arm in `runtime.rs` — see `kernel/mod.rs` for the
//! tier-wide "bodies do not live here" claim this home is an instance of.
//!
//! ## ★★ THE STRAIN REPORT — fourteen bodies, the largest sample this
//! taxonomy has faced
//!
//! Every row below was read at the body, not derived from the name. One
//! genuinely needed a paragraph before it landed:
//!
//! - **`after`** (body `runtime.rs:32904`–`33057`) — the brief's framing
//!   ("the handle is time itself, which no one holds") does not survive
//!   the body-read and would have filed this WRONG if trusted. `after`
//!   does not hand back an abstraction over time; it constructs a REAL
//!   handle — a `crossbeam` timer `Receiver` (thread tier, futex-backed)
//!   or a `timerfd`-backed `process::Receiver` (process tier, an actual
//!   OS fd) — wraps it in a `Peer'` (`ThreadOwnedCell`/`RustOpaque`,
//!   `PEER_TYPE_PATH`), and returns THAT. The caller holds the peer, not
//!   time; its lifetime is administered by the same `close'`/Drop
//!   machinery as any other peer. Once corrected, this lands the same way
//!   `listener`/`connect`/`accept` do — the paragraph was needed to reject
//!   the naive reading, not to force a fit.
//!
//! `pipe` and `allow`/`deny` were also named as candidates but land
//! WITHOUT argument once read:
//!
//! - **`pipe`** (`src/io.rs:1573`) — "constructs rather than acquires" is
//!   a wat-syntax-level observation (`(:wat::kernel::pipe)` takes no
//!   existing value as an argument) that does not survive the body: the
//!   body's first move is `libc::pipe2(2)`, an OS syscall that claims two
//!   fresh kernel-tracked fds and hands them to the caller — textbook
//!   resource ACQUISITION (the same shape as `open(2)`/`socket(2)`; this
//!   is the canonical example the "RAII" idiom is named after). `Drop`
//!   administers the close. No paragraph needed once the body, not the
//!   call-site arity, is the evidence.
//! - **`allow`/`deny`** (`runtime.rs:26471`/`26550`) — "is a capability a
//!   handle?" is the wrong question once the body is read: neither verb
//!   manipulates a free-standing capability value. Both take the
//!   caller-held `Listener'` HANDLE (downcast + `SocketListener` inner)
//!   and mutate ITS allow-set (`sl.allow(pid, …)` / `sl.deny(pid, …)`).
//!   The thing being administered is the listener the caller already
//!   holds, not an abstract permission token — the third disjunct
//!   ("administers a handle") applies directly, no capability-as-handle
//!   argument required. `:Mutate` was refused for these (per the axis
//!   prose) because the observable change is to WHO may `connect'`
//!   through the listener, not a value the caller can read back and
//!   compare — a `:Resource` administration, not a `:Mutate` on data.
//!
//! The remaining ten — `HandlePool::{new,pop,finish}`, `spawn-thread`,
//! `spawn-process`, `close`, `signal`, `listener`, `connect`, `accept` —
//! land as the textbook shape: mint, release, or operate on a handle the
//! caller holds (or is handed) whose lifetime the runtime tracks outside
//! the immediate expression. `signal` is worth one sentence: it neither
//! acquires nor releases the `Process'` peer it is given (`with_ref`, not
//! `with_mut` + `take` — `close'` remains the only consumer) — pure third
//! disjunct, administering a live handle by delivering a signal through
//! it.
//!
//! ## ★ Gate coverage — CORRECTED from the design stone's claim
//!
//! The design stone states gate-LIVE as `pipe ·
//! HandlePool::{new,pop,finish}` (4). Reading `check.rs` end to end
//! confirms the split: **gate LIVE (4): `pipe`, `HandlePool::{new,pop,
//! finish}`** — plain registered `TypeScheme`s (`check.rs:18028,18169,
//! 18187,18199`) — **gate SKIPS (10): `listener`/`connect`/`accept`/
//! `after`/`close`/`signal`/`spawn-thread`/`spawn-process`/`allow`/`deny`**,
//! bespoke `infer_list` arms (`check.rs:4003–4256`).
//! `doc_arg_ret_types_match_checker_scheme` opens `None => continue`, so
//! it silently skips all ten; a green gate here verifies FOUR rows, not
//! fourteen. Each of the ten below carries a `//` (not `///`) maintainer
//! comment naming its `infer_*` fn as the real authority, per the
//! `kernel_message.rs` shape. **No stub `TypeScheme`s were minted to
//! manufacture coverage.**
//!
//! (`drop` — retired by stone 255.1c-retire-kernel-drop — used to be
//! this section's third case: check-time presence via a bespoke
//! `infer_drop` arm no fixed-arity `TypeScheme` could express. That
//! analysis is now moot; see the header note.)
//!
//! ## `spawn-thread` / `spawn-process` — NOT inline blocks
//!
//! The design stone flagged these as inline blocks needing bodies lifted
//! to named `pub(crate)` fns. Reading `runtime.rs:6794`/`6797` (pre-edit)
//! shows both are already single delegate calls —
//! `crate::kernel::spawn::eval_kernel_spawn_thread_prime` /
//! `_process_prime` — wrapped in braces only because the match arm's RHS
//! is a block expression, not because logic lives inline. Both delegates
//! are ALREADY `pub fn` in `src/kernel/spawn.rs` (`:453`, `:539`) — wider
//! than `pub(crate)`, so no visibility edit was needed there either. No
//! lifting was required; this home wraps the existing `pub` fns exactly
//! like every other row wraps its `runtime.rs` delegate.
//!
//! ## ⚠ One residual `cfg(test)` gate failure — STOP-4, reported not fixed
//!
//! A structural gap in shared, pre-existing test infrastructure that
//! this home is the FIRST to exercise — not a defect in the fourteen
//! rows' registration or category placement, and outside the declared
//! blast radius (`kernel_resource.rs` / one `mod.rs` line /
//! `runtime.rs`'s arms+visibility). Not hacked around: no purity was
//! falsified and no doc-string was contorted to force a match.
//!
//! (This section used to carry a second item — `purity_mandated_examples`
//! on `drop`, whose `@Purity Pure` mandated a RUNNABLE `@example` that
//! could never be written because `drop`'s argument type had no live
//! constructor reachable from wat source anywhere in the corpus. That
//! body-read finding — "`drop` appears to be unreachable dead code at
//! the wat-language level" — is what stone 255.1c-retire-kernel-drop
//! acted on. The gate failure is now RESOLVED by deletion, not fixed.)
//!
//! 1. **`doc_arg_ret_types_match_checker_scheme` — `pipe`.** `pipe` is
//!    the FIRST `#[wat_intrinsic]`-registered row anywhere in the
//!    corpus whose scheme's `ret` is a non-empty `TypeExpr::Tuple`
//!    (`check.rs:18028`'s `Tuple(vec![IOWriter, IOReader])`).
//!    `typeexpr_to_doc_string` (`src/intrinsic/mod.rs:469`) has a
//!    special case ONLY for the EMPTY tuple (→ `:wat::core::nil`); its
//!    own comment says so explicitly: *"no registered intrinsic returns
//!    [a non-empty tuple], so a spelling for that case would be invented
//!    rather than verified."* Every other case falls through to
//!    `format!("{:?}", other)` — Rust `Debug` output, which contains
//!    spaces. The `@ret` directive's OWN grammar (`crates/wat-doc/src/
//!    lib.rs`: `@ret <type> <desc>`) forbids whitespace inside the type
//!    token — so no `@ret` string can EVER equal a `Debug`-rendered
//!    non-empty tuple; this is not a wording problem, it is a
//!    structural mismatch between the doc grammar and the fallback
//!    renderer that only a Tuple-rendering arm in `typeexpr_to_doc_string`
//!    can close. That fn is outside this stone's declared blast radius
//!    (`EDIT src/intrinsic/mod.rs — one mod kernel_resource; line`).
//!
//! `cargo nextest run --release -E 'test(/intrinsic::tests::/)'` is RED
//! on exactly this one; every other scoped test (including
//! `declared_purity_vs_effectful_by_prefix_census`,
//! `all_see_fqdns_resolve_to_registered_intrinsics`, and
//! `yields_type_matches_fn_arg_param`) is green.

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, SymbolTable, Value};

/// `(:wat::kernel::HandlePool::new name handles)` → `(:wat::kernel::HandlePool :- [T])`.
/// Builds a pool of N handles of the same type. `name` surfaces in error
/// messages; the pool drains as callers `pop` and asserts empty at
/// `finish` — FOUNDATION's claim-or-panic discipline.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Resource
/// @arg     name :wat::core::String the pool's name, surfaced in pop/finish error messages
/// @arg     handles (:wat::core::Vector :- [T]) the handles to pool, in claim order
/// @ret     (:wat::kernel::HandlePool :- [T]) the built pool
/// @example-norun (:wat::kernel::HandlePool::new "workers" handles) #=> #wat.kernel/HandlePool{}
// Registered `TypeScheme` — `check.rs:18169` — gate LIVE.
//
// Deciding line for `@Category Resource`: `runtime.rs:26895`
// `eval_handle_pool_new` mints a fresh bounded `crossbeam_channel`,
// pre-fills it from `handles`, and drops the sender so the channel's
// emptiness tracks the pool's drain state — a fresh handle-pool ACQUIRED
// and returned, whose lifetime `pop`/`finish` administer thereafter.
//
// Deciding line for `@Purity Effectful` / `@Determinism Deterministic`:
// allocates real synchronization state (a channel with lock-free
// multi-consumer semantics); given the same `handles` vector, the pool's
// content and claim order are a pure function of the input — no external
// actor, no wait.
#[wat_intrinsic(":wat::kernel::HandlePool::new")]
pub(crate) fn eval_handle_pool_new(
    name: &WatAST,
    handles: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_handle_pool_new(&[name.clone(), handles.clone()], env, sym, list_span)
}

/// `(:wat::kernel::HandlePool::pop pool)` → `:T`. Claims one handle from
/// the pool. Empty pool → a `MalformedForm` naming the pool: callers are
/// expected to pop exactly the count they committed to at construction.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Resource
/// @arg     pool (:wat::kernel::HandlePool :- [T]) the pool to claim from
/// @ret     :T the claimed handle
/// @example-norun (:wat::kernel::HandlePool::pop pool) #=> handle-0
// Registered `TypeScheme` — `check.rs:18187` — gate LIVE.
//
// Deciding line for `@Category Resource`: `runtime.rs:26964`
// `eval_handle_pool_pop` — `rx.recv()` claims (acquires custody of) one
// pre-pooled handle; the sender was dropped at construction so `recv`
// never blocks (immediate `Ok`/`Err`). Administers the pool's drain
// state one claim at a time.
//
// Deciding line for `@Purity Effectful` / `@Determinism Deterministic`:
// mutates the pool's internal channel state (one fewer handle available);
// single-owner discipline (no concurrent popper) makes the claim order —
// and therefore the outcome for a given call sequence — a pure function
// of construction order, not an external race.
#[wat_intrinsic(":wat::kernel::HandlePool::pop")]
pub(crate) fn eval_handle_pool_pop(
    pool: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_handle_pool_pop(std::slice::from_ref(pool), env, sym, list_span)
}

/// `(:wat::kernel::HandlePool::finish pool)` → `:()`. Asserts the pool is
/// fully drained (no orphaned handles) and returns `:()`. Orphans →
/// `MalformedForm` naming the pool and the orphan count — FOUNDATION's
/// Pipeline Discipline rule 2, catching a mis-counted handle budget
/// before any thread runs.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Total         Unreviewed
/// @Category      Resource
/// @arg     pool (:wat::kernel::HandlePool :- [T]) the pool to check is drained
/// @ret     :wat::core::nil always `:()` on success
/// @example-norun (:wat::kernel::HandlePool::finish pool) #=> #wat.core/nil{}
// Registered `TypeScheme` — `check.rs:18199` — gate LIVE.
//
// Deciding line for `@Category Resource`: `runtime.rs:27021`
// `eval_handle_pool_finish` — reads `rx.len()` and asserts it is zero.
// Administers the pool's lifecycle by CLOSING it out (the checkpoint
// between "handles claimed" and "safe to run threads"), even though the
// body itself only reads, never mutates.
//
// Deciding line for `@Purity Pure`: `rx.len()` is a read with no mutation
// and no external call. That half stands.
//
// ⊘ `@Determinism` CORRECTED by the orchestrator, 2026-08-19. The rider
// declared `Deterministic` and flagged it as a dissent-worthy judgment call;
// `purity_mandated_examples` then fired, because a Pure+Deterministic row owes
// a RUNNABLE `@example` and none could be written. The gate was right, and the
// missing example was the SYMPTOM — the mis-declared axis was the defect.
//
// The rider's own justification was "single-owner discipline means no
// concurrent party can change the pool between calls". That is a USAGE
// CONVENTION, not a fact about what the body can observe — the weakest rung,
// and declaring an axis on it is the class this arc exists to delete. `rx.len()`
// reads a live crossbeam queue depth: two calls holding the SAME handle can
// return different answers, and this body RAISES on one and returns Unit on the
// other. Same shape as `sigusr1?` reading an `AtomicBool` (home #4) and
// `time::now` reading the wall clock — both `Nondeterministic`.
//
// By the rider's OWN stated criterion — "does the return value depend on an
// external actor's timing/state not fixed by the call's own arguments?" — this
// is Nondeterministic. It applied that test to `accept`/`close`/`signal` and
// missed its own instance.
#[wat_intrinsic(":wat::kernel::HandlePool::finish")]
pub(crate) fn eval_handle_pool_finish(
    pool: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_handle_pool_finish(std::slice::from_ref(pool), env, sym, list_span)
}

/// `(:wat::kernel::pipe)` → `:(wat::io::IOWriter, wat::io::IOReader)`.
/// Creates a fresh Unix pipe via `libc::pipe2(2)` with `O_CLOEXEC`. Write
/// end first (producer), read end second (consumer). `Drop` closes both.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Resource
/// @ret     (:wat::core::Tuple :- [:wat::io::IOWriter :wat::io::IOReader]) the fresh pipe's write and read ends
/// @example-norun (:wat::kernel::pipe) #=> #wat.core/Tuple[#wat.io/IOWriter{} #wat.io/IOReader{}]
// Registered `TypeScheme` — `check.rs:18028` — gate LIVE.
//
// Deciding line for `@Category Resource`: `src/io.rs:1573`
// `eval_kernel_pipe` — `libc::pipe2(2)` claims two fresh kernel-tracked
// fds; each is wrapped in an `OwnedFd` (`Drop` → `close(2)`). A syscall
// resource ACQUISITION, not merely a wat-level construction — see the
// module doc's strain-report entry (lands without argument once the body
// is read).
//
// Deciding line for `@Purity Effectful` / `@Determinism Deterministic`:
// a real syscall with an observable OS-level effect (two new open fds);
// no external actor's timing is awaited (unlike `accept`), so the outcome
// is deterministic given the process's fd-table capacity.
#[wat_intrinsic(":wat::kernel::pipe")]
pub(crate) fn eval_kernel_pipe(
    env: &Environment, // rune:lint(unused-env) — pipe2(2) needs no env; kept for the intrinsic signature
    sym: &SymbolTable,  // rune:lint(unused-sym) — see above
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    let _ = (env, sym);
    crate::io::eval_kernel_pipe(&[], list_span).map_err(Into::into)
}

/// `(:wat::kernel::spawn-thread prog init-fn post-spawn-fn)` →
/// `(:wat::kernel::Thread :- [R S])`. Spawns a thread-tier program peer running
/// `prog` (self-peer model: `fn([self <- (Peer' :- [S R])]) -> nil`); `init-fn`
/// (0-arg, returns `:wat::core::Record`) becomes the peer's `user-data`;
/// `post-spawn-fn` runs owner-side after spawn, for effects.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Resource
/// @arg     prog [(:wat::kernel::Peer :- [S R]) :-> :wat::core::nil] the self-peer program body, run once on the new thread
/// @arg     init_fn [:-> :wat::core::Record] 0-arg fn run at peer-start; its return becomes the peer's user-data
/// @arg     post_spawn_fn [:wat::spawn::ThreadLaunch :-> :wat::core::nil] runs owner-side after spawn
/// @yields  prog the new thread's own self-peer handle — a `(Peer' :- [S R])` — handed to prog when the thread starts running it
/// @yields  post_spawn_fn the just-spawned thread's ThreadLaunch record, handed to post_spawn_fn owner-side after spawn
/// @ret     (:wat::kernel::Thread :- [R S]) the new thread's peer handle
/// @example-norun (:wat::kernel::spawn-thread prog init-fn post-fn) #=> #wat.kernel/Thread{}
// No registered `TypeScheme` — `check.rs`'s `infer_spawn_thread_prime`
// (`:10310`) is the real authority: `args[0]` projects through
// `infer_thread_prog_type` (the shared self-peer projection helper) to
// `(Thread' :- [R S])`; `init-fn`/`post-spawn-fn` are inferred and (for
// post-spawn-fn) unified against `Fn(ThreadLaunch) -> nil`, but not
// further projected into the return.
//
// Deciding line for `@Category Resource`: `src/kernel/spawn.rs:453`
// `eval_kernel_spawn_thread_prime` spawns a real OS thread
// (`spawn_thread_peer`) and returns a `(Thread' :- [R S])` peer handle whose
// teardown `close'` administers — textbook acquisition. See the module
// doc's "spawn-thread / spawn-process — NOT inline blocks" note: this
// delegate was already `pub fn`, no lifting needed.
//
// Deciding line for `@Purity Effectful` / `@Determinism Deterministic`:
// spawning a thread is a real OS-level effect; the RETURN value (a valid
// `Thread'` peer) is produced immediately without waiting on any external
// actor's timing, so it is a deterministic function of the three fn
// arguments (unlike `accept`, which blocks on someone else's `connect`).
#[wat_intrinsic(":wat::kernel::spawn-thread")]
pub(crate) fn eval_kernel_spawn_thread_prime(
    prog: &WatAST,
    init_fn: &WatAST,
    post_spawn_fn: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::kernel::spawn::eval_kernel_spawn_thread_prime(
        &[prog.clone(), init_fn.clone(), post_spawn_fn.clone()],
        list_span,
        env,
        sym,
    )
}

/// `(:wat::kernel::spawn-process forms post-spawn-fn env-fn max-message-bytes identity)`
/// → `(:wat::kernel::Process :- [I O])`. Forks a process-tier program peer
/// running `forms` (the forms-server program); `post-spawn-fn` runs
/// owner-side after fork with the child pid; `env-fn` is a source string
/// the child evals for `user-data`; `max-message-bytes` is the
/// per-receiver frame budget; `identity` is an optional ps-visible label.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Resource
/// @arg     forms (:wat::core::Vector :- [:wat::WatAST]) the forms-server program to run in the child
/// @arg     post_spawn_fn [:wat::spawn::ProcessLaunch :-> :wat::core::nil] runs owner-side after fork, with the child pid
/// @yields  post_spawn_fn the just-forked child's ProcessLaunch record (including its pid), handed to post_spawn_fn owner-side after fork
/// @arg     env_fn :wat::core::String source string the child evals to produce user-data
/// @arg     max_message_bytes :wat::core::i64 per-receiver frame-size budget
/// @arg     identity (:wat::core::Option :- [:wat::core::Record]) optional ps-visible identity label
/// @ret     (:wat::kernel::Process :- [I O]) the new process's peer handle
/// @example-norun (:wat::kernel::spawn-process forms post-fn env-fn 524288 :wat::core::None) #=> #wat.kernel/Process{}
// No registered `TypeScheme` — `check.rs`'s `infer_spawn_process_prime`
// (`:10378`) is the real authority: `forms` projects through
// `infer_process_prog_type` to `(Process' :- [I O])`; the other four args are
// inferred (and `post-spawn-fn` unified against `Fn(ProcessLaunch) ->
// nil`) but not further projected into the return.
//
// Deciding line for `@Category Resource`: `src/kernel/spawn.rs:539`
// `eval_kernel_spawn_process_prime` forks a real OS process
// (`spawn_process_peer`) and returns a `(Process' :- [I O])` peer handle whose
// teardown `close'` administers — same acquisition shape as
// `spawn-thread`. Already `pub fn`; no lifting needed (see module doc).
//
// Deciding line for `@Purity Effectful` / `@Determinism Deterministic`:
// forking is a real OS-level effect; the returned handle is produced
// immediately without waiting on the child's own behavior.
#[wat_intrinsic(":wat::kernel::spawn-process")]
// `#[expect]`, not `#[allow]`, per this module's own convention (`mod.rs:25`) — it goes
// red the moment it stops being needed. EARNED, not unfinished: the parameter count is
// imposed by the `#[wat_intrinsic]` ABI (one `&WatAST` per wat arg, plus the
// `env`/`sym`/`list_span` context tail), NOT by this fn braiding concerns — its whole body
// is one delegate call. `spawn-process` is the first 5-arg verb the registry has carved, so
// 5 + 3 = 8 is the first time the shape has exceeded clippy's 7. The alternatives are worse:
// changing the macro ABI, or declining to register verbs above arity 4.
#[expect(clippy::too_many_arguments)]
pub(crate) fn eval_kernel_spawn_process_prime(
    forms: &WatAST,
    post_spawn_fn: &WatAST,
    env_fn: &WatAST,
    max_message_bytes: &WatAST,
    identity: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::kernel::spawn::eval_kernel_spawn_process_prime(
        &[
            forms.clone(),
            post_spawn_fn.clone(),
            env_fn.clone(),
            max_message_bytes.clone(),
            identity.clone(),
        ],
        list_span,
        env,
        sym,
    )
}

/// `(:wat::kernel::after peer-kind duration msg)` → `(:wat::kernel::Thread :- [nil O])`.
/// One-shot timer peer: fires `msg` once after `duration`, then EOFs.
/// Drops into `poll'`/`select'` by construction — a real `Peer'`, not a
/// tier-specific `Timer'`.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Resource
/// @arg     peer_kind :wat::program::PeerKind `:thread` or `:process` — selects the timer's tier
/// @arg     duration :wat::time::NonZeroDuration positive delay before the timer fires
/// @arg     msg :O the payload delivered when the timer fires; becomes the peer's output type
/// @ret     (:wat::kernel::Thread :- [:wat::core::nil O]) a one-shot timer peer (`I` = nil — the timer takes no input)
/// @example-norun (:wat::kernel::after (:thread) (:wat::time::Milliseconds 50) "tick") #=> #wat.kernel/Thread{}
// No registered `TypeScheme` — `check.rs`'s `infer_kernel_after`
// (`:10595`) is the real authority: `peer-kind` must conform to
// `PeerKind`, `duration` to `NonZeroDuration`; `O` is `msg`'s inferred type,
// projected into the `(Peer' :- [nil O])` return — projective, no fixed-arity
// scheme.
//
// Deciding line for `@Category Resource`: `runtime.rs:32904`–`33057`
// `eval_kernel_after` builds a REAL handle — a crossbeam timer `Receiver`
// (thread tier, `comms::thread::timer`) or a `timerfd`-backed
// `process::Receiver` (process tier, an actual OS fd,
// `comms::process::timer`) — and wraps it in a `Peer'`
// (`ThreadOwnedCell`/`RustOpaque`, `PEER_TYPE_PATH`). The caller holds
// THAT handle, administered by the same `close'` machinery as any peer.
// See the module doc's strain-report entry: the brief's "nobody holds
// time" framing does not survive this body-read.
//
// Deciding line for `@Purity Effectful` / `@Determinism Deterministic`:
// allocates a real timer resource (futex or timerfd); the returned
// handle is produced immediately, not contingent on an external actor's
// timing (the FIRING is later and unpredictable, but that is a property
// of USING the returned peer, not of `after`'s own return).
#[wat_intrinsic(":wat::kernel::after")]
pub(crate) fn eval_kernel_after(
    peer_kind: &WatAST,
    duration: &WatAST,
    msg: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_kernel_after(
        &[peer_kind.clone(), duration.clone(), msg.clone()],
        list_span,
        env,
        sym,
    )
}

/// `(:wat::kernel::close peer)` → `:wat::kernel::CloseOutcome`. Consumes
/// the peer (`(Thread' :- [I O])` or `(Process' :- [I O])`) and returns a matchable
/// outcome for every HANDLEABLE teardown result (`Closed`/`Failed`/
/// `Signaled`); only double-close/use-after-close and type mismatches
/// stay raises.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Total         Unreviewed
/// @Category      Resource
/// @arg     peer (:wat::kernel::Peer :- [I O]) the peer to close (Thread' or Process')
/// @ret     :wat::kernel::CloseOutcome Closed[exit] / Failed[cause] / Signaled[signal] — must-use
/// @example-norun (:wat::kernel::close my-thread) #=> #wat.kernel/CloseOutcome.Closed{exit: #wat.core/None{}}
// No registered `TypeScheme` — `check.rs`'s `infer_close_prime`
// (`:10964`) is the real authority: ∀-parametric over `peer<∀I,∀O>` —
// rank-1 HM cannot enumerate every `(I,O)`, so this is a bespoke arm, not
// a fixed scheme. Return is the must-use `CloseOutcome` (`MUST_USE_TYPES`).
//
// Deciding line for `@Category Resource`: `runtime.rs:31708`
// `eval_peer_close_prime` — `cell.with_mut(… |opt_peer| opt_peer.take()
// …)` CONSUMES the peer (leaves `None` for subsequent calls), drains and
// joins it, and returns a classified outcome. Textbook RELEASE.
//
// Deciding line for `@Purity Effectful` / `@Determinism Nondeterministic`:
// joining blocks on the far side and the outcome (`Closed` vs. `Failed`)
// depends on whether the joined thread/process crashed — a fact not
// determined by `close`'s own arguments, the same "depends on the other
// side" reasoning `kernel_message.rs` used for `recv`/`select`/`poll`.
#[wat_intrinsic(":wat::kernel::close")]
pub(crate) fn eval_peer_close_prime(
    peer: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_peer_close_prime(std::slice::from_ref(peer), list_span, env, sym)
}

/// `(:wat::kernel::signal peer sig)` → `:wat::kernel::SignalOutcome`.
/// Sends a POSIX signal to a `(Process' :- [I O])` peer's child via
/// `Pidfd::send_signal` (never `kill(pid, sig)`). Does NOT consume the
/// peer — signal any number of times before `close'`.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Total         Unreviewed
/// @Category      Resource
/// @arg     peer (:wat::kernel::Process :- [I O]) the process peer to signal (process-tier only)
/// @arg     sig :wat::kernel::Signal the POSIX signal to deliver (User1/User2/Hangup/Interrupt/Terminate/Kill)
/// @ret     :wat::kernel::SignalOutcome the delivery outcome — must-use
/// @example-norun (:wat::kernel::signal my-process :wat::kernel::Signal::Interrupt) #=> #wat.kernel/SignalOutcome.Sent{}
// No registered `TypeScheme` — `check.rs`'s `infer_signal` (`:11039`) is
// the real authority: `(Process :- [I O])`-only (unlike `close'`, not shared
// with `Thread'`), return is the must-use `SignalOutcome`.
//
// Deciding line for `@Category Resource`: `runtime.rs:31909`
// `eval_signal` reads the peer via `cell.with_ref` (NOT `with_mut` +
// `take` — never consumes) and calls
// `bundle.peer.pidfd.send_signal(sig_posix)`. Neither acquires nor
// releases the handle — pure third-disjunct ADMINISTRATION of a handle
// the caller already holds, delivered through the pidfd. See the module
// doc's "remaining ten" note — lands without argument.
//
// Deciding line for `@Purity Effectful` / `@Determinism Nondeterministic`:
// a real syscall against a live external process; whether it succeeds
// (and what the signaled process does in response) depends on that
// process's own, externally-determined state — the same "depends on the
// other side" shape as `close`.
#[wat_intrinsic(":wat::kernel::signal")]
pub(crate) fn eval_signal(
    peer: &WatAST,
    sig: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_signal(&[peer.clone(), sig.clone()], list_span, env, sym)
}

/// `(:wat::kernel::listener locus …)` → `:(wat::kernel::Listener<S,R>, wat::kernel::Address<S,R>)`
/// (thread tier, 3 args: `locus :S :R`) or `(:wat::kernel::Listener :- [S R])`
/// (process tier, 2–4 args: `locus addr [:max-frame-bytes]`). Mints a
/// fresh connection listener — thread tier: a crossbeam rendezvous;
/// process tier: a kernel-autobound abstract-namespace UDS.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Resource
/// @arg     xs… :wat::core::Value locus (+ tier-dependent trailing args — see `infer_listener_prime`)
/// @ret     (:wat::kernel::Listener :- [S R]) the fresh listener (thread tier also returns its paired Address in a tuple)
/// @example-norun (:wat::kernel::listener (:thread) :S :R) #=> #wat.core/Tuple[#wat.kernel/Listener{} #wat.kernel/Address{}]
// No registered `TypeScheme` — `check.rs`'s `infer_listener_prime`
// (`:9622`) is the real authority: dispatches on the evaluated locus
// (`ThreadOpts` → 3-arg form; `ProcessOpts` → 2–4-arg autobind form)
// BEFORE arity is even checked — a shape no fixed scheme expresses.
// Variadic wrapper (`xs: &[WatAST]`) because the legal arg COUNT differs
// by tier, decided at runtime.
//
// Deciding line for `@Category Resource`: `runtime.rs:26051`
// `eval_listener_prime` mints a fresh listener (crossbeam rendezvous or
// kernel-autobound UDS) and returns it — textbook ACQUISITION of a handle
// whose lifetime `accept`/`close'` (thread tier: scope) administer
// thereafter.
//
// Deciding line for `@Purity Effectful` / `@Determinism Deterministic`:
// real synchronization/socket state is minted; the listener is returned
// immediately without waiting on any external actor (unlike `accept`,
// which blocks on someone else's `connect`).
#[wat_intrinsic(":wat::kernel::listener")]
pub(crate) fn eval_listener_prime(
    xs: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_listener_prime(xs, list_span, env, sym)
}

/// `(:wat::kernel::connect addr)` → `(:wat::kernel::Peer :- [S R])`. Dials
/// `addr` (a unified `(Address' :- [S R])`, both tiers) and returns the client
/// end as a `(Peer' :- [S R])`.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Resource
/// @arg     addr (:wat::kernel::Address :- [S R]) the address to dial (from `listener`'s thread-tier tuple, or discovered process-tier)
/// @ret     (:wat::kernel::Peer :- [S R]) the client end of the new connection
/// @example-norun (:wat::kernel::connect addr) #=> #wat.kernel/Peer{}
// No registered `TypeScheme` — `check.rs`'s `infer_connect_prime`
// (`:9872`) is the real authority: extracts `S,R` from the `(Address' :- [S R])`
// argument's parametric type — projective on the input, not a fixed
// scheme.
//
// Deciding line for `@Category Resource`: `runtime.rs:26237`
// `eval_connect_prime` downcasts `addr` and calls `inner.connect(sym,
// span)`, wrapping the result as a `PEER_TYPE_PATH` opaque — mints and
// returns a fresh peer handle. ACQUISITION.
//
// Deciding line for `@Purity Effectful` / `@Determinism Deterministic`:
// a real connect (crossbeam send over rendezvous, or a `connect(2)`
// syscall); returns immediately without blocking on the remote side
// calling `accept` (the OS backlog absorbs the connect), so the outcome
// is deterministic given a live address.
#[wat_intrinsic(":wat::kernel::connect")]
pub(crate) fn eval_connect_prime(
    addr: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_connect_prime(std::slice::from_ref(addr), list_span, env, sym)
}

/// `(:wat::kernel::accept listener)` → `(:wat::kernel::Peer :- [R S])`. Blocks
/// until a connection arrives on `listener`, then returns the server end
/// as a `(Peer' :- [R S])` (the flipped pair: server recvs S, sends R).
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Total         Unreviewed
/// @Category      Resource
/// @arg     listener (:wat::kernel::Listener :- [S R]) the listener to accept a connection from
/// @ret     (:wat::kernel::Peer :- [R S]) the server end of the accepted connection
/// @example-norun (:wat::kernel::accept my-listener) #=> #wat.kernel/Peer{}
// No registered `TypeScheme` — `check.rs`'s `infer_accept_prime`
// (`:9946`) is the real authority: returns `(Peer' :- [R S])` — the FLIPPED
// pair relative to the listener's `[S R]` — a projection a fixed scheme
// could express in shape but not in the flip's provenance-sensitive
// wiring shared with `listener`/`connect`.
//
// Deciding line for `@Category Resource`: `runtime.rs:26421`
// `eval_accept_prime` — thread tier blocks on the rendezvous Receiver;
// process tier calls `.accept()` on the downcast `UnixListener` (blocks
// until a connection). Mints and returns a fresh peer — ACQUISITION,
// same shape as `connect`.
//
// Deciding line for `@Purity Effectful` / `@Determinism Nondeterministic`:
// blocks on an external, unpredictable event (someone else calling
// `connect`) — WHO connects and WHEN is not fixed by `accept`'s own
// argument, the same reasoning `kernel_message.rs` gave `poll`'s listener
// arm.
#[wat_intrinsic(":wat::kernel::accept")]
pub(crate) fn eval_accept_prime(
    listener: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_accept_prime(std::slice::from_ref(listener), list_span, env, sym)
}

/// `(:wat::kernel::allow listener pid)` → `:()`. Inserts `pid` into the
/// listener's allow-set (process-tier only — a thread listener has no
/// allow-set; its crossbeam handle IS the grant).
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Resource
/// @arg     listener (:wat::kernel::Listener :- [S R]) the listener whose allow-set to administer
/// @arg     pid :wat::core::i64 the pid to allow
/// @ret     :wat::core::nil always `:()`
/// @example-norun (:wat::kernel::allow my-listener 4242) #=> #wat.core/nil{}
// No registered `TypeScheme` — `check.rs`'s `infer_allow_prime`
// (`:10011`) is the real authority: `[(Listener' :- [S R]) i64 :-> nil]`; tier
// (thread vs. process) is not checked here — both tiers share the type
// `(Listener' :- [S R])` at check time, and tier-rejection is runtime-only.
//
// Deciding line for `@Category Resource`: `runtime.rs:26471`
// `eval_allow_prime` downcasts `listener` to `SocketListener` and calls
// `sl.allow(pid, …)` — mutates the CALLER-HELD LISTENER HANDLE's
// allow-set. See the module doc's strain-report entry: the target is the
// handle, not a free-standing capability value — lands without a
// capability-as-handle argument once the body is read.
//
// Deciding line for `@Purity Effectful` / `@Determinism Deterministic`:
// a real mutation of shared listener state; insert is idempotent and the
// result (`:()`) is the same regardless of prior allow-set contents —
// same "always the same store, same reasoning kernel_ambient.rs gave its
// three resetter writers.
#[wat_intrinsic(":wat::kernel::allow")]
pub(crate) fn eval_allow_prime(
    listener: &WatAST,
    pid: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_allow_prime(&[listener.clone(), pid.clone()], list_span, env, sym)
}

/// `(:wat::kernel::deny listener pid)` → `:()`. Removes `pid` from the
/// listener's allow-set (future accepts by that pid bounce). Identical
/// shape to `allow`.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Resource
/// @arg     listener (:wat::kernel::Listener :- [S R]) the listener whose allow-set to administer
/// @arg     pid :wat::core::i64 the pid to deny
/// @ret     :wat::core::nil always `:()`
/// @example-norun (:wat::kernel::deny my-listener 4242) #=> #wat.core/nil{}
// No registered `TypeScheme` — `check.rs`'s `infer_deny_prime`
// (`:10079`) is the real authority: "Identical shape to
// infer_allow_prime" (the doc comment's own words).
//
// Deciding line for `@Category Resource` / `@Purity Effectful` /
// `@Determinism Deterministic`: identical reasoning to `allow` —
// `runtime.rs:26550` `eval_deny_prime` calls `sl.deny(pid, …)` on the
// same caller-held listener handle.
#[wat_intrinsic(":wat::kernel::deny")]
pub(crate) fn eval_deny_prime(
    listener: &WatAST,
    pid: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_deny_prime(&[listener.clone(), pid.clone()], list_span, env, sym)
}
