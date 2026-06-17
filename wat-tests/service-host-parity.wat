;; wat-tests/service-host-parity.wat — arc 272 6b-iii: the host-parity proof at the WAT level.
;;
;; ONE defservice (the counter); two deftests that differ in EXACTLY ONE token — the host
;; (:wat::spawn::thread) vs (:wat::spawn::process). The generated client face (start, connect',
;; increment/get, the request constructors, the Handle, the Response accessor) is byte-identical.
;; This is the parity contract written as a test: swap the host, the same service runs.
;;
;; The Rust-level proof is `tests/probe_arc272_6b_defservice_on_process.rs` (a forking [[test]] binary);
;; this dogfoods the same surface in wat. defservice names NO transport — the (process) literal the
;; service rides lives only in the ProcessOpts `launch` arm (design C).

;; ── the service, defined once at top-level (shared by both deftests) ──────────
(:wat::service::defservice :wat-tests::counter
  :state :wat::core::i64
  :ops
  [(:Get [s <- :State]
         -> [value <- :wat::core::i64]
     (:wat::service::Outcome::Reply s (:wat-tests::counter::GetResponse s)))
   (:Increment [s <- :State n <- :wat::core::i64]
               -> [value <- :wat::core::i64]
     (:wat::core::let [s' (:wat::core::i64::+ s n)]
       (:wat::service::Outcome::Reply s' (:wat-tests::counter::IncrementResponse s'))))])

;; ── thread tier ──────────────────────────────────────────────────────────────
(:wat::test::deftest' :wat-tests::service::counter-on-thread
  ()
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::counter/start (:wat::spawn::thread) 0)
       c (:wat::kernel::connect' (:wat-tests::counter::Handle/addr h))
       _ (:wat-tests::counter/increment c (:wat-tests::counter/increment-request 5))
       r (:wat-tests::counter/get c (:wat-tests::counter/get-request))]
      (:wat-tests::counter::GetResponse/value r))
    5))

;; ── process tier — IDENTICAL except the host token ───────────────────────────
(:wat::test::deftest' :wat-tests::service::counter-on-process
  ()
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::counter/start (:wat::spawn::process) 0)
       c (:wat::kernel::connect' (:wat-tests::counter::Handle/addr h))
       _ (:wat-tests::counter/increment c (:wat-tests::counter/increment-request 5))
       r (:wat-tests::counter/get c (:wat-tests::counter/get-request))]
      (:wat-tests::counter::GetResponse/value r))
    5))
