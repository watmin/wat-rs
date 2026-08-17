;; wat-scripts/perf/grid/where-or-conditions.wat — condition `:or` (or of activations).
;; Twin of where-or-conditions.clj. Clara `[:or [Temp] [Wind]]` → one Hit per loc.
;; Both sides print unique-loc n= (wat's value session; Clara counted unique).
;;
;; Rows 1–3: trailing `:or` (the last-entry case we already had).
;; Rows 4–7: prefix fact then `:or` (Clara does not require `:or` last).
;; Rows 8–11: `:or` then a later fact.
;; Rows 12–14: prefix + `:or` + trailing `:where` (Test hangs off N arm terminals).

(:wat::core::defrecord :wor::Temp    [c <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wor::Wind    [kph <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wor::Station [loc <- :wat::core::String])
(:wat::core::defrecord :wor::Reading [loc <- :wat::core::String v <- :wat::core::i64])
(:wat::core::defrecord :wor::Hit     [loc <- :wat::core::String])

;; ROW 1–3 — `:or` is the only :when entry.
(:wat::rete::defrule :wor::or-hit
  :when [(:wat::rete::or
           (:wor::Temp (?loc <- :loc) (?c <- :c)
             (:wat::rete::core::i64::< ?c 20))
           (:wor::Wind (?loc <- :loc) (?w <- :kph)
             (:wat::rete::core::i64::> ?w 30)))]
  :then [(:wor::Hit :loc ?loc)])

;; ROW 4–7 — Station, then `:or`. Cold-only / wind-only / both / no-station.
(:wat::rete::defrule :wor::prefix-then-or
  :when [(:wor::Station (?loc <- :loc))
         (:wat::rete::or
           (:wor::Temp (?loc <- :loc) (?c <- :c)
             (:wat::rete::core::i64::< ?c 20))
           (:wor::Wind (?loc <- :loc) (?w <- :kph)
             (:wat::rete::core::i64::> ?w 30)))]
  :then [(:wor::Hit :loc ?loc)])

;; ROW 8–11 — `:or`, then Station. Same bag as prefix-then-or.
(:wat::rete::defrule :wor::or-then-fact
  :when [(:wat::rete::or
           (:wor::Temp (?loc <- :loc) (?c <- :c)
             (:wat::rete::core::i64::< ?c 20))
           (:wor::Wind (?loc <- :loc) (?w <- :kph)
             (:wat::rete::core::i64::> ?w 30)))
         (:wor::Station (?loc <- :loc))]
  :then [(:wor::Hit :loc ?loc)])

;; ROW 12–14 — Reading, `:or`, then `:where` on the prefix-bound ?v.
(:wat::rete::defrule :wor::or-then-where
  :when [(:wor::Reading (?loc <- :loc) (?v <- :v))
         (:wat::rete::or
           (:wor::Temp (?loc <- :loc))
           (:wor::Wind (?loc <- :loc)))
         (:wat::rete::where (:wat::rete::core::i64::> ?v 10))]
  :then [(:wor::Hit :loc ?loc)])

(:wat::rete::defquery :wor::q-Hit
  :params []
  :when [(?fact <- :wor::Hit)])


(:wat::core::defn :wor::n-hit [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s (:wor::q-Hit))))

(:wat::core::defn :wor::line [row <- :wat::core::i64 name <- :wat::core::String n <- :wat::core::i64] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::String/concat
      (:wat::core::String/concat "row " (:wat::core::i64::to-string row))
      (:wat::core::String/concat
        (:wat::core::String/concat " " name)
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string n))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  ;; 1–3 trailing :or
  (:wor::line 1 "or-cold-only"
    (:wor::n-hit
      (:wat::rete::fire-rules
        (:wat::rete::insert (:wat::rete::compile-all (:wat::core::PersistentVector (:wor::or-hit)) (:wat::core::PersistentVector (:wor::q-Hit)))
          (:wor::Temp :c 15 :loc "MCI")))))
  (:wor::line 2 "or-wind-only"
    (:wor::n-hit
      (:wat::rete::fire-rules
        (:wat::rete::insert (:wat::rete::compile-all (:wat::core::PersistentVector (:wor::or-hit)) (:wat::core::PersistentVector (:wor::q-Hit)))
          (:wor::Wind :kph 50 :loc "MCI")))))
  (:wor::line 3 "or-both"
    (:wor::n-hit
      (:wat::rete::fire-rules
        (:wat::rete::insert (:wat::rete::compile-all (:wat::core::PersistentVector (:wor::or-hit)) (:wat::core::PersistentVector (:wor::q-Hit)))
          (:wor::Temp :c 15 :loc "MCI")
          (:wor::Wind :kph 50 :loc "MCI")))))

  ;; 4–7 prefix then :or
  (:wor::line 4 "prefix-or-cold"
    (:wor::n-hit
      (:wat::rete::fire-rules
        (:wat::rete::insert (:wat::rete::compile-all (:wat::core::PersistentVector (:wor::prefix-then-or)) (:wat::core::PersistentVector (:wor::q-Hit)))
          (:wor::Station :loc "MCI")
          (:wor::Temp :c 15 :loc "MCI")))))
  (:wor::line 5 "prefix-or-wind"
    (:wor::n-hit
      (:wat::rete::fire-rules
        (:wat::rete::insert (:wat::rete::compile-all (:wat::core::PersistentVector (:wor::prefix-then-or)) (:wat::core::PersistentVector (:wor::q-Hit)))
          (:wor::Station :loc "MCI")
          (:wor::Wind :kph 50 :loc "MCI")))))
  (:wor::line 6 "prefix-or-both"
    (:wor::n-hit
      (:wat::rete::fire-rules
        (:wat::rete::insert (:wat::rete::compile-all (:wat::core::PersistentVector (:wor::prefix-then-or)) (:wat::core::PersistentVector (:wor::q-Hit)))
          (:wor::Station :loc "MCI")
          (:wor::Temp :c 15 :loc "MCI")
          (:wor::Wind :kph 50 :loc "MCI")))))
  (:wor::line 7 "prefix-or-no-station"
    (:wor::n-hit
      (:wat::rete::fire-rules
        (:wat::rete::insert (:wat::rete::compile-all (:wat::core::PersistentVector (:wor::prefix-then-or)) (:wat::core::PersistentVector (:wor::q-Hit)))
          (:wor::Temp :c 15 :loc "MCI")))))

  ;; 8–11 :or then fact
  (:wor::line 8 "or-fact-cold"
    (:wor::n-hit
      (:wat::rete::fire-rules
        (:wat::rete::insert (:wat::rete::compile-all (:wat::core::PersistentVector (:wor::or-then-fact)) (:wat::core::PersistentVector (:wor::q-Hit)))
          (:wor::Station :loc "MCI")
          (:wor::Temp :c 15 :loc "MCI")))))
  (:wor::line 9 "or-fact-wind"
    (:wor::n-hit
      (:wat::rete::fire-rules
        (:wat::rete::insert (:wat::rete::compile-all (:wat::core::PersistentVector (:wor::or-then-fact)) (:wat::core::PersistentVector (:wor::q-Hit)))
          (:wor::Station :loc "MCI")
          (:wor::Wind :kph 50 :loc "MCI")))))
  (:wor::line 10 "or-fact-both"
    (:wor::n-hit
      (:wat::rete::fire-rules
        (:wat::rete::insert (:wat::rete::compile-all (:wat::core::PersistentVector (:wor::or-then-fact)) (:wat::core::PersistentVector (:wor::q-Hit)))
          (:wor::Station :loc "MCI")
          (:wor::Temp :c 15 :loc "MCI")
          (:wor::Wind :kph 50 :loc "MCI")))))
  (:wor::line 11 "or-fact-no-station"
    (:wor::n-hit
      (:wat::rete::fire-rules
        (:wat::rete::insert (:wat::rete::compile-all (:wat::core::PersistentVector (:wor::or-then-fact)) (:wat::core::PersistentVector (:wor::q-Hit)))
          (:wor::Temp :c 15 :loc "MCI")))))

  ;; 12–14 prefix + :or + :where
  (:wor::line 12 "or-where-pass"
    (:wor::n-hit
      (:wat::rete::fire-rules
        (:wat::rete::insert (:wat::rete::compile-all (:wat::core::PersistentVector (:wor::or-then-where)) (:wat::core::PersistentVector (:wor::q-Hit)))
          (:wor::Reading :loc "MCI" :v 15)
          (:wor::Temp :c 15 :loc "MCI")))))
  (:wor::line 13 "or-where-fail"
    (:wor::n-hit
      (:wat::rete::fire-rules
        (:wat::rete::insert (:wat::rete::compile-all (:wat::core::PersistentVector (:wor::or-then-where)) (:wat::core::PersistentVector (:wor::q-Hit)))
          (:wor::Reading :loc "MCI" :v 5)
          (:wor::Temp :c 15 :loc "MCI")))))
  (:wor::line 14 "or-where-both"
    (:wor::n-hit
      (:wat::rete::fire-rules
        (:wat::rete::insert (:wat::rete::compile-all (:wat::core::PersistentVector (:wor::or-then-where)) (:wat::core::PersistentVector (:wor::q-Hit)))
          (:wor::Reading :loc "MCI" :v 15)
          (:wor::Temp :c 15 :loc "MCI")
          (:wor::Wind :kph 50 :loc "MCI"))))))
