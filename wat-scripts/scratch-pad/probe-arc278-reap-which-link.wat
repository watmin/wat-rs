;; probe-arc278-reap-which-link.wat — WHICH dropped value kills the service?
;;
;; Companion measurement to probe-arc278-tco-drops-caller-env.wat (which shows THAT the
;; caller's env is dropped before the tail callee runs). This one asks WHICH link in the
;; dropped env is load-bearing: the Handle's LINEAGE peer (the admin channel, poll' index 0)
;; or the Handle's ADDRESS (the listener rendezvous, poll' index 1).
;;
;; Method: a tail call that carries the Handle THROUGH as an argument. `emit_tail_call`
;; evaluates the args BEFORE signalling, so an arg-carried Handle survives the env drop.
;; If carrying `h` restores service, the Handle IS the load-bearing reference.
;;
;;   row 1  non-tail, h in scope            -> served      (control)
;;   row 2  tail, h NOT carried             -> CLOSED      (the bug)
;;   row 3  tail, h CARRIED as an argument  -> ?           (the discriminator)

(:wat::core::defsurface :rw::Bag :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :rw::Bag::PutRequest [n <- :wat::core::i64])
   (:wat::core::defenum :rw::Bag::PutResponse :wat::enum::Pure
     :Ok               [n <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(put [self <- :rw::Bag  req <- :rw::Bag::PutRequest]
     -> :rw::Bag::PutResponse :max-request-bytes 4096)])

(:wat::service::defservice :rw::bag-svc
  :satisfies :rw::Bag  :durable [n <- :wat::core::i64]  :ephemeral []
  :impls
  [(put [s ctx req] (:wat::service::Outcome::Reply s (:rw::Bag::PutResponse::Ok 1)))])

(:wat::core::defn :rw::try [c <- (:wat::kernel::Peer :- [:rw::Bag::Op :rw::Bag::Reply])
                           label <- :wat::core::String] -> :wat::core::nil
  (:wat::core::match (:rw::Bag/put c (:rw::Bag::PutRequest :n 1))
    ((:wat::kernel::RecvOutcome::Message resp)
      (:wat::kernel::println (:wat::string::concat label " => Message (served)")))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::println (:wat::string::concat label " => LOST")))
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::println (:wat::string::concat label " => STOPPED")))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::println (:wat::string::concat label " => CLOSED")))))

;; The discriminator: the Handle rides in as an argument, so it outlives the caller's env.
(:wat::core::defn :rw::try-with-handle [c <- (:wat::kernel::Peer :- [:rw::Bag::Op :rw::Bag::Reply])
                                       h <- :rw::bag-svc::Handle
                                       label <- :wat::core::String] -> :wat::core::nil
  ;; NOT a tail call — a bare `(:rw/try c label)` here would itself tail-drop `h` before
  ;; the put ran, re-manufacturing the very reap this row is trying to rule out.
  (:wat::core::do (:rw::try c label) nil))

;; Field-level discriminators: carry ONLY the lineage peer, or ONLY the address.
(:wat::core::defn :rw::try-with-lineage [c <- (:wat::kernel::Peer :- [:rw::Bag::Op :rw::Bag::Reply])
                                        lp <- (:wat::kernel::Peer :- [:rw::bag-svc::Admin :rw::bag-svc::Status])
                                        label <- :wat::core::String] -> :wat::core::nil
  (:wat::core::do (:rw::try c label) nil))

(:wat::core::defn :rw::try-with-addr [c <- (:wat::kernel::Peer :- [:rw::Bag::Op :rw::Bag::Reply])
                                     a <- (:wat::kernel::Address :- [:rw::Bag::Op :rw::Bag::Reply])
                                     label <- :wat::core::String] -> :wat::core::nil
  (:wat::core::do (:rw::try c label) nil))

(:wat::core::defn :rw::row-non-tail [] -> :wat::core::nil
  (:wat::core::let
    [h (:rw::bag-svc/start :locus (:wat::spawn::thread) :record (:rw::bag-svc::Record :n 0))
     c (:wat::core::match (:wat::kernel::connect (:rw::bag-svc::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected p) p)
         ((:wat::kernel::ConnectOutcome::Refused f)  (:wat::kernel::assertion-failed! "refused" :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Rejected f) (:wat::kernel::assertion-failed! "rejected" :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Failed f)   (:wat::kernel::assertion-failed! "failed" :wat::core::None :wat::core::None)))]
    (:wat::core::do (:rw::try c "row1 non-tail       ") nil)))

;; rune:check(handle-lifetime-creation-escape) — INSTRUMENT: this file measures
;; which Handle field keeps a tail-called service alive. It must construct the
;; escape. Rune the instrument, never the acceptance criterion.
(:wat::core::defn :rw::row-tail-bare [] -> :wat::core::nil
  (:wat::core::let
    [h (:rw::bag-svc/start :locus (:wat::spawn::thread) :record (:rw::bag-svc::Record :n 0))
     c (:wat::core::match (:wat::kernel::connect (:rw::bag-svc::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected p) p)
         ((:wat::kernel::ConnectOutcome::Refused f)  (:wat::kernel::assertion-failed! "refused" :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Rejected f) (:wat::kernel::assertion-failed! "rejected" :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Failed f)   (:wat::kernel::assertion-failed! "failed" :wat::core::None :wat::core::None)))]
    (:rw::try c "row2 tail, no handle")))

;; rune:check(handle-lifetime-creation-escape) — INSTRUMENT: tail escape with Handle
;; carried as an extra arg; the probe must construct it.
(:wat::core::defn :rw::row-tail-carry-handle [] -> :wat::core::nil
  (:wat::core::let
    [h (:rw::bag-svc/start :locus (:wat::spawn::thread) :record (:rw::bag-svc::Record :n 0))
     c (:wat::core::match (:wat::kernel::connect (:rw::bag-svc::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected p) p)
         ((:wat::kernel::ConnectOutcome::Refused f)  (:wat::kernel::assertion-failed! "refused" :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Rejected f) (:wat::kernel::assertion-failed! "rejected" :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Failed f)   (:wat::kernel::assertion-failed! "failed" :wat::core::None :wat::core::None)))]
    (:rw::try-with-handle c h "row3 tail, carry h  ")))

;; rune:check(handle-lifetime-creation-escape) — INSTRUMENT: tail escape with lineage
;; carried as an extra arg; the probe must construct it.
(:wat::core::defn :rw::row-tail-carry-lineage [] -> :wat::core::nil
  (:wat::core::let
    [h (:rw::bag-svc/start :locus (:wat::spawn::thread) :record (:rw::bag-svc::Record :n 0))
     c (:wat::core::match (:wat::kernel::connect (:rw::bag-svc::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected p) p)
         ((:wat::kernel::ConnectOutcome::Refused f)  (:wat::kernel::assertion-failed! "refused" :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Rejected f) (:wat::kernel::assertion-failed! "rejected" :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Failed f)   (:wat::kernel::assertion-failed! "failed" :wat::core::None :wat::core::None)))]
    (:rw::try-with-lineage c (:rw::bag-svc::Handle/handle h) "row4 tail, carry lineage")))

;; rune:check(handle-lifetime-creation-escape) — INSTRUMENT: tail escape with addr
;; carried as an extra arg; the probe must construct it.
(:wat::core::defn :rw::row-tail-carry-addr [] -> :wat::core::nil
  (:wat::core::let
    [h (:rw::bag-svc/start :locus (:wat::spawn::thread) :record (:rw::bag-svc::Record :n 0))
     c (:wat::core::match (:wat::kernel::connect (:rw::bag-svc::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected p) p)
         ((:wat::kernel::ConnectOutcome::Refused f)  (:wat::kernel::assertion-failed! "refused" :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Rejected f) (:wat::kernel::assertion-failed! "rejected" :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Failed f)   (:wat::kernel::assertion-failed! "failed" :wat::core::None :wat::core::None)))]
    (:rw::try-with-addr c (:rw::bag-svc::Handle/addr h) "row5 tail, carry addr   ")))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:rw::row-non-tail)
    (:rw::row-tail-bare)
    (:rw::row-tail-carry-handle)
    (:rw::row-tail-carry-lineage)
    (:rw::row-tail-carry-addr)
    nil))
