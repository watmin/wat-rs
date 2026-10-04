;; tests/diagnostics/probe_ex003_f2_gf2b_raised_and_returned_agree.wat — co-located fixture
;; for probe_ex003_f2_gf2b_raised_and_returned_agree.rs.
;;
;; Excursus 003 strike F2, GF2b: one location rule for raised AND returned errors alike.
;; `:raises` RAISES (an `i64` overflow inside `:wat::core::+`, D4's path) and `:returns`
;; RETURNS a fault (`:wat::cache::Lru/new` with capacity 0, error-site's path) — both calls
;; sit on the SAME source line below, so both derivations, run from the same user call site,
;; must report the SAME line. They are declared on one physical line deliberately: the point
;; is not merely that each locates at "a" user line, but that the TWO rules (D4 for raised,
;; error-site for returned) agree when driven from an identical call site.
(:wat::core::defn :my::test::raises [] -> :wat::core::i64 (:wat::core::+ 41 9223372036854775807)) (:wat::core::defn :my::test::returns [] -> :wat::core::i64 (:wat::core::match (:wat::cache::Lru/new :- [:wat::core::keyword :wat::core::i64] 0) [:wat::core::Result.Err {:error f} (:wat::core::Span/line (:wat::cache::Fault/location f))] [:wat::core::Result.Ok {:value _v} (:wat::kernel::assertion-failed! :message "Lru/new with capacity 0 must refuse")]))
