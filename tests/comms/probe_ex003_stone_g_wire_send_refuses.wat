;; tests/comms/probe_ex003_stone_g_wire_send_refuses.wat — co-located fixture for
;; probe_ex003_stone_g_wire_send_refuses.rs (startup_beside). No placeholder main at the top level;
;; each inner :user::main is a spawned CHILD's entrypoint.
;;
;; Excursus 003 stone G — a wire `send` refuses a value that cannot cross, AT THE SENDER.
;; A generic fn opens `(self-peer (Box :- [T]) i64)`: the compile-time wire wall sees `T` and lets it
;; through, so an `Lru` handle reaches the wire at runtime. Each parent fn returns a String naming
;; what the parent saw.
;;
;; BEFORE this stone (measured at 2ff12c5cf): the child's send returned `SendOutcome.Sent` (and
;; `try-send` `TrySendOutcome.Sent`) — the handle shipped as `#rust.cache/Lru nil` — and only the
;; PARENT learned, as `RecvOutcome.Lost` "recv EDN decode failed: src/edn/render.rs:3587:19:
;; unsupported substrate tag #rust.cache/Lru has a bare-nil body — retired (arc 278 A.0) …". The
;; parent's own send down a `Process` peer (a3) returned `Sent`. AFTER: every wire arm raises at the
;; user's span, naming the value's type and the handle's.

;; The parent decodes a Box off the wire, so it must know the record too (a forked child runs a
;; FRESH startup and does not inherit these top-level defs — each child re-declares it).
(:wat::core::defrecord :g::Box :- [T] [x <- :T])

;; (a) T = Lru over `send` on the child's self-peer (`:wat::kernel::Peer`, socket tier). The parent
;; returns the Lost cause's message — the child's death, i.e. its send's raise.
(:wat::core::defn :g::probe-handle [] -> :wat::core::String
  (:wat::core::let
    [svc (:wat::test::spawn-peer (:wat::spawn::process)
           (:wat::core::forms
             (:wat::core::defrecord :g::Box :- [T] [x <- :T])
             (:wat::core::defn :g::ship :- [T] [x <- :T] -> :wat::core::String
               (:wat::core::match
                 (:wat::kernel::send (:wat::program::self-peer (:g::Box :- [:T]) :wat::core::i64)
                                     (:g::Box :x x))
                 [:wat::kernel::SendOutcome.Sent {} "Sent"]
                 [:wat::kernel::SendOutcome.Closed {} "Closed"]
                 [:wat::kernel::SendOutcome.Lost {:cause c}
                   (:wat::core::format "Lost: {m}" :m (:wat::kernel::LociDiedError/message c))]
                 [:wat::kernel::SendOutcome.Stopped {} "Stopped"]))
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:wat::core::let
                 [h       (:wat::core::Result/expect
                            (:wat::cache::Lru/new :- [:wat::core::i64 :wat::core::i64] 2) "lru")
                  _       (:g::ship h)]
                 nil))))]
    (:wat::core::match (:wat::kernel::recv svc)
      [:wat::kernel::RecvOutcome.Message {:msg _m} "Message"]
      [:wat::kernel::RecvOutcome.Lost {:cause c} (:wat::kernel::LociDiedError/message c)]
      [:wat::kernel::RecvOutcome.Stopped {} "Stopped"]
      [:wat::kernel::RecvOutcome.Closed {} "Closed"])))

;; (a2) T = Lru over `try-send` — the non-blocking twin's socket-tier arm.
(:wat::core::defn :g::probe-handle-try-send [] -> :wat::core::String
  (:wat::core::let
    [svc (:wat::test::spawn-peer (:wat::spawn::process)
           (:wat::core::forms
             (:wat::core::defrecord :g::Box :- [T] [x <- :T])
             (:wat::core::defn :g::ship :- [T] [x <- :T] -> :wat::core::String
               (:wat::core::match
                 (:wat::kernel::try-send (:wat::program::self-peer (:g::Box :- [:T]) :wat::core::i64)
                                         (:g::Box :x x))
                 [:wat::kernel::TrySendOutcome.Sent {} "Sent"]
                 [:wat::kernel::TrySendOutcome.WouldBlock {} "WouldBlock"]
                 [:wat::kernel::TrySendOutcome.Closed {} "Closed"]
                 [:wat::kernel::TrySendOutcome.Lost {:cause c}
                   (:wat::core::format "Lost: {m}" :m (:wat::kernel::LociDiedError/message c))]))
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:wat::core::let
                 [h       (:wat::core::Result/expect
                            (:wat::cache::Lru/new :- [:wat::core::i64 :wat::core::i64] 2) "lru")
                  _       (:g::ship h)]
                 nil))))]
    (:wat::core::match (:wat::kernel::recv svc)
      [:wat::kernel::RecvOutcome.Message {:msg _m} "Message"]
      [:wat::kernel::RecvOutcome.Lost {:cause c} (:wat::kernel::LociDiedError/message c)]
      [:wat::kernel::RecvOutcome.Stopped {} "Stopped"]
      [:wat::kernel::RecvOutcome.Closed {} "Closed"])))

;; (a3) The PARENT's side: a generic fn sends a Box holding a handle down a process peer
;; (`:wat::kernel::Process`, the lineage handle — the third wire arm).
(:wat::core::defn :g::push :- [T] [p <- (:wat::kernel::Process :- [(:g::Box :- [:T]) :wat::core::i64]) x <- :T]
  -> :wat::core::String
  (:wat::core::match (:wat::kernel::send p (:g::Box :x x))
    [:wat::kernel::SendOutcome.Sent {} "Sent"]
    [:wat::kernel::SendOutcome.Closed {} "Closed"]
    [:wat::kernel::SendOutcome.Lost {:cause c}
      (:wat::core::format "Lost: {m}" :m (:wat::kernel::LociDiedError/message c))]
    [:wat::kernel::SendOutcome.Stopped {} "Stopped"]))

(:wat::core::defn :g::probe-parent-send-handle [] -> :wat::core::String
  (:wat::core::let
    [svc (:wat::test::spawn-peer (:wat::spawn::process)
           (:wat::core::forms
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:wat::core::let [_ (:wat::core::match (:wat::kernel::readln ) [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum] [:wat::kernel::ReadlnOutcome.Eof {} (:wat::kernel::assertion-failed! :message "readln: end of input")] [:wat::kernel::ReadlnOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "readln: stop requested")])] nil))))
     h   (:wat::core::Result/expect (:wat::cache::Lru/new :- [:wat::core::i64 :wat::core::i64] 2) "lru")]
    (:g::push svc h)))

;; (b) T = i64 — the wire works; the parent renders what it received.
(:wat::core::defn :g::render-box [b <- (:g::Box :- [:wat::core::i64])] -> :wat::core::String
  (:wat::edn::write b))

(:wat::core::defn :g::probe-pure [] -> :wat::core::String
  (:wat::core::let
    [svc (:wat::test::spawn-peer (:wat::spawn::process)
           (:wat::core::forms
             (:wat::core::defrecord :g::Box :- [T] [x <- :T])
             (:wat::core::defn :g::ship :- [T] [x <- :T] -> :wat::core::nil
               (:wat::core::match
                 (:wat::kernel::send (:wat::program::self-peer (:g::Box :- [:T]) :wat::core::i64)
                                     (:g::Box :x x))
                 [:wat::kernel::SendOutcome.Sent {} nil]
                 [:wat::kernel::SendOutcome.Closed {} nil]
                 [:wat::kernel::SendOutcome.Lost {:cause _c} nil]
                 [:wat::kernel::SendOutcome.Stopped {} nil])) ;; arc 278 #73 — the parent's recv says whether it arrived
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:g::ship 42))))]
    (:wat::core::match (:wat::kernel::recv svc)
      [:wat::kernel::RecvOutcome.Message {:msg m}
        (:g::render-box m)]
      [:wat::kernel::RecvOutcome.Lost {:cause c}
        (:wat::core::format "Lost: {m}" :m (:wat::kernel::LociDiedError/message c))]
      [:wat::kernel::RecvOutcome.Stopped {} "Stopped"]
      [:wat::kernel::RecvOutcome.Closed {} "Closed"])))

;; (c) A Wire `Address` — an opaque that DOES cross (a registered capability, encoded by
;; `encode_capability`). The positive control for the strict encode: it must not be refused.
(:wat::core::defn :g::probe-address [] -> :wat::core::String
  (:wat::core::let
    [svc (:wat::test::spawn-peer (:wat::spawn::process)
           (:wat::core::forms
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:wat::core::let
                 [b    (:wat::kernel::listener (:wat::spawn::process) :wat::core::i64 :wat::core::i64)
                  self (:wat::program::self-peer
                         (:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64]) :wat::core::i64)
                  _    (:wat::core::match (:wat::kernel::send self (:wat::spawn::Bound/address b))
                         [:wat::kernel::SendOutcome.Sent {} nil]
                         [:wat::kernel::SendOutcome.Closed {} nil]
                         [:wat::kernel::SendOutcome.Lost {:cause _c} nil]
                         [:wat::kernel::SendOutcome.Stopped {} nil])] ;; arc 278 #73 — the parent's recv says whether it arrived
                 nil))))]
    (:wat::core::match (:wat::kernel::recv svc)
      [:wat::kernel::RecvOutcome.Message {:msg m} (:g::address-crossed m)]
      [:wat::kernel::RecvOutcome.Lost {:cause c}
        (:wat::core::format "Lost: {m}" :m (:wat::kernel::LociDiedError/message c))]
      [:wat::kernel::RecvOutcome.Stopped {} "Stopped"]
      [:wat::kernel::RecvOutcome.Closed {} "Closed"])))

(:wat::core::defn :g::address-crossed
  [a <- (:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::String
  "Message: an Address")

;; (d) Outside any wire, `:wat::edn::write` of a handle still renders the per-type-home tag with a
;; nil body — arc 294, unchanged by this stone.
(:wat::core::defn :g::probe-edn-write-handle [] -> :wat::core::String
  (:wat::edn::write
    (:g::Box :x (:wat::core::Result/expect
                  (:wat::cache::Lru/new :- [:wat::core::i64 :wat::core::i64] 2) "lru"))))
