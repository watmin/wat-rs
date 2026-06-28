;; Co-located fixture for wat_dispatch_e5_owned_move.rs — slurped via startup_beside(file!()).
;; compute-double-redeem errors at eval time (second redeem on already-consumed cell).

(:wat::core::use! :rust::test::Ticket)

(:wat::core::defn :my::compute-redeem [] -> :wat::core::i64
  (:wat::core::let
    [t (:rust::test::Ticket::new 777)]
    (:rust::test::Ticket::redeem t)))

(:wat::core::defn :my::compute-double-redeem [] -> :wat::core::i64
  (:wat::core::let
    [t     (:rust::test::Ticket::new 42)
     first (:rust::test::Ticket::redeem t)]
    (:rust::test::Ticket::redeem t)))

