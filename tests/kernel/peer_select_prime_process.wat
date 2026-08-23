;; Co-located fixture for peer_select_prime_process.rs — slurped via startup_beside(file!()).
;; #[ignore] process-tier probe (arc 214 Stone 4.6b).

;; ServiceEvent is <I,O,A> — arc 291 3a-i added A (the self-peer's admin receive type).
;; This signature was written when it was <I,O> and was never updated, so the body produced
;; a 3-param type against a 2-param declaration and the fixture stopped freezing. There is no
;; service/self-peer here (a bare `select` over two process peers), so A is unconstrained;
;; naming it i64 alongside I and O is what the fixture means.
(:wat::core::defn :user::compute [] -> (:wat::spawn::ServiceEvent :- [:wat::core::i64 :wat::core::i64 :wat::core::i64])
  (:wat::core::let
    [a (:wat::test::spawn-peer (:wat::spawn::process)
          (:wat::core::forms
            (:wat::core::defn :user::main [] -> :wat::core::nil
              (:wat::core::let
                [n (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                 _ (:wat::kernel::println (:wat::core::i64::+ n 1))]
                nil))))
     b (:wat::test::spawn-peer (:wat::spawn::process)
          (:wat::core::forms
            (:wat::core::defn :user::main [] -> :wat::core::nil
              (:wat::core::let
                [n (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                 _ (:wat::kernel::println (:wat::core::i64::+ n 1))]
                nil))))
     _ (:wat::core::match (:wat::kernel::send b 98) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) ((:wat::kernel::SendOutcome::Lost _c) nil) (:wat::kernel::SendOutcome::Stopped nil)) ;; arc 278 #73 — fire-and-forget request; outcome ignored uniformly regardless of cause
     picked (:wat::kernel::select [a b])]
    picked))

