;; A trivial serve loop — proves the launch WIRING (not the poll loop, which 4a-iii covers).
(:wat::core::defn :my::svc::serve
  [self    <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>
   l       <- :wat::kernel::Listener'<wat::core::i64,wat::core::i64>
   clients <- :wat::core::Vector<wat::kernel::Peer'<wat::core::i64,wat::core::i64>>
   st      <- :wat::core::i64] -> :wat::core::nil
  nil)

;; init fn: takes ship (i64) and returns St (i64) — identity for this wiring probe.
;; arc 291: launch now takes [self ship init serve service-forms lu-addr-kw] (6 args).
(:wat::core::defn :my::svc::init [ship <- :wat::core::i64] -> :wat::core::i64 ship)

;; Locus-AGNOSTIC: the param is the abstract `:wat::spawn::Locus`. `Locus/launch` routes through it.
;; arc 291: launch signature = [self ship init serve service-forms lu-addr-kw] (6 args).
;; Launched now has 4 type params <S,R,Sh,Lu>; here all are i64.
(:wat::core::defn :user::start-it [h <- :wat::spawn::Locus] -> :wat::spawn::Launched<wat::core::i64,wat::core::i64,wat::core::i64,wat::core::i64>
  (:wat::spawn::Locus/launch h 0
    (:wat::core::keyword/from-string "my::svc::init")
    (:wat::core::keyword/from-string "my::svc::serve")
    (:wat::core::forms)
    (:wat::core::keyword/from-string "my::svc::init")))

;; Drive it with a concrete (thread): reaching `true` means the whole locus-agnostic launch wired and
;; ran without crashing (listener' accepted :Locus, launch dispatched, apply invoked serve, peer spawned).
(:wat::core::defn :user::go [] -> :wat::core::bool
  (:wat::core::let [h (:user::start-it (:wat::spawn::thread))]
    true))
