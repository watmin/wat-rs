;; Co-located fixture for wat_stat.rs — slurped via startup_beside(file!()).

(wat.core/defn my/compute-mean-known [] :- wat.type/String
  (wat.core/let
    [xs (wat.type/Vector :- [wat.type/f64] 1.0 2.0 3.0 4.0 5.0)
     m  (wat.stat/mean xs)
     v  (wat.core/match m 
           [wat.core/Option.Some {:value x} x] [wat.core/Option.None {} -1.0])]
    (wat.f64/to-string v)))

(wat.core/defn my/compute-mean-empty [] :- wat.type/String
  (wat.core/let
    [xs    (wat.type/Vector :- [wat.type/f64])
     m     (wat.stat/mean xs)
     label (wat.core/match m 
              [wat.core/Option.Some {:value _} "some"] [wat.core/Option.None {} "none"])]
    label))

(wat.core/defn my/compute-variance-known [] :- wat.type/String
  (wat.core/let
    [xs (wat.type/Vector :- [wat.type/f64] 1.0 2.0 3.0 4.0 5.0)
     v  (wat.core/match (wat.stat/variance xs) 
           [wat.core/Option.Some {:value x} x] [wat.core/Option.None {} -1.0])]
    (wat.f64/to-string v)))

(wat.core/defn my/compute-variance-single [] :- wat.type/String
  (wat.core/let
    [xs (wat.type/Vector :- [wat.type/f64] 7.0)
     v  (wat.core/match (wat.stat/variance xs) 
           [wat.core/Option.Some {:value x} x] [wat.core/Option.None {} -1.0])]
    (wat.f64/to-string v)))

(wat.core/defn my/compute-stddev-known [] :- wat.type/String
  (wat.core/let
    [xs (wat.type/Vector :- [wat.type/f64] 1.0 2.0 3.0 4.0 5.0)
     sd (wat.core/match (wat.stat/stddev xs) 
           [wat.core/Option.Some {:value x} x] [wat.core/Option.None {} -1.0])]
    (wat.core/if (wat.core/> sd 1.41)  "ok" "bad")))

