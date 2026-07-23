;; tests/comms/wat_arc113_raise_round_trip.wat — co-located fixture for the raise round-trip probe,
;; slurped via startup_beside(file!()). No placeholder main — startup_beside loads defns only.
;;
;; Arc 296 re-gate: raise! now requires :wat::core::Error. The round-trip is
;; STRUCTURAL — the error survives the panic boundary as data, not a string:
;;   (raise! (Fault/of "arc113-raise-data"))
;;     → Failure/message carries the Fault's EDN (#wat.core/Fault {…})
;;     → (edn::read …) LIFTS IT BACK to a :wat::core::Fault RECORD
;;     → (Fault/message …) reads the message field off the reconstructed record.
;; Returns Option<String> = the reconstructed record's message, proving the Fault
;; round-trips as a structured record (reconstruct_record), not a raw string blob.

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
       ;; STRUCTURAL round-trip: edn::read lifts the Failure's EDN back to a
       ;; :wat::core::Fault RECORD (reconstruct_record); Fault/message then reads
       ;; the field off the reconstructed record — not the raw string.
       (:wat::core::Some
         (:wat::core::Fault/message
           (:wat::edn::read (:wat::kernel::Failure/message f)))))
      (:wat::core::None :wat::core::None))))
