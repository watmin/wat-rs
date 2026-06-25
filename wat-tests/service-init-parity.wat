;; wat-tests/service-init-parity.wat — arc 291 strike-1 RED probe: the `:init` keystone, both loci.
;;
;; THE PROPHECY, proven small: a service whose State is built by an `:init` callback FROM EDN ARGS,
;; run IN-LOCUS — so `start` takes an EDN seed (42), not a pre-built State. ONE defservice, two
;; deftests differing in EXACTLY one token (the locus). Modeled byte-for-byte on the GREEN
;; `service-locus-parity.wat`; the ONLY addition is the `:init` clause.
;;
;; arc 291 4b-ii: State is now a defstruct; :durable [count] mints ::Record; ::State holds it.
;; :init now defaults to (fn [d <- ::Record] -> ::State (::State/new d)) for pure-data services.
;; start takes a ::Record (not a raw i64). The "seeded" semantics now live in start taking the
;; record: (seeded-counter/start locus (seeded-counter::Record 42)).
;; Op body reads count through State/durable.

;; ── the service, defined once at top-level (shared by both deftests) ──────────
;; :init defaults — pure-data service, ephemeral empty → default init = (fn [d <- ::Record] -> ::State (::State/new d))
(:wat::service::defservice :wat-tests::seeded-counter
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :ops
  [(:Get [s <- :State]
         -> [value <- :wat::core::i64]
     (:wat::service::Outcome::Reply s
       (:wat-tests::seeded-counter::GetResponse
         (:wat-tests::seeded-counter::Record/count (:wat-tests::seeded-counter::State/durable s)))))])

;; ── thread tier ──────────────────────────────────────────────────────────────
;; start takes the Record (seeded-counter::Record 42); init defaults to State/new(d).
(:wat::test::deftest' :wat-tests::service::seeded-counter-on-thread
  ()
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::seeded-counter/start :locus (:wat::spawn::thread) :record (:wat-tests::seeded-counter::Record 42))
       c (:wat::kernel::connect' (:wat-tests::seeded-counter::Handle/addr h))
       r (:wat-tests::seeded-counter/get c (:wat-tests::seeded-counter/get-request))]
      (:wat-tests::seeded-counter::GetResponse/value r))
    42))

;; ── process tier — IDENTICAL except the locus token ──────────────────────────
;; the Record crosses the wire; init builds State child-side; State never crosses.
(:wat::test::deftest' :wat-tests::service::seeded-counter-on-process
  ()
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::seeded-counter/start :locus (:wat::spawn::process) :record (:wat-tests::seeded-counter::Record 42))
       c (:wat::kernel::connect' (:wat-tests::seeded-counter::Handle/addr h))
       r (:wat-tests::seeded-counter/get c (:wat-tests::seeded-counter/get-request))]
      (:wat-tests::seeded-counter::GetResponse/value r))
    42))
