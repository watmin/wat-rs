;; wat-tests/service-stop-resp.wat — arc 291 strike-3b RED probe: stop → resp decouple.
;;
;; THE CONTRACT, proven at the surface: `stop`'s RETURN is DECOUPLED from the live State. A `:stop`
;; callback projects the final State → a serializable `resp` of the AUTHOR'S type — here `:i64` (the count),
;; NOT the `::Record`. `(<svc>/stop h)` returns that i64 directly. This is the out-locus mirror of
;; `:init` (which builds State from a Record in-locus); `:stop` renders State to resp out-locus.
;;
;; ONE defservice, two deftests differing in exactly one token (the locus). Modeled on
;; service-admin-facet.wat (owner-only stop via the Handle) + the shipped `:init`/`:stop` callbacks.
;;
;; arc 291 4b-ii: State is now a defstruct; :durable [count] mints ::Record; ::State holds it.
;; :init defaults (pure-data, ephemeral empty). start takes ::Record(0).
;; Op body reads through State/durable. State building uses State/new (Record c).
;; :stop projection now reads through State/durable: (Record/count (State/durable s)).

;; ── the service: a counter; :stop projects State → i64 (the count) ──
(:wat::service::defservice :wat-tests::resp-counter
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :ops
  [(:Increment [s <- :State n <- :wat::core::i64]
               -> [value <- :wat::core::i64]
     (:wat::core::let [c (:wat::core::i64::+
                           (:wat-tests::resp-counter::Record/count (:wat-tests::resp-counter::State/durable s)) n)]
       (:wat::service::Outcome::Reply
         (:wat-tests::resp-counter::State/new (:wat-tests::resp-counter::Record c))
         (:wat-tests::resp-counter::IncrementResponse c))))  ]
  ;; :stop — the projection: final State → its count (an i64). The stop RETURN is this i64,
  ;; decoupled from the ::Record. Read count through State/durable.
  :stop (:wat::core::fn [s <- :wat-tests::resp-counter::State] -> :wat::core::i64
          (:wat-tests::resp-counter::Record/count (:wat-tests::resp-counter::State/durable s))))

;; ── thread tier ──────────────────────────────────────────────────────────────
;; Increment to 7; the Handle-holder stops; stop returns the PROJECTED i64 (7), not a Record.
(:wat::test::deftest' :wat-tests::service::stop-resp-on-thread
  ()
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::resp-counter/start :locus (:wat::spawn::thread) :record (:wat-tests::resp-counter::Record 0))
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
      [h (:wat-tests::resp-counter/start :locus (:wat::spawn::process) :record (:wat-tests::resp-counter::Record 0))
       c (:wat::kernel::connect' (:wat-tests::resp-counter::Handle/addr h))
       _ (:wat-tests::resp-counter/increment c (:wat-tests::resp-counter/increment-request 7))
       final (:wat-tests::resp-counter/stop h)]
      final)
    7))
