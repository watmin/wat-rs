;; ⛔ THE SAME GAP VIA `match` — 277 of the 600 came through match arms, not if.
(wat.core/defenum usr/Sig wat.enum/Pure
  :Up   [n :- wat.type/i64]
  :Down [n :- wat.type/i64]
  :Flat [])
(wat.core/defn user/route [s :- usr/Sig] :- usr/Sig
  (wat.core/match s
    [usr/Sig.Up   {:n n} (usr/Sig.Down {:n n})]
    [usr/Sig.Down {:n n} (usr/Sig.Up   {:n n})]
    [usr/Sig.Flat {}     (usr/Sig.Flat {})]))
(wat.core/defn user/main [] :- wat.type/nil (wat.kernel/println "ok"))
