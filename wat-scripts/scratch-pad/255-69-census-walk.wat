;; wat-scripts/scratch-pad/255-69-census-walk.wat — STONE 255.69 extraction helper.
;;
;; Report-only census walk (NOT a codemod — writes nothing). For each path read from stdin
;; (an EDN vector of paths, same shape `wat-scripts/fixes/*.wat` drivers use), parses the
;; file's source and walks the whole form tree, printing one TSV line per untyped
;; constructor-call site: `head\tfile\tline\tcol`. Mirrors the hit rule
;; SCORE-STONE-255.68 measured: a List node whose head is one of the five constructor
;; names, in either spelling (`:wat::core::X` keyword or the post-255.67 `wat.type/X`
;; symbol), where `items[1]` is not the `:-` binder keyword (an empty call is still a
;; hit). Position recorded is the LIST node's OWN span (its `(`) — verified against
;; `tests/cli/wat_cli__check_types.wat`'s golden: the recorded type-record node for
;; `(:wat::core::PersistentVector)` at line 11 sits at column 23, the call's own `(`, not
;; the head keyword's column.
;;
;; Used to build STONE 255.69's type table (paired against `WAT_CHECK_TYPES=1 wat --check`
;; output, joined by exact file:line:col) — not itself part of the committed codemod.

(wat.core/defn c255-69/head-names [] :- (wat.type/HashSet :- [wat.type/String])
  (wat.type/HashSet :- [wat.type/String]
    ":wat::core::PersistentVector" "wat.type/PersistentVector"
    ":wat::core::Tuple" "wat.type/Tuple"
    ":wat::core::PersistentMap" "wat.type/PersistentMap"
    ":wat::core::List" "wat.type/List"
    ":wat::core::Vector" "wat.type/Vector"))

(wat.core/defn c255-69/item-name [n :- wat.type/AST] :- wat.type/String
  (wat.core/let [k (wat.core/ast-kind n)]
    (wat.core/if (wat.core/or (wat.core/= k "keyword") (wat.core/= k "symbol"))
      (wat.core/ast-name n)
      "")))

;; is-hit? — a genuine constructor-call List (a reader-synthesized keyword head is not
;; a documented corpus shape for these five names, unlike the 24-name type-position
;; sweep `types-to-wat-type.wat` guards against — no genuineness check needed here).
(wat.core/defn c255-69/is-hit? [node :- wat.type/AST] :- wat.type/bool
  (wat.core/if (wat.core/= (wat.core/ast-kind node) "list")
    (wat.core/let [ch (wat.core/ast->children node)]
      (wat.core/if (wat.core/empty? ch)
        false
        (wat.core/let [head  (wat.core/first ch)
                          hname (c255-69/item-name head)]
          (wat.core/if (wat.core/contains? (c255-69/head-names) hname)
            (wat.core/if (wat.core/< (wat.core/length ch) 2)
              true
              (wat.core/not (wat.core/= (c255-69/item-name (wat.core/nth ch 1)) ":-")))
            false))))
    false))

(wat.core/defn c255-69/line-for
  [node :- wat.type/AST
   path :- wat.type/String]
  :- wat.type/String
  (wat.core/let [ch    (wat.core/ast->children node)
                    head  (wat.core/first ch)
                    hname (c255-69/item-name head)
                    line  (wat.core/str (wat.fix/span-line node))
                    col   (wat.core/str (wat.fix/span-col node))]
    (wat.string/concat hname "\t" path "\t" line "\t" col)))

;; walk — every node in the tree; recurse into structural nodes; report every hit.
(wat.core/defn c255-69/walk
  [node :- wat.type/AST
   path :- wat.type/String]
  :- (wat.type/Vector :- [wat.type/String])
  (wat.core/let [self-hit (wat.core/if (c255-69/is-hit? node)
                                (wat.type/Vector :- [wat.type/String] (c255-69/line-for node path))
                                (wat.type/Vector :- [wat.type/String]))]
    (wat.core/if (wat.fix/structural? node)
      (wat.core/concat self-hit (c255-69/walk-seq (wat.core/ast->children node) path))
      self-hit)))

(wat.core/defn c255-69/walk-seq
  [items :- (wat.type/Vector :- [wat.type/AST])
   path  :- wat.type/String]
  :- (wat.type/Vector :- [wat.type/String])
  (wat.core/if (wat.core/empty? items)
    (wat.type/Vector :- [wat.type/String])
    (wat.core/concat
      (c255-69/walk (wat.core/first items) path)
      (c255-69/walk-seq (wat.core/rest items) path))))

;; A handful of corpus files are DELIBERATE lex/parse-negative fixtures (`.wat.bad`,
;; SCORE-STONE-255.68: "9 files could not even be parsed"). Uncountable, correctly:
;; skip rather than abort the whole census over one intentionally-broken file.
(wat.core/defn c255-69/census-one [path :- wat.type/String] :- wat.type/nil
  (wat.core/let [src  (wat.io/read-file path)]
    (wat.core/match (wat.core/read-string src)
      [wat.core/ReadOutcome.Forms {:forms __forms}
        (c255-69/print-all (c255-69/walk-seq (wat.core/ast->children __forms) path))]
      [wat.core/ReadOutcome.Malformed {:cause __cause} nil])))

(wat.core/defn c255-69/print-all [lines :- (wat.type/Vector :- [wat.type/String])] :- wat.type/nil
  (wat.core/if (wat.core/empty? lines)
    nil
    (wat.core/do
      (wat.kernel/println (wat.core/first lines))
      (c255-69/print-all (wat.core/rest lines)))))

(wat.core/defn c255-69/census-each [paths :- (wat.type/Vector :- [wat.type/String])] :- wat.type/nil
  (wat.core/if (wat.core/empty? paths)
    nil
    (wat.core/do
      (c255-69/census-one (wat.core/first paths))
      (c255-69/census-each (wat.core/rest paths)))))

(wat.core/defn user/main [] :- wat.type/nil
  (c255-69/census-each
    (wat.core/match (wat.kernel/readln)
      [wat.kernel/ReadlnOutcome.Datum {:v __datum} __datum]
      [wat.kernel/ReadlnOutcome.Eof {} (wat.kernel/assertion-failed! :message "readln: end of input")]
      [wat.kernel/ReadlnOutcome.Stopped {} (wat.kernel/assertion-failed! :message "readln: stop requested")])))
