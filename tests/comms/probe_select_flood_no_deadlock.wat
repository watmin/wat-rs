;; tests/comms/probe_select_flood_no_deadlock.wat — co-located fixture, slurped via
;; startup_beside(file!()). No placeholder main — startup_beside loads defns only.
;;
;; Stone — select' over a process child that floods stdout with > 512 KiB of un-terminated
;; data must return ServiceEvent::Lost without deadlocking. See probe_select_flood_no_deadlock.rs
;; for the bug/fix narrative (eval_peer_select_prime's process arm, src/runtime.rs). This fixture
;; is the wat program under test: it spawns the flooding child and drives select' over it,
;; exactly as the (deleted) dynamically-built Rust AST used to.
;;
;; Flood strategy (arc 170 Strike 3): the child builds a 1 MiB string (2^20 = 1,048,576 bytes of
;; 'x') via double-string and RAW-writes it to fd 1 via `:wat::kernel::write-fd-raw` — NOT `println`.
;; After the verb flip, `println` is bounded by StdOut's `:max-request-bytes` op budget, so a
;; conforming peer's oversized println now fails with RequestTooLarge BEFORE it can flood the wire
;; (correct — bounded stdio). This probe needs a NON-CONFORMING peer, so it emits raw un-terminated
;; bytes straight to fd 1 (the peer wire): 1,048,576 bytes with no frame terminator, well above the
;; 512 KiB cap. The child stays alive (blocked in the kernel's write(2), pipe buffer full) while the
;; parent's frame reader accumulates past the cap → FrameTooLarge → Lost.

(:wat::core::defn :user::compute [] -> (:wat::spawn::ServiceEvent :- [:wat::core::nil :wat::core::nil :wat::core::nil])
  (:wat::core::let
    [child (:wat::test::spawn-peer (:wat::spawn::process)
             (:wat::core::forms
               (:wat::core::defn :user::main [] -> :wat::core::nil
                 ;; Simulate a NON-CONFORMING peer that floods fd 1 (the peer wire) with un-terminated
                 ;; bytes. The `:user::` child controls NOTHING: every raw-write primitive is
                 ;; kernel/test-gated (users cannot flood ad-hoc), and the ONLY user-reachable flood is
                 ;; the ZERO-ARG, hardcoded `:wat::test::flood-own-stdout` (fixed ~1 MiB to its OWN
                 ;; stdout) — in the child's closure (stdlib loads in every child). It blocks in
                 ;; write(2) once the pipe fills (parent stops draining at FrameTooLarge), keeping the
                 ;; child alive.
                 (:wat::core::let
                   [_n (:wat::test::flood-own-stdout)]
                   nil))))]
    (:wat::kernel::select (:wat::core::Vector (:wat::kernel::Process :- [:wat::core::nil :wat::core::nil]) child))))
