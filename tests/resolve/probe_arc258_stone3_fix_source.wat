(wat.core/defn user/topform [src :- wat.type/String] :- wat.type/AST
  (wat.core/first (wat.core/ast->children (wat.core/match (wat.core/read-string src) [wat.core/ReadOutcome.Forms {:forms __forms} __forms] [wat.core/ReadOutcome.Malformed {:cause __cause} (wat.kernel/assertion-failed! :message (wat.core.Error/message __cause))]))))

(wat.core/defn user/structural? [node :- wat.type/AST] :- wat.type/bool
  (wat.core/let [k (wat.core/ast-kind node)]
    (wat.core/if (wat.core/= k "list") true
      (wat.core/if (wat.core/= k "vector") true
        (wat.core/if (wat.core/= k "map") true
          (wat.core/if (wat.core/= k "set") true false))))))

(wat.core/defn user/annotated-if? [node :- wat.type/AST] :- wat.type/bool
  (wat.core/if (wat.core/= (wat.core/ast-kind node) "list")
    (wat.core/let [ch (wat.core/ast->children node)]
      ;; Stone 118.B4-iii — THE WALL: was `(empty? (drop ch 2))`. `drop` returns a lazy Stream
      ;; (arc 118.2a); `empty?` no longer accepts one. `ch` is a Vector (eager) so `length`
      ;; answers the identical question — "does ch have fewer than 3 elements" — in O(1).
      (wat.core/if (wat.core/< (wat.core/length ch) 3)
        false
        (wat.core/let [head (wat.core/first ch)
                          c2   (wat.core/nth ch 2)]
          (wat.core/if (wat.core/= (wat.core/ast-name head) ":wat::core::if")
            (wat.core/if (wat.core/= (wat.core/ast-kind c2) "symbol")
              (wat.core/= (wat.core/ast-name c2) "->")
              false)
            false))))
    false))

;; Arc 118.2a — `take`/`drop` flipped LAZY (return Stream); `concat` (unchanged) needs both
;; sides eager.
(wat.core/defn user/strip-if [node :- wat.type/AST] :- wat.type/AST
  (wat.core/with-children node
    (wat.core/concat (wat.core/into [] (wat.core/take (wat.core/ast->children node) 2))
                        (wat.core/into [] (wat.core/drop (wat.core/ast->children node) 4)))))

;; Arc 118.2a — `map` flipped LAZY; `with-children` needs a concrete `(Vector :- [WatAST])`.
(wat.core/defn user/fix-source [node :- wat.type/AST] :- wat.type/AST
  (wat.core/if (user/structural? node)
    (wat.core/let [rebuilt (wat.core/with-children node
                                (wat.core/mapv
                                  (wat.core/fn [c :- wat.type/AST] :- wat.type/AST (user/fix-source c))
                                  (wat.core/ast->children node)))]
      (wat.core/if (user/annotated-if? rebuilt)
        (user/strip-if rebuilt)
        rebuilt))
    node))

(wat.core/defn user/c01 [] :- wat.type/bool
  (user/annotated-if? (user/topform "(:wat::core::if true -> :wat::core::i64 1 2)")))
(wat.core/defn user/c02 [] :- wat.type/bool
  (user/annotated-if? (user/topform "(:wat::core::if true 1 2)")))
(wat.core/defn user/c03 [] :- wat.type/bool
  (user/annotated-if? (user/topform "(:wat::core::Option/expect -> :wat::core::i64 x \"m\")")))
(wat.core/defn user/c04a [] :- wat.type/bool
  (user/annotated-if? (user/fix-source (user/topform "(:wat::core::if true -> :wat::core::i64 1 2)"))))
(wat.core/defn user/c04b [] :- wat.type/bool
  (wat.core/= (wat.core/ast-kind
    (wat.core/nth
      (wat.core/ast->children (user/fix-source (user/topform "(:wat::core::if true -> :wat::core::i64 1 2)")))
      2))
    "int"))
(wat.core/defn user/c05 [] :- wat.type/String
  (wat.core/write-forms (user/fix-source (user/topform "(:wat::core::do (:wat::core::if true -> :wat::core::i64 1 2))"))))
(wat.core/defn user/c06 [] :- wat.type/String
  (wat.core/write-forms (user/fix-source (user/topform "(:wat::core::do (:wat::core::Option/expect -> :wat::core::i64 x \"m\"))"))))
(wat.core/defn user/c07-str [] :- wat.type/String
  (wat.core/write-forms (user/fix-source (user/topform "(:wat::core::if true -> :wat::core::i64 1 2)"))))
(wat.core/defn user/c07-bool [] :- wat.type/bool
  (wat.core/= (wat.core/ast-name
    (wat.core/first (wat.core/ast->children
      (user/fix-source (user/topform "(:wat::core::if true -> :wat::core::i64 1 2)")))))
    ":wat::core::if"))
(wat.core/defn user/qq [a :- wat.type/AST b :- wat.type/AST c :- wat.type/AST] :- wat.type/AST
  `(wat.core/if ~a ~b ~c))
(wat.core/defn user/compute-c08 [] :- wat.type/bool
  (wat.core/List? (user/qq
    (user/topform "true")
    (user/topform "1")
    (user/topform "2"))))
