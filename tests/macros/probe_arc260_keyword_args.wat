;; tests/macros/probe_arc260_keyword_args.wat — co-located fixture for
;; probe_arc260_keyword_args.rs, slurped via startup_beside(file!()).
;;
;; Arc 260 RED — wat has NO keyword args; call sites are positional.
;; A user fn called with OUT-OF-ORDER keyword args; only a real kwargs feature
;; (reorder by param name) yields the right answer.
(wat.core/defn user/sub [a :- wat.type/i64  b :- wat.type/i64] :- wat.type/i64
  (wat.i64/- a b))

(wat.core/defn user/compute [] :- wat.type/i64
  (user/sub :b 3 :a 10))

