;; wat-tests/service-locus-parity.wat — arc 272 6b-iii: the locus-parity proof at the WAT level.
;;
;; ONE defservice (the counter); two deftests that differ in EXACTLY ONE token — the locus
;; (:wat::spawn::thread) vs (:wat::spawn::process). The generated client face (start, connect',
;; increment/get, the request constructors, the Handle, the Response accessor) is byte-identical.
;; This is the parity contract written as a test: swap the locus, the same service runs.
;;
;; The Rust-level proof is `tests/probe_arc272_6b_defservice_on_process.rs` (a forking [[test]] binary);
;; this dogfoods the same surface in wat. defservice names NO transport — the (process) literal the
;; service rides lives only in the ProcessOpts `launch` arm (design C).
;;
;; arc 291 4b-ii: State is now a defstruct; :durable mints ::Record (the soul); ::State holds it.
;; start takes a ::Record (not a pre-built ::State). Accessors read through State/durable.

;; ── the service, defined once at top-level (shared by both deftests) ──────────
(:wat::service::defservice :wat-tests::counter
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :ops
  [(:Get [s <- :State]
         -> [value <- :wat::core::i64]
     (:wat::service::Outcome::Reply s
       (:wat-tests::counter::GetResponse
         (:wat-tests::counter::Record/count (:wat-tests::counter::State/durable s)))))
   (:Increment [s <- :State n <- :wat::core::i64]
               -> [value <- :wat::core::i64]
     (:wat::core::let [c (:wat::core::i64::+
                           (:wat-tests::counter::Record/count (:wat-tests::counter::State/durable s)) n)]
       (:wat::service::Outcome::Reply
         (:wat-tests::counter::State (:wat-tests::counter::Record c))
         (:wat-tests::counter::IncrementResponse c))))])

;; ── thread tier ──────────────────────────────────────────────────────────────
(:wat::test::deftest' :wat-tests::service::counter-on-thread
  ()
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::counter/start :locus (:wat::spawn::thread) :record (:wat-tests::counter::Record 0))
       c (:wat::kernel::connect' (:wat-tests::counter::Handle/addr h))
       _ (:wat-tests::counter/increment c (:wat-tests::counter/increment-request 5))
       r (:wat-tests::counter/get c (:wat-tests::counter/get-request))]
      (:wat-tests::counter::GetResponse/value r))
    5))

;; ── process tier — IDENTICAL except the locus token ──────────────────────────
(:wat::test::deftest' :wat-tests::service::counter-on-process
  ()
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::counter/start :locus (:wat::spawn::process) :record (:wat-tests::counter::Record 0))
       c (:wat::kernel::connect' (:wat-tests::counter::Handle/addr h))
       _ (:wat-tests::counter/increment c (:wat-tests::counter/increment-request 5))
       r (:wat-tests::counter/get c (:wat-tests::counter/get-request))]
      (:wat-tests::counter::GetResponse/value r))
    5))
