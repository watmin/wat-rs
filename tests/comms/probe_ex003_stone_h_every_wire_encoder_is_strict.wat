;; tests/comms/probe_ex003_stone_h_every_wire_encoder_is_strict.wat — co-located fixture for
;; probe_ex003_stone_h_every_wire_encoder_is_strict.rs (startup_beside). No placeholder main at the top
;; level; each inner :user::main is a spawned CHILD's entrypoint.
;;
;; Excursus 003 stone H — every encoder that ships bytes a peer decodes as DATA is strict (stone G's
;; loose ends). Measured BEFORE, at eef4f0003:
;;   - a process-tier `after` whose msg held an Lru returned its timer peer; the `select` that fired
;;     it raised "select (process tier) EDN decode failed: src/edn/render.rs:3587:19: unsupported
;;     substrate tag #rust.cache/Lru has a bare-nil body — retired (arc 278 A.0) …";
;;   - a THREAD-tier `try-send` of a holon raised ":wat::edn::write … cannot encode HolonAST to the
;;     wire", located in src/edn/render.rs — for a value the thread tier never encodes (`send` of the
;;     same value: Sent);
;;   - a spawned child's `println` of a Box<Lru> reached the parent as `RecvOutcome.Lost` "recv EDN
;;     decode failed … #rust.cache/Lru has a bare-nil body" — a child's stdout IS its wire.
;; AFTER: each raises at the user's call, or (thread tier) is Sent.

(:wat::core::defrecord :h::Box :- [T] [x <- :T])

;; ── `after`, process tier ──────────────────────────────────────────────────────────────────────
;; A generic fn: `T` is a parameter where the timer is typed.
(:wat::core::defn :h::fire :- [T] [x <- :T] -> :wat::core::String
  (:wat::core::match
    (:wat::kernel::select
      (:wat::core::Vector :- [(:wat::kernel::Peer :- [:wat::core::nil (:h::Box :- [:T])])]
        (:wat::kernel::after :wat::program::PeerKind.process (:wat::time::Millisecond 5) (:h::Box :x x))))
    [:wat::spawn::ServiceEvent.Message {:idx _i :msg m} (:h::fired m)]
    [:wat::spawn::ServiceEvent.Closed {:idx _i} "Closed"]
    [:wat::spawn::ServiceEvent.Lost {:idx _i :cause _c} "Lost"]
    [:wat::spawn::ServiceEvent.Malformed {:idx _i :cause _c} "Malformed"]
    [:wat::spawn::ServiceEvent.Rejected {:idx _i :cause _c} "Rejected"]
    [:wat::spawn::ServiceEvent.Shutdown {} "Shutdown"]
    [:wat::spawn::ServiceEvent.Connection {:peer _p} "Connection"]
    [:wat::spawn::ServiceEvent.Admin {:msg _m} "Admin"]))

(:wat::core::defn :h::fired :- [T] [b <- (:h::Box :- [:T])] -> :wat::core::String
  (:wat::core::format "Message: {w}" :w (:wat::edn::write b)))

;; (a) T = Lru — raises at the `after`, in this (the parent) process.
(:wat::core::defn :h::probe-after-handle [] -> :wat::core::String
  (:h::fire (:wat::core::Result/expect (:wat::cache::Lru/new :- [:wat::core::i64 :wat::core::i64] 2) "lru")))

;; (b) T = i64 — the timer delivers the msg whole.
(:wat::core::defn :h::probe-after-pure [] -> :wat::core::String
  (:h::fire 42))

;; ── `try-send`, thread tier ────────────────────────────────────────────────────────────────────
;; (c) A holon — which the EDN writer cannot encode — over a thread-tier peer: a thread peer carries
;; the Value itself, so there is nothing to encode.
(:wat::core::defn :h::probe-thread-try-send-holon [] -> :wat::core::String
  (:wat::core::let
    [bound (:wat::kernel::listener (:wat::spawn::thread) :wat::holon::HolonAST :wat::holon::HolonAST)
     tx    (:wat::core::match (:wat::kernel::connect (:wat::spawn::Bound/address bound))
             [:wat::kernel::ConnectOutcome.Connected {:peer p} p]
             [:wat::kernel::ConnectOutcome.Refused {:cause _c} (:wat::kernel::assertion-failed! :message "refused")]
             [:wat::kernel::ConnectOutcome.Rejected {:cause _c} (:wat::kernel::assertion-failed! :message "rejected")]
             [:wat::kernel::ConnectOutcome.Failed {:cause _c} (:wat::kernel::assertion-failed! :message "failed")])
     rx    (:wat::core::match (:wat::kernel::accept (:wat::spawn::Bound/listener bound))
             [:wat::kernel::AcceptOutcome.Accepted {:peer p} p]
             [:wat::kernel::AcceptOutcome.Closed {} (:wat::kernel::assertion-failed! :message "closed")]
             [:wat::kernel::AcceptOutcome.Failed {:cause _c} (:wat::kernel::assertion-failed! :message "failed")])
     sent  (:wat::core::match (:wat::kernel::try-send tx #holon [1 2 3])
             [:wat::kernel::TrySendOutcome.Sent {} "Sent"]
             [:wat::kernel::TrySendOutcome.WouldBlock {} "WouldBlock"]
             [:wat::kernel::TrySendOutcome.Closed {} "Closed"]
             [:wat::kernel::TrySendOutcome.Lost {:cause _c} "Lost"])]
    (:wat::core::match (:wat::kernel::recv rx)
      [:wat::kernel::RecvOutcome.Message {:msg m}
        (:wat::core::format "{s}; arrived equal: {e}" :s sent :e (:wat::core::= m #holon [1 2 3]))]
      [:wat::kernel::RecvOutcome.Lost {:cause _c} "Lost"]
      [:wat::kernel::RecvOutcome.Stopped {} "Stopped"]
      [:wat::kernel::RecvOutcome.Closed {} "Closed"])))

;; ── `println` / `pprintln` in a spawned process — its stdout IS its wire ───────────────────────
;; (d) The child prints a Box<Lru>; its println raises, and the parent reads that death as the Lost.
(:wat::core::defn :h::probe-child-println-handle [] -> :wat::core::String
  (:wat::core::let
    [svc (:wat::test::spawn-peer (:wat::spawn::process)
           (:wat::core::forms
             (:wat::core::defrecord :h::Box :- [T] [x <- :T])
             (:wat::core::defn :h::show :- [T] [x <- :T] -> :wat::core::nil
               (:wat::kernel::println (:h::Box :x x)))
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:wat::core::let
                 [_p (:wat::program::self-peer (:h::Box :- [:wat::core::i64]) :wat::core::i64)]
                 (:h::show (:wat::core::Result/expect
                             (:wat::cache::Lru/new :- [:wat::core::i64 :wat::core::i64] 2) "lru"))))))]
    (:h::child-said svc)))

;; (e) The same through pprintln — the pretty writer has no strict mode; the strict encoder is asked first.
(:wat::core::defn :h::probe-child-pprintln-handle [] -> :wat::core::String
  (:wat::core::let
    [svc (:wat::test::spawn-peer (:wat::spawn::process)
           (:wat::core::forms
             (:wat::core::defrecord :h::Box :- [T] [x <- :T])
             (:wat::core::defn :h::show :- [T] [x <- :T] -> :wat::core::nil
               (:wat::kernel::pprintln (:h::Box :x x)))
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:wat::core::let
                 [_p (:wat::program::self-peer (:h::Box :- [:wat::core::i64]) :wat::core::i64)]
                 (:h::show (:wat::core::Result/expect
                             (:wat::cache::Lru/new :- [:wat::core::i64 :wat::core::i64] 2) "lru"))))))]
    (:h::child-said svc)))

;; (f) A pure Box printed by the child arrives at the parent whole — println there is a send.
(:wat::core::defn :h::probe-child-println-pure [] -> :wat::core::String
  (:wat::core::let
    [svc (:wat::test::spawn-peer (:wat::spawn::process)
           (:wat::core::forms
             (:wat::core::defrecord :h::Box :- [T] [x <- :T])
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:wat::core::let
                 [_p (:wat::program::self-peer (:h::Box :- [:wat::core::i64]) :wat::core::i64)]
                 (:wat::kernel::println (:h::Box :x 42))))))]
    (:h::child-said svc)))

(:wat::core::defn :h::child-said
  [svc <- (:wat::kernel::Process :- [:wat::core::i64 (:h::Box :- [:wat::core::i64])])] -> :wat::core::String
  (:wat::core::match (:wat::kernel::recv svc)
    [:wat::kernel::RecvOutcome.Message {:msg m} (:h::fired m)]
    [:wat::kernel::RecvOutcome.Lost {:cause c} (:wat::kernel::LociDiedError/message c)]
    [:wat::kernel::RecvOutcome.Stopped {} "Stopped"]
    [:wat::kernel::RecvOutcome.Closed {} "Closed"]))
