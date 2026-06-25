;; wat-tests/service-admin-facet.wat — arc 291 strike-3a RED probe: the admin/data facet split.
;;
;; THE CONTRACT, proven at the surface: `stop` is OWNER-ONLY. It moves off the client `Op` enum onto the
;; Handle's admin surface — so its caller argument flips from a CLIENT peer (`connect'`-derived) to the
;; `Handle` itself (held only by the spawner). A client holding only the dial-`Address'` has no `stop`
;; method at all; the Handle-holder calls `(<svc>/stop handle)`.
;;
;; ONE defservice, two deftests differing in exactly one token (the locus). Modeled on
;; service-locus-parity.wat + service-init-parity.wat (uses the shipped `:init`).
;;
;; arc 291 4b-ii: State is now a defstruct; :durable [count] mints ::Record; ::State holds it.
;; :init defaults (pure-data, ephemeral empty). start takes ::Record(0).
;; Op body reads through State/durable. State building uses State/new (Record c).
;; stop defaults to (fn [s] -> ::Record (State/durable s)) → final is a ::Record.
;; Assertion reads Record/count final.

;; ── the service: a counter; Increment is a client (data-plane) op; stop is admin (control-plane) ──
(:wat::service::defservice :wat-tests::admin-counter
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :ops
  [(:Increment [s <- :State n <- :wat::core::i64]
               -> [value <- :wat::core::i64]
     (:wat::core::let [c (:wat::core::i64::+
                           (:wat-tests::admin-counter::Record/count (:wat-tests::admin-counter::State/durable s)) n)]
       (:wat::service::Outcome::Reply
         (:wat-tests::admin-counter::State/new (:wat-tests::admin-counter::Record c))
         (:wat-tests::admin-counter::IncrementResponse c))))])

;; ── thread tier ──────────────────────────────────────────────────────────────
;; A client (dial-Address') does the data op; the Handle-holder issues the admin stop.
;; stop takes the HANDLE (h), not the client peer (c) — owner-only by construction.
;; stop defaults to returning the ::Record — extract count via Record/count.
(:wat::test::deftest' :wat-tests::service::admin-stop-on-thread
  ()
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::admin-counter/start :locus (:wat::spawn::thread) :record (:wat-tests::admin-counter::Record 0))
       c (:wat::kernel::connect' (:wat-tests::admin-counter::Handle/addr h))
       _ (:wat-tests::admin-counter/increment c (:wat-tests::admin-counter/increment-request 7))
       final (:wat-tests::admin-counter/stop h)]
      (:wat-tests::admin-counter::Record/count final))
    7))

;; ── process tier — IDENTICAL except the locus token ──────────────────────────
(:wat::test::deftest' :wat-tests::service::admin-stop-on-process
  ()
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::admin-counter/start :locus (:wat::spawn::process) :record (:wat-tests::admin-counter::Record 0))
       c (:wat::kernel::connect' (:wat-tests::admin-counter::Handle/addr h))
       _ (:wat-tests::admin-counter/increment c (:wat-tests::admin-counter/increment-request 7))
       final (:wat-tests::admin-counter/stop h)]
      (:wat-tests::admin-counter::Record/count final))
    7))
