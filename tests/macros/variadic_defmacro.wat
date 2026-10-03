;; tests/macros/variadic_defmacro.wat — co-located fixture for variadic_defmacro.rs,
;; slurped via startup_beside(file!()) for the three positive tests.
;;
;; Three named compute functions (renamed from :my::compute to unique names).

;; Test 1: vec-of macro — splice rest into a Vector.
(wat.core/defmacro my/vec-of
  [& items :- (wat.type/Vector :- [wat.type/AST])]
  :- wat.type/AST
  `(wat.type/Vector :- [wat.type/i64] ~@items))

(wat.core/defn my/compute-splice [] :- wat.type/i64
  (wat.core/first (my/vec-of 10 20 30)))

;; Test 2: empty-vec macro — zero rest-args.
(wat.core/defmacro my/empty-vec
  [& items :- (wat.type/Vector :- [wat.type/AST])]
  :- wat.type/AST
  `(wat.type/Vector :- [wat.type/i64] ~@items))

(wat.core/defn my/compute-empty [] :- (wat.type/Vector :- [wat.type/i64])
  (my/empty-vec))

;; Test 3: sum-of macro — fixed params + rest.
(wat.core/defmacro my/sum-of
  [init :- wat.type/AST
   & items :- (wat.type/Vector :- [wat.type/AST])]
  :- wat.type/AST
  `(wat.core/foldl
      (wat.core/fn [acc :- wat.type/i64 x :- wat.type/i64] :- wat.type/i64
        (wat.i64/+ acc x))
      ~init
      (wat.type/Vector :- [wat.type/i64] ~@items)))

(wat.core/defn my/compute-sum [] :- wat.type/i64
  (my/sum-of 100 1 2 3))

