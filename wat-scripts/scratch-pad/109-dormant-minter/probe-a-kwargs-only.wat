;; BRIEF-STONE-the-dormant-minter.md — control A: kwargs, NO binder. Expect 3.
;; Ships alongside probe-b (binder, no kwargs) and probe-c (both — the survivor's
;; reach) as the pair that proves a fix to the join, not a fix that disabled a half.
(wat.core/defn dm109a/hold
  [seed :- wat.type/i64 & [times :- wat.type/i64]] :- wat.type/i64
  times)

(wat.core/defn user/main [] :- wat.type/nil
  (wat.kernel/println
    (wat.string/interpolate "A={a}"
      :a (wat.i64/to-string (dm109a/hold 1 :times 3)))))
