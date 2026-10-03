;; tests/wat_lang/wat_arc143_lookup.wat — co-located fixture.
;; Arc 143 slice 1 — three substrate introspection primitives:
;;   :wat::runtime::lookup-define
;;   :wat::runtime::signature-of-defn
;;   :wat::runtime::body-of
;; All functions return bool (Some→true / None→false) or String (rendered EDN).

;; Helper functions needed by the introspection tests.
(wat.core/defn t/my-add [x :- wat.type/i64 y :- wat.type/i64] :- wat.type/i64
  (wat.core/+ x y))
(wat.core/defn t/my-mul [a :- wat.type/i64 b :- wat.type/i64] :- wat.type/i64
  (wat.core/* a b))
(wat.core/defn t/my-neg [n :- wat.type/i64] :- wat.type/i64
  (wat.core/- 0 n))
(wat.core/defn t/my-square [x :- wat.type/i64] :- wat.type/i64
  (wat.core/* x x))

;; ─── :wat::runtime::lookup-define ──────────────────────────────────────────

;; test1: user-define returns Some
(wat.core/defn t/test1-lookup-user [] :- wat.type/bool
  (wat.core/match
    (wat.runtime/lookup-define t/my-add)
    
    [wat.core/Option.Some {:value _} true]
    [wat.core/Option.None {}    false]))

;; test2: substrate primitive :wat::core::foldl returns Some
(wat.core/defn t/test2-lookup-foldl [] :- wat.type/bool
  (wat.core/match
    (wat.runtime/lookup-define wat.core/foldl)
    
    [wat.core/Option.Some {:value _} true]
    [wat.core/Option.None {}    false]))

;; test3: unknown name returns None
(wat.core/defn t/test3-lookup-none [] :- wat.type/bool
  (wat.core/match
    (wat.runtime/lookup-define user/this-does-not-exist)
    
    [wat.core/Option.Some {:value _} false]
    [wat.core/Option.None {}    true]))

;; ─── :wat::runtime::signature-of-defn ──────────────────────────────────────

;; test4: user-defined function → Some
(wat.core/defn t/test4-sig-user [] :- wat.type/bool
  (wat.core/match
    (wat.runtime/signature-of-defn t/my-mul)
    
    [wat.core/Option.Some {:value _} true]
    [wat.core/Option.None {}    false]))

;; test5: substrate primitive :wat::core::foldl → Some
(wat.core/defn t/test5-sig-foldl [] :- wat.type/bool
  (wat.core/match
    (wat.runtime/signature-of-defn wat.core/foldl)
    
    [wat.core/Option.Some {:value _} true]
    [wat.core/Option.None {}    false]))

;; test6: unknown name → None
(wat.core/defn t/test6-sig-none [] :- wat.type/bool
  (wat.core/match
    (wat.runtime/signature-of-defn no.such/function)
    
    [wat.core/Option.Some {:value _} false]
    [wat.core/Option.None {}    true]))

;; ─── :wat::runtime::body-of ─────────────────────────────────────────────────

;; test7: user-defined function → Some
(wat.core/defn t/test7-body-user [] :- wat.type/bool
  (wat.core/match
    (wat.runtime/body-of t/my-neg)
    
    [wat.core/Option.Some {:value _} true]
    [wat.core/Option.None {}    false]))

;; test8: substrate primitive :wat::core::foldl → None (no wat body)
(wat.core/defn t/test8-body-prim-none [] :- wat.type/bool
  (wat.core/match
    (wat.runtime/body-of wat.core/foldl)
    
    [wat.core/Option.Some {:value _} false]
    [wat.core/Option.None {}    true]))

;; test9: unknown name → None
(wat.core/defn t/test9-body-unknown-none [] :- wat.type/bool
  (wat.core/match
    (wat.runtime/body-of totally/unknown)
    
    [wat.core/Option.Some {:value _} false]
    [wat.core/Option.None {}    true]))

;; ─── Rendered EDN shape verification ────────────────────────────────────────

;; test10: signature-of-defn for :wat::core::foldl renders synthesised shape
(wat.core/defn t/test10-sig-render [] :- wat.type/String
  (wat.core/let [sig-opt (wat.runtime/signature-of-defn wat.core/foldl)
                   rendered (wat.edn/write sig-opt)]
    rendered))

;; test11: lookup-define for a user function contains "defn" and function name
(wat.core/defn t/test11-def-render [] :- wat.type/String
  (wat.core/let [def-opt  (wat.runtime/lookup-define t/my-square)
                   rendered (wat.edn/write def-opt)]
    rendered))
