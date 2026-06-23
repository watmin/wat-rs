;; wat-tests/service-init-parity.wat — arc 291 strike-1 RED probe: the `:init` keystone, both loci.
;;
;; THE PROPHECY, proven small: a service whose State is built by an `:init` callback FROM EDN ARGS,
;; run IN-LOCUS — so `start` takes an EDN seed (42), not a pre-built State. ONE defservice, two
;; deftests differing in EXACTLY one token (the locus). Modeled byte-for-byte on the GREEN
;; `service-locus-parity.wat`; the ONLY addition is the `:init` clause.
;;
;; RED at HEAD: the defservice macro does not know `:init` — it is not in `known-opts`
;; (wat/service.wat:74), so expansion macro-errors "unknown trailing option :init". That macro-error
;; IS the gap arc 291 strike-2 fills. GREEN once `:init` lands: `start` takes the seed, the locus runs
;; `(init seed)` in-place to build State (thread: in the spawned thread; process: child-side after
;; recv'ing the EDN seed), and Get returns the seeded count — the soul built where it lives, the wire
;; carrying only EDN.

;; ── the service, defined once at top-level (shared by both deftests) ──────────
;; :init builds State from an EDN seed; the State (count) is NOT passed pre-built to start.
(:wat::service::defservice :wat-tests::seeded-counter
  :state [count <- :wat::core::i64]
  :ops
  [(:Get [s <- :State]
         -> [value <- :wat::core::i64]
     (:wat::service::Outcome::Reply s (:wat-tests::seeded-counter::GetResponse (:wat-tests::seeded-counter::State/count s))))]
  :init (:wat::core::fn [seed <- :wat::core::i64] -> :wat-tests::seeded-counter::State
          (:wat-tests::seeded-counter::State seed)))

;; ── thread tier ──────────────────────────────────────────────────────────────
;; start takes the EDN seed 42 (not (State 42)); init builds State in the spawned thread.
;; IGNORE-MARKED RED: `:init` is unbuilt (strike-1 disconfirming probe). The strike-2 build
;; REMOVES this ignore — un-ignoring it green is the kill's proof. (arc 122 ignore convention.)
(:wat::test::deftest' :wat-tests::service::seeded-counter-on-thread
  ()
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::seeded-counter/start (:wat::spawn::thread) 42)
       c (:wat::kernel::connect' (:wat-tests::seeded-counter::Handle/addr h))
       r (:wat-tests::seeded-counter/get c (:wat-tests::seeded-counter/get-request))]
      (:wat-tests::seeded-counter::GetResponse/value r))
    42))

;; ── process tier — IDENTICAL except the locus token ──────────────────────────
;; the EDN seed crosses the wire; init builds State child-side; State never crosses.
(:wat::test::deftest' :wat-tests::service::seeded-counter-on-process
  ()
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::seeded-counter/start (:wat::spawn::process) 42)
       c (:wat::kernel::connect' (:wat-tests::seeded-counter::Handle/addr h))
       r (:wat-tests::seeded-counter/get c (:wat-tests::seeded-counter/get-request))]
      (:wat-tests::seeded-counter::GetResponse/value r))
    42))
