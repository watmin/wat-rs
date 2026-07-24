;; tests/comms/wat_arc113_raise_round_trip.wat — co-located fixture for the raise round-trip probe,
;; slurped via startup_beside(file!()). No placeholder main — startup_beside loads defns only.
;;
;; Arc 296 re-gate + arc 278 the string-wrap annihilation: raise! requires
;; :wat::core::Error, and the round-trip is now STRUCTURAL WITH NO STRING WRAP —
;; the error survives the panic boundary as a RECORD carried in Failure's `error`:
;;   (raise! (Fault/of "arc113-raise-data"))
;;     → (:wat::kernel::Failure/error f) yields the :wat::core::Fault RECORD DIRECTLY
;;       (no edn::write into a String, no edn::read back out)
;;     → (Fault/message …) reads the message field off it.
;; Returns Option<String> = the raised Fault's message field, read structurally,
;; proving the error rode the boundary as a record — not a stringified blob.

(:wat::core::defn :my::compute [] -> :wat::core::Option<wat::core::String>
  (:wat::core::let
    [r
      (:wat::test::run-thread
        (:wat::kernel::raise!
          (:wat::core::Fault/of "arc113-raise-data")))
     fail
      (:wat::kernel::RunResult/failure r)]
    (:wat::core::match fail
      ((:wat::core::Some f)
       ;; STRUCTURAL read: Failure/error yields the raised :wat::core::Fault RECORD
       ;; directly (it rode the panic boundary as data); Fault/message reads the
       ;; field off it — no edn::write, no edn::read, no string round-trip.
       (:wat::core::Some
         (:wat::core::Fault/message (:wat::kernel::Failure/error f))))
      (:wat::core::None :wat::core::None))))
