;; wat-scripts/fixes/fn-keyword-to-bracket.wat — arc 255, STONE 255.81.
;; SCOPE: corpus
;;
;; The keyword-bodied fn type dies. The only arrow is the bracket `[A :-> R]`
;; (stone 251.4c). This codemod rewrites zero or one argument:
;;
;;   :wat::core::Fn(A)->R   ->   [A' :-> R']
;;   :fn(A)->R              ->   [A' :-> R']
;;   :wat::core::Fn()->R    ->   [:-> R']          nullary body renders `[:-> R]`
;;   :fn()->R               ->   [:-> R']
;;
;; bare `:fn(` is the same bracket as `:wat::core::Fn(`. Argument and return
;; types themselves take `wat.type/` spelling when they are one of the 24.
;; `wat::WatAST` renders `wat.type/AST`. A name that is not one of the 24 stays a keyword (`wat::rete::Session` -> `:wat::rete::Session`). A bare type
;; variable stays bare.
;;
;; Two or more arguments (a top-level comma in the argument list) and a nested
;; keyword body (`Fn(` / `fn(` / a `(` inside the argument or the return) are
;; left unchanged. The lexer already refuses a comma inside a keyword body, so
;; a parseable program cannot carry two arguments. A read that comes back
;; Malformed (prose `:fn(...)`, a comma the extractor lifted) returns the
;; source unchanged.
;;
;; Idempotent: the replacement is a vector, not a keyword, so a second pass
;; finds nothing to edit.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["pathA" "pathB" …]\n' | ./target/release/wat ./wat-scripts/fixes/fn-keyword-to-bracket.wat
;;
;; Dry-run on a /tmp copy first and diff it.

(wat.core/defn fkb/hard-tails [] :- (wat.type/HashSet :- [wat.type/String])
  (wat.type/HashSet :- [wat.type/String]
    "i64" "f64" "u8" "bigint" "rational" "char" "String" "bool" "keyword" "nil"
    "Value" "Never" "Fn" "Record" "Struct" "Vector" "HashMap" "HashSet" "List"
    "Tuple" "PersistentVector" "PersistentMap" "Bytes" "AST"))

(wat.core/defn fkb/fn-keyword? [name :- wat.type/String] :- wat.type/bool
  (wat.core/and
    (wat.core/or
      (wat.string/starts-with? name ":fn(")
      (wat.string/starts-with? name ":wat::core::Fn("))
    (wat.string/contains? name ")->")
    (fkb/one-arg? name)))

;; one-arg? — zero or one argument, no nested keyword body. A comma, a
;; parenthesis, or a nested `Fn(` / `fn(` is not this migration (STOP-3
;; shapes stay untouched: the lexer already refuses a comma in a keyword
;; body, and a prose token like `:fn(...)` has no `)->`).
(wat.core/defn fkb/one-arg? [name :- wat.type/String] :- wat.type/bool
  (wat.core/let [parts (wat.string/split name ")->")]
    (wat.core/if (wat.core/= (wat.core/length parts) 2)
      (wat.core/let [head (wat.core/nth parts 0)
                        ret  (wat.core/nth parts 1)
                        args (wat.string/subs head (fkb/prefix-len name) (wat.string/length head))]
        (wat.core/not
          (wat.core/or
            (wat.string/contains? args ",")
            (wat.string/contains? args "(")
            (wat.string/contains? args "Fn(")
            (wat.string/contains? args "fn(")
            (wat.string/contains? ret ",")
            (wat.string/contains? ret "(")
            (wat.string/contains? ret "Fn(")
            (wat.string/contains? ret "fn("))))
      false)))

(wat.core/defn fkb/prefix-len [name :- wat.type/String] :- wat.type/i64
  (wat.core/if (wat.string/starts-with? name ":fn(") 4 15))

;; render-type — one inner type, already stripped of any leading colon
;; (compound keyword bodies forbid an inner colon).
(wat.core/defn fkb/render-type [s :- wat.type/String] :- wat.type/String
  (wat.core/if (wat.core/= s "wat::WatAST")
    "wat.type/AST"
    (wat.core/if (wat.string/starts-with? s "wat::core::")
      (wat.core/let [tail (wat.string/subs s 11 (wat.string/length s))]
        (wat.core/if (wat.core/contains? (fkb/hard-tails) tail)
          (wat.string/concat "wat.type/" tail)
          (wat.string/concat ":" s)))
      (wat.core/if (wat.string/contains? s "::")
        (wat.string/concat ":" s)
        s))))

(wat.core/defn fkb/render [name :- wat.type/String] :- wat.type/String
  (wat.core/let [parts (wat.string/split name ")->")]
    (wat.core/if (wat.core/= (wat.core/length parts) 2)
      (wat.core/let [head (wat.core/nth parts 0)
                        ret  (wat.core/nth parts 1)
                        args (wat.string/subs head (fkb/prefix-len name) (wat.string/length head))]
        (wat.core/if (wat.core/= args "")
          (wat.string/concat "[:-> " (wat.string/concat (fkb/render-type ret) "]"))
          (wat.string/concat "["
            (wat.string/concat (fkb/render-type args)
              (wat.string/concat " :-> "
                (wat.string/concat (fkb/render-type ret) "]"))))))
      name)))

(wat.core/defn fkb/edit
  [node  :- wat.type/AST
   lines :- (wat.type/Vector :- [wat.type/String])]
  :- (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])])
  (wat.core/let [off      (wat.fix/fix-text-offset-of (wat.core/ast-span node) lines)
                    old-name (wat.core/ast-name node)
                    new-text (fkb/render old-name)]
    (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])]
      (wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String] off old-name new-text))))

(wat.core/defn fkb/walk-children
  [items :- (wat.type/Vector :- [wat.type/AST])
   lines :- (wat.type/Vector :- [wat.type/String])
   src   :- wat.type/String]
  :- (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])])
  (wat.core/if (wat.core/empty? items)
    (wat.fix/empty-edits)
    (wat.core/concat
      (fkb/walk (wat.core/first items) lines src)
      (fkb/walk-children (wat.core/rest items) lines src))))

(wat.core/defn fkb/walk
  [node  :- wat.type/AST
   lines :- (wat.type/Vector :- [wat.type/String])
   src   :- wat.type/String]
  :- (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])])
  (wat.core/if (wat.fix/structural? node)
    (fkb/walk-children (wat.core/ast->children node) lines src)
    (wat.core/if (wat.core/and
                      (wat.core/= (wat.core/ast-kind node) "keyword")
                      (fkb/fn-keyword? (wat.core/ast-name node))
                      (wat.fix/source-matches-name? node lines src))
      (fkb/edit node lines)
      (wat.fix/empty-edits))))

(wat.core/defn fkb/convert [src :- wat.type/String] :- wat.type/String
  (wat.core/match (wat.core/read-string src)
    ;; An embedded snippet that does not lex (prose `:fn(...)`, a comma inside
    ;; a keyword, a character the extractor lifted) is not rewritten.
    [wat.core/ReadOutcome.Malformed {:cause __cause} src]
    [wat.core/ReadOutcome.Forms {:forms tree}
      (wat.core/let
        [lines  (wat.string/split src "\n")
         forms  (wat.core/ast->children tree)
         edits  (fkb/walk-children forms lines src)
         sorted (wat.core/sort
                  (wat.core/fn [a :- (wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])
                                   b :- (wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])]
                    :- wat.type/bool
                    (wat.core/> (wat.core/first a) (wat.core/first b)))
                  edits)]
        (wat.fix/fix-text-apply src sorted))]))

(wat.core/defn user/apply-each
  [paths :- (wat.type/Vector :- [wat.type/String])] :- wat.type/nil
  (wat.core/if (wat.core/empty? paths)
    nil
    (wat.core/let [path (wat.core/first paths)]
      (wat.core/do
        (wat.io/write-file path (fkb/convert (wat.io/read-file path)))
        (wat.kernel/println (wat.string/concat "[fn-keyword-to-bracket] " path))
        (user/apply-each (wat.core/rest paths))))))

(wat.core/defn user/main [] :- wat.type/nil
  (user/apply-each
    (wat.core/match (wat.kernel/readln)
      [wat.kernel/ReadlnOutcome.Datum {:v __datum} __datum]
      [wat.kernel/ReadlnOutcome.Eof {} (wat.kernel/assertion-failed! :message "readln: end of input")]
      [wat.kernel/ReadlnOutcome.Stopped {} (wat.kernel/assertion-failed! :message "readln: stop requested")])))
