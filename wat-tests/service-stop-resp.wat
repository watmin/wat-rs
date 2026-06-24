;; wat-tests/service-stop-resp.wat — arc 291 strike-3b RED probe: stop → resp decouple.
;;
;; THE CONTRACT, proven at the surface: `stop`'s RETURN is DECOUPLED from the live State. A `:stop`
;; callback projects the final State → a serializable `resp` of the AUTHOR'S type — here `:i64` (the count),
;; NOT the `:State` record. `(<svc>/stop h)` returns that i64 directly. This is the out-locus mirror of
;; `:init` (which builds State from an EDN seed in-locus); `:stop` renders State to resp out-locus.
;;
;; ONE defservice, two deftests differing in exactly one token (the locus). Modeled on
;; service-admin-facet.wat (owner-only stop via the Handle) + the shipped `:init`/`:stop` callbacks.
;;
;; RED at HEAD: `:stop` is an UNKNOWN trailing option → defservice macro-errors
;; ("unknown trailing option :stop"). GREEN once `:stop` is supported: stop projects State → i64 and
;; `(resp-counter/stop h)` returns 7 (the projected i64), NOT a State record.

;; ── the service: a counter; :init seeds from i64; :stop projects State → i64 (the count) ──
(:wat::service::defservice :wat-tests::resp-counter
  :state [count <- :wat::core::i64]
  :ops
  [(:Increment [s <- :State n <- :wat::core::i64]
               -> [value <- :wat::core::i64]
     (:wat::core::let [s' (:wat::core::i64::+ (:wat-tests::resp-counter::State/count s) n)]
       (:wat::service::Outcome::Reply (:wat-tests::resp-counter::State s')
         (:wat-tests::resp-counter::IncrementResponse s'))))]
  :init (:wat::core::fn [seed <- :wat::core::i64] -> :wat-tests::resp-counter::State
          (:wat-tests::resp-counter::State seed))
  ;; :stop — the projection: final State → its count (an i64). The stop RETURN is this i64,
  ;; decoupled from the live State record. (Default would be identity → Resp = State.)
  :stop (:wat::core::fn [s <- :wat-tests::resp-counter::State] -> :wat::core::i64
          (:wat-tests::resp-counter::State/count s)))

;; ── thread tier ──────────────────────────────────────────────────────────────
;; Increment to 7; the Handle-holder stops; stop returns the PROJECTED i64 (7), not a State.
(:wat::test::deftest' :wat-tests::service::stop-resp-on-thread
  ()
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::resp-counter/start (:wat::spawn::thread) 0)
       c (:wat::kernel::connect' (:wat-tests::resp-counter::Handle/addr h))
       _ (:wat-tests::resp-counter/increment c (:wat-tests::resp-counter/increment-request 7))
       final (:wat-tests::resp-counter/stop h)]
      final)
    7))

;; ── process tier — IDENTICAL except the locus token ──────────────────────────
(:wat::test::deftest' :wat-tests::service::stop-resp-on-process
  ()
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::resp-counter/start (:wat::spawn::process) 0)
       c (:wat::kernel::connect' (:wat-tests::resp-counter::Handle/addr h))
       _ (:wat-tests::resp-counter/increment c (:wat-tests::resp-counter/increment-request 7))
       final (:wat-tests::resp-counter/stop h)]
      final)
    7))
