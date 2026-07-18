;; tests/comms/probe_select_flood_no_deadlock.wat — co-located fixture, slurped via
;; startup_beside(file!()). No placeholder main — startup_beside loads defns only.
;;
;; Stone — select' over a process child that floods stdout with > 512 KiB of un-terminated
;; data must return ServiceEvent::Lost without deadlocking. See probe_select_flood_no_deadlock.rs
;; for the bug/fix narrative (eval_peer_select_prime's process arm, src/runtime.rs). This fixture
;; is the wat program under test: it spawns the flooding child and drives select' over it,
;; exactly as the (deleted) dynamically-built Rust AST used to.
;;
;; Flood strategy: the child builds a 1 MiB string (2^20 = 1,048,576 bytes of 'x') via
;; double-string and prints it. The println wire format is a quoted EDN string
;; ("xxxx...xxx"\n) — total frame ~1,048,578 bytes, well above the 512 KiB cap. The child
;; stays alive (blocked in the kernel's write(2), pipe buffer full) while the parent
;; accumulates bytes past the cap.

(:wat::core::defn :user::compute [] -> :wat::spawn::ServiceEvent<wat::core::nil,wat::core::nil,wat::core::nil>
  (:wat::core::let
    [child (:wat::kernel::spawn-program' (:wat::spawn::process)
             (:wat::core::forms
               (:wat::core::defn :user::double-string
                   [s <- :wat::core::String n <- :wat::core::i64]
                   -> :wat::core::String
                 (:wat::core::if (:wat::core::= n 0) -> :wat::core::String
                   s
                   (:user::double-string (:wat::core::String/concat s s) (:wat::core::- n 1))))
               (:wat::core::defn :user::main [] -> :wat::core::nil
                 (:wat::kernel::println
                   (:user::double-string "x" 20)))))]
    (:wat::kernel::select' (:wat::core::Vector :wat::kernel::Process'<wat::core::nil,wat::core::nil> child))))
