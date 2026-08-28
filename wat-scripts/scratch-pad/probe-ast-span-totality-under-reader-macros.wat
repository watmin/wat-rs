;; PROBE — is `ast-span` TOTAL for nodes reached through `ast->children`, INCLUDING the ones the
;; READER SYNTHESIZES?
;;
;; The Span-fact stone rests on "ast-span and ast-end-span are BOTH TOTAL", measured across LEAF
;; KINDS. That samples SHAPES. The mechanism that could break it is different: a reader macro
;; sigil (`'` `~` `~@` `` ` ``) expands into a LIST the reader builds itself. A synthesized node
;; may carry no source location at all — and corpus-03's own header flags exactly this family as
;; "the reader-macro sigils that CORRUPTED the text-edit engine".
;;
;; If ast-span raises on a synthesized node, `Span == Node` is unreachable and the stone's guard
;; design inverts back to Named's. So it is measured here, before the stone is briefed.
;;
;; A raise anywhere = the probe FAILS LOUD. A clean run printing Node==Span is the answer.

(:wat::core::defrecord :probe::Acc
  [nodes <- :wat::core::i64
   spans <- :wat::core::i64])

(:wat::core::defn :probe::structural? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::let [k (:wat::core::ast-kind node)]
    (:wat::hashset::contains?
      (:wat::core::HashSet :wat::type::Infer "list" "vector" "map" "set") k)))

;; walk — call ast-span AND ast-end-span on EVERY node. Both are unguarded on purpose: this probe
;; exists to find the node that raises, not to survive it.
(:wat::core::defn :probe::walk [acc <- :probe::Acc  node <- :wat::WatAST] -> :probe::Acc
  (:wat::core::let
    [s    (:wat::core::ast-span node)
     e    (:wat::core::ast-end-span node)
     _l   (:wat::core::Option/expect (:wat::hashmap::get s :line) "start :line")
     _c   (:wat::core::Option/expect (:wat::hashmap::get e :col)  "end :col")
     acc' (:probe::Acc :nodes (:wat::i64::+ (:probe::Acc/nodes acc) 1)
                       :spans (:wat::i64::+ (:probe::Acc/spans acc) 1))]
    (:wat::core::if (:probe::structural? node)
      (:wat::core::foldl
        (:wat::core::fn [a <- :probe::Acc  child <- :wat::WatAST] -> :probe::Acc
          (:probe::walk a child))
        acc'
        (:wat::core::ast->children node))
      acc')))

(:wat::core::defn :probe::run [label <- :wat::core::String  src <- :wat::core::String] -> :wat::core::nil
  (:wat::core::match (:wat::core::read-string src)
    ((:wat::core::ReadOutcome::Forms forms)
      (:wat::core::let
        [acc (:wat::core::foldl
               (:wat::core::fn [a <- :probe::Acc  form <- :wat::WatAST] -> :probe::Acc
                 (:probe::walk a form))
               (:probe::Acc :nodes 0 :spans 0)
               (:wat::core::ast->children forms))]
        (:wat::kernel::println
          (:wat::string::concat label
            (:wat::string::concat "  Node=" (:wat::core::str (:probe::Acc/nodes acc))
              (:wat::string::concat "  Span=" (:wat::core::str (:probe::Acc/spans acc))))))))
    ((:wat::core::ReadOutcome::Malformed cause)
      (:wat::kernel::println (:wat::string::concat label (:wat::string::concat "  MALFORMED " (:wat::core::str cause)))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    ;; 1 — every reader sigil, inline. THE mechanism under test.
    (:probe::run "sigils-inline" "(a 'b `c ~d ~@e #{1 2} {:k 1} [1 2])")
    ;; 2 — the file corpus-03 itself names as the sigil-bearing hazard
    (:probe::run "probe_do_splice" (:wat::io::read-file "tests/macros/probe_do_splice_define_via_macro.wat"))
    ;; 3 — the codemod, the biggest real file the stone quotes (Node=4316)
    (:probe::run "wat/fix.wat" (:wat::io::read-file "wat/fix.wat"))))
