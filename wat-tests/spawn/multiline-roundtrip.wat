;; wat-tests/spawn/multiline-roundtrip.wat — arc 259.S3.6 dogfood, wat surface.
;;
;; THE GOLD-STANDARD round-trip: a MULTI-LINE pretty-printed value crosses a
;; process peer as ONE message.
;;
;; The client (this test body) spawns a forms-server child; the server's
;; :user::main `pprintln`s a map to its stdout (fd 1) — `pprintln` breaks a
;; multi-key map across PHYSICAL lines. The client `recv'`s it off the named-fd
;; `(Process' :- [I O])` peer.
;;
;; Before 259.S3.6 the comms io_uring framer split on the FIRST '\n', so a
;; multi-line value was mis-framed — `recv'` yielded just "{". After: the one
;; frame-finder (`next_complete_frame`) value-frames the whole value. This test
;; asserts the multi-line map round-trips intact — the proof the fix earns.
;;
;; Model: wat-tests/service-locus-parity.wat (deftest' + (:wat::spawn::process))
;; + wat/test.wat run-hermetic' (spawn-program' (process) + forms + recv').
;; PRIMED ONLY — the non-prime spawn-process + 4-field Process are doomed.

(:wat::test::deftest :wat-tests::process::multiline-pprintln-roundtrip
  
  (:wat::test::assert-eq
    (:wat::core::let
      [p (:wat::test::spawn-peer (:wat::spawn::process)
           (:wat::core::forms
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:wat::kernel::pprintln {:alpha 1 :beta 2 :gamma 3 :delta 4 :epsilon 5}))))]
      (:wat::core::match (:wat::kernel::recv p)
        ((:wat::kernel::RecvOutcome::Message m) m)
        ((:wat::kernel::RecvOutcome::Lost cause)
          (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
        (:wat::kernel::RecvOutcome::Stopped
          (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
        (:wat::kernel::RecvOutcome::Closed
          (:wat::kernel::assertion-failed! "recv': p closed unexpectedly" :wat::core::None :wat::core::None))))
    {:alpha 1 :beta 2 :gamma 3 :delta 4 :epsilon 5}))
