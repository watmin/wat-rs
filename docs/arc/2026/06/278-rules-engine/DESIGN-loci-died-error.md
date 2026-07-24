# DESIGN — `LociDiedError` + crash-reason records (the loci-agnostic death report)

> **Status: BUILT + SHIPPED** (`d60b1887`, 2026-07-24; floor 4216/0 by own re-run, pushed).
> Prerequisite BANKED: crash-reason `Frame` honesty (`6e98733b`). Followed immediately by the
> **string-wrap annihilation** (`251b43b3`) — `Failure` now carries the raised `:wat::core::Error`
> structurally (see the FAR-SIDE UPDATE 24f breadcrumb in REALIZATIONS.md). The whole death/crash
> surface is now structured EDN end-to-end: error → `:wat::core::Error`, frames → `Vector<Frame>`,
> location → `Location`, chain → `Vector<LociDiedError>`. Zero string-wrapping remains.

## How we got here (the stack the run-hermetic de-prime surfaced)

The `run-thread`/`run-hermetic` de-prime (part of the IPC de-prime) hit a real substrate flaw
via the failure-payload test bucket (`ALIVS ARGVIT` — the consumer surfaced it): the primed
`Lost[cause]` hands the caller a crash reason that is **not an EDN-round-trippable record**.
Grounding pulled up three roots:

1. **The anon-fn identity was a stringy `<fn@file:line:col>` blob** (invalid EDN keyword when it
   lands in a crash frame's `:callee`). → **FIXED + BANKED (`6e98733b`, arc-109 note realized):**
   `:wat::kernel::Frame` is now non-`Option` `{file: String, line: i64, symbol: String}`; the anon
   symbol is the FQDN of the Fn **type** `:wat::core::Fn`; macro-call-site's symbol is the **macro
   name** (threaded through `MacroCallSiteGuard`); `call-site`'s empty-stack all-`None` **mask** is
   replaced with an honest `MalformedForm` error.
2. **The crash chain is heterogeneous** — `#wat.kernel/ProcessPanics` is a tagged *vector* whose
   elements are `ThreadDiedError | ProcessDiedError` (two near-twin enums). → this stone.
3. **`AssertionFailure` is a hand-built Map with the wrong shapes** — `:frames` is an ad-hoc
   `{:callee, :at}` map (not the `Frame` record), `:location` is a `Span` (no registered TypeDef).
   → this stone.

## The builder's rulings (verbatim intent)

- *"the primed tooling was purpose built to annihilate our initial IPC functions — all the non-prime
  tooling must be burned to the ground for the primed to replace them."*
- *"we do not fear refactors — annihilation is our greatest joy."* / *"do not defend the dead."*
- *"one Enum who bears all possible failures that every peer must explicitly handle … we never know
  what some service or bracket worker may be hosted on, so we measure that every loci is handled
  correctly."*

So: **annihilate `ThreadDiedError`, `ProcessDiedError`, and `ProcessPanics`.** Replace them with
**one loci-agnostic `LociDiedError`** enum every peer exhaustively handles — because a service /
bracket-worker never knows its own locus (thread · process · uds · localhost tcp · remote mTLS · …),
so the exhaustive match is the shield (the *explicit-exception-paths, verbosity-is-the-shield* doctrine).

## The four-questions decisions (all flat YES → decided)

**Q1 — `recv'`'s `Lost` cause becomes `:wat::kernel::LociDiedError`** (replacing today's `Failure`-wrapped cause):
- Obvious? **YES** — a `Lost` *is* a peer death; `LociDiedError` names exactly how it died; `Failure` mumbles.
- Simple? **YES** — one concept: "a peer death is a `LociDiedError`."
- Honest? **YES** — no-hidden-failures: the death cause is an explicit matchable variant, not flattened
  into a string. Nothing lost — the raised `Fault` rides in `Panic.failure`.
- Good UX? **YES** — the exhaustive match is compiler-forced; every peer handles every death regardless
  of the locus it turns out to be on.

**Q2 — annihilate `#wat.kernel/ProcessPanics`; the chain is `Vector<LociDiedError>`** (no wrapper):
- Obvious? **YES** — "ProcessPanics" is a Process-and-panic-specific name over loci-agnostic,
  not-all-panics data (`StartupError`/`Disconnected` aren't panics). A bare `Vector<LociDiedError>` says what it is.
- Simple? **YES** — no wrapper type/accessor; the chain *is* the vector.
- Honest? **YES** — it's a dead non-prime (a hand-serialized, locus-specific stderr-scrape tag). *Don't defend the dead.*
- Good UX? **YES** — callers map/match a homogeneous `Vector<LociDiedError>` directly, no unwrap.

## The user-forms (the UX, ratified)

### The one enum every peer handles
Variant set = the union of the grounded `ThreadDiedError` + `ProcessDiedError` variants, generalized
loci-agnostic (the variants name *how* a peer died; the locus rides as data). Registered as a Rust
builtin `EnumDef` in `src/types.rs` (mirroring how `Thread/ProcessDiedError` are registered today):

```clojure
;; :wat::kernel::LociDiedError — the ONE death report, loci-agnostic
;; (thread · process · uds · localhost tcp · remote mTLS · whatever comes).
(:wat::core::defenum :wat::kernel::LociDiedError
  (Panic            [message <- :wat::core::String  failure <- (:wat::core::Option :wat::kernel::Failure)])  ;; peer raised/panicked
  (RuntimeError     [message <- :wat::core::String])   ;; type/arity/etc. surfaced at run
  Disconnected                                          ;; the wire dropped (was ChannelDisconnected)
  Shutdown                                              ;; shutdown signal mid-recv, any locus
  (StartupError     [message <- :wat::core::String])   ;; locus didn't come up — fork/exec fail, or remote ECONNREFUSED
  (EntryFormFailure [message <- :wat::core::String])   ;; peer program's entry form malformed
  (MainSignature    [message <- :wat::core::String])   ;; peer's :user::main bad signature
  (BadReturn        [message <- :wat::core::String]))  ;; peer returned a value that won't cross the wire
```
(`ThreadDiedError` variants: `Panic{message,failure}` · `RuntimeError{message}` · `ChannelDisconnected` ·
`Shutdown`. `ProcessDiedError` adds `StartupError` · `EntryFormFailure` · `MainSignature` · `BadReturn`.
Reconcile `ChannelDisconnected` → `Disconnected`. Consider whether `LociDiedError` needs a recursive
`upstream-chain` field for the death cascade, or the chain stays at the container level — GROUND at draw.)

### The exhaustive match a peer/service/bracket-worker writes (THE UX)
```clojure
(:wat::core::match (:wat::kernel::recv' peer)
  ((:wat::kernel::RecvOutcome::Message v)   (:my::on-message v))
  ((:wat::kernel::RecvOutcome::Lost cause)
    (:wat::core::match cause
      ((:wat::kernel::LociDiedError::Panic message failure)    (:my::on-panic message failure))
      ((:wat::kernel::LociDiedError::RuntimeError message)     (:my::on-runtime message))
      (:wat::kernel::LociDiedError::Disconnected               (:my::on-disconnected))
      (:wat::kernel::LociDiedError::Shutdown                   (:my::on-shutdown))
      ((:wat::kernel::LociDiedError::StartupError message)     (:my::on-startup message))
      ((:wat::kernel::LociDiedError::EntryFormFailure message) (:my::on-entry-form message))
      ((:wat::kernel::LociDiedError::MainSignature message)    (:my::on-main-sig message))
      ((:wat::kernel::LociDiedError::BadReturn message)        (:my::on-bad-return message))))
  (:wat::kernel::RecvOutcome::Closed        (:my::on-closed)))
```

### The corrected `AssertionFailure` (records-are-EDN)
```clojure
(:wat::core::defrecord :wat::kernel::AssertionFailure
  [thread         <- :wat::core::String
   message        <- :wat::core::String
   location       <- (:wat::core::Option :wat::kernel::Location)
   actual         <- (:wat::core::Option :wat::core::String)
   expected       <- (:wat::core::Option :wat::core::String)
   frames         <- (:wat::core::Vector :wat::kernel::Frame)          ;; was the ad-hoc {:callee,:at} map
   upstream-chain <- (:wat::core::Vector :wat::kernel::LociDiedError)]) ;; was heterogeneous Thread|Process
```
(`:wat::kernel::Location {file: String, line: i64, col: i64}` is ALREADY registered, `types.rs:1057`.
`:wat::kernel::Failure` and `:wat::kernel::Frame` are ALREADY registered records.)

## The strike (blast radius — a real stone, ablaze-driven)

1. Register `:wat::kernel::LociDiedError` (the enum above) + register `AssertionFailure` (record) in
   `src/types.rs`; **delete `ThreadDiedError` + `ProcessDiedError`** registrations.
2. Change `RecvOutcome::Lost`'s cause type → `LociDiedError` (the recv' wall). Change the producers that
   build a `Thread/ProcessDiedError` (join-result, waitpid/exit paths, the crash channel) → build a
   `LociDiedError`. Annihilate `ProcessPanics` (the `format!` tag in `process/verbs.rs`; the special
   case at `runtime.rs:23606`; the bespoke `extract-panics` reader) → the death crosses as a
   `LociDiedError` / `Vector<LociDiedError>` self-describing EDN value, read by generic `edn::read`.
3. Route the `AssertionFailure` writer (`panic_hook.rs`'s hand-built Map) through the derived `ToEdn`
   of the registered record (frames → `Frame`, location → `Location`, chain → `Vector<LociDiedError>`).
4. **The re-type is the worklist (R52 `QVOD LEX ACCENDIT`):** deleting `Thread/ProcessDiedError` +
   changing `Lost`'s cause makes the checker + rustc scream every producer/consumer — the `*/to-failure`
   accessors, `sandbox.wat`/`hermetic.wat`, the chain builders (`conj_died_chain`, `runtime.rs:23265`),
   the 2 exemplar reshapes (`6a9f8f59` — the capture one reads the Lost cause as `Failure`). Fix each.
5. Weigh `cargo nextest run --release`, floor 4215/0 (own re-run). Verify a crash reason now
   round-trips through `edn::read` (the failure-payload probe `raise_round_trip`).

## The dependency chain this unblocks

`Frame` honesty (BANKED `6e98733b`) → **`LociDiedError` + AssertionFailure records (this stone)** →
the failure-payload run-hermetic bucket round-trips → fleet the 3 buckets (capture/stderr proven at
`6a9f8f59`; failure-payload) to their ~17 sibling probes → then the four-step run-thread/run-hermetic
de-prime completion (prime the ~54 `run-thread`/`run-hermetic` direct callers → delete the non-prime
runners + macros → shrink `RunResult` to failure-only → reclaim the plain names).
