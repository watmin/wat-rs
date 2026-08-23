;; tests/process/signal_kill_produces_close_outcome_signaled.wat
;;
;; EVIDENCE fixture for EXPECTATIONS-process-signal-p2-mint.md row 5 ("Kill reaches the
;; owner-side observable") — the sharpest gate in the strike, because SIGKILL is uncatchable:
;; there is no handler to fake this with.
;;
;; `:user::compute` spawns a :process child blocked in `readln`, sends
;; `:wat::kernel::Signal::Kill` via the P2 verb, FACES the returned SignalOutcome (both arms —
;; the must-use gate would refuse anything less), and returns the SAME peer to its caller.
;;
;; ⚠ WHY THIS DOES NOT ALSO CALL `close'` FROM WAT, AND WHY THAT IS HONEST, NOT AN OVERSIGHT:
;; `:wat::kernel::close` is restricted to `:wat::kernel::` callers (arc 259 S2d — "the user never
;; holds the rope"; teardown is RAII Drop for ordinary code), and today has ZERO production wat
;; call sites. The obvious test-only workaround — declare a helper function UNDER the
;; `:wat::kernel::` namespace so a test file can legitimately satisfy that restriction — does NOT
;; work: it hits a SEPARATE, earlier-firing guard (`RuntimeErrorKind::ReservedPrefix` /
;; `CheckErrorKind::ReservedPrefix`, `check.rs:7942` / `runtime.rs:918`) that flatly refuses ANY
;; non-stdlib source defining anything under the `:wat::`/`:rust::` prefixes — verified empirically
;; (see the co-located .rs's header). Stdlib source bypasses this via `RegistrationPrivilege::
;; Stdlib` (`stdlib.rs`), which is NOT available to `tests/`-loaded fixtures. NET RESULT: there is
;; currently NO way for ANY wat-level test fixture — not this one, not a wat-scripts deftest — to
;; call `close'` and observe its `CloseOutcome` directly. The co-located .rs works around this by
;; reading the SAME underlying Rust mechanism `close'` itself uses (`Process::wait()`), one layer
;; below the restricted wat verb, on the EXACT peer `:wat::kernel::signal` (the code under test)
;; just delivered Kill through.
;;
;; P4 LANDED. The three self-signalling harness tests are gone: two re-exec and signal a real
;; child (`probe_arc170_writer_joins_lockstep.rs`, `probe_arc278_send_poll_arm.rs`), and the two
;; shutdown-cascade tests were DELETED outright in favour of
;; `wat-tests/process/signal-terminate-kills-the-child-and-the-read-sees-it.wat` — a parent that
;; spawns a child, reads it, signals it, and reads it gone. `grep -rn 'libc::raise' tests/` now
;; returns only this sentence. If a future
;; strike gives wat source a sanctioned path to `close'` (a stdlib-privileged test helper, or a
;; non-restricted "peek" verb), THIS fixture should be replaced with a pure-wat one that matches
;; `CloseOutcome::Signaled` directly instead of reading `Process::wait()` from Rust.
(:wat::core::defn :user::compute [] -> (:wat::kernel::Process :- [:wat::core::i64 :wat::core::String])
  (:wat::core::let
    [proc (:wat::test::spawn-peer
            (:wat::spawn::process)
            (:wat::core::forms
              (:wat::core::defn :user::main [] -> :wat::core::nil
                (:wat::core::let
                  [n (:wat::core::match (:wat::kernel::readln)
                       ((:wat::kernel::ReadlnOutcome::Datum d) d)
                       (:wat::kernel::ReadlnOutcome::Eof nil)
                       (:wat::kernel::ReadlnOutcome::Stopped nil))]
                  nil))))]
    (:wat::core::do
      (:wat::core::match (:wat::kernel::signal proc :wat::kernel::Signal::Kill)
        (:wat::kernel::SignalOutcome::Delivered nil)
        ((:wat::kernel::SignalOutcome::Failed _c) nil))
      proc)))
