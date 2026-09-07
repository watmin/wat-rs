;; wat-scripts/fixes/match-arm-to-bracket-map-pattern.wat — arc 109 / 296.
;;
;; Self-hosted, comment-faithful. Rewrites every `:wat::core::match` ARM:
;;
;;   (( :Enum::Variant a b) body)   ->  [:Enum::Variant {:a a :b b} body]
;;   ( :Enum::Unit body)            ->  [:Enum::Unit {} body]
;;   (_ body) / (x body) / ({v :f} body)  ->  delimiter flip only
;;
;; Field names come from defenum declarations (plus Option/Result). A tagged
;; arm whose variant is not in that map is STOP-4 — report, do not hand-edit.
;; Idempotent: a Vector arm is already migrated. Nested match in a body is
;; still walked. `cond` is not touched.
;;
;;   printf '["pathA" …]\n' | cargo wat ./wat-scripts/fixes/match-arm-to-bracket-map-pattern.wat

(:wat::core::defn :user::node-text
  [node  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)
                      h  (:wat::core::if (:wat::core::empty? ch) "" (:wat::fix::kw-name (:wat::core::first ch)))]
      (:wat::core::if (:wat::core::= h ":wat::core::unquote")
        (:wat::string::concat "~"
          (:wat::core::ast-name (:wat::core::Option/expect (:wat::core::get ch 1) "unquote name")))
        (:wat::core::if (:wat::core::= h ":wat::core::unquote-splicing")
          (:wat::string::concat "~@"
            (:wat::core::ast-name (:wat::core::Option/expect (:wat::core::get ch 1) "unquote-splicing name")))
          (:wat::string::subs src
            (:wat::fix::node-start-offset node lines)
            (:wat::fix::node-end-offset node lines)))))
    (:wat::string::subs src
      (:wat::fix::node-start-offset node lines)
      (:wat::fix::node-end-offset node lines))))

(:wat::core::defn :user::join-with-space [xs <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::String
  (:wat::core::if (:wat::core::empty? xs)
    ""
    (:wat::core::let [h (:wat::core::first xs) tl (:wat::core::rest xs)]
      (:wat::core::if (:wat::core::empty? tl)
        h
        (:wat::string::concat h (:wat::string::concat " " (:user::join-with-space tl)))))))

(:wat::core::defn :user::simple-binder? [n <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::let [k (:wat::core::ast-kind n)]
    (:wat::core::not
      (:wat::core::if (:wat::core::= k "list") true
        (:wat::core::if (:wat::core::= k "vector") true
          (:wat::core::= k "map"))))))

;; ── defenum field map ────────────────────────────────────────────────────────

(:wat::core::defn :user::fieldvec-names [fv <- :wat::WatAST] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::let [ch (:wat::core::ast->children fv)
                    n  (:wat::core::length ch)]
    (:wat::core::if (:wat::core::= (:wat::i64::rem n 3) 0)
      (:wat::core::foldl
        (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) i <- :wat::core::i64]
          -> (:wat::core::Vector :- [:wat::core::String])
          (:wat::core::conj acc
            (:wat::core::ast-name (:wat::core::Option/expect (:wat::core::get ch (:wat::i64::* i 3)) "fv-name"))))
        (:wat::core::Vector :- [:wat::core::String])
        (:wat::core::range 0 (:wat::i64::/ n 3)))
      (:wat::core::Vector :- [:wat::core::String]))))

(:wat::core::defn :user::strip-params [name <- :wat::core::String] -> :wat::core::String
  (:wat::core::first (:wat::string::split name "<")))

(:wat::core::defn :user::seed-builtins
  [m <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])]
  -> (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
  (:wat::hashmap::assoc
    (:wat::hashmap::assoc
      (:wat::hashmap::assoc
        (:wat::hashmap::assoc
          (:wat::hashmap::assoc
            (:wat::hashmap::assoc
              (:wat::hashmap::assoc
                (:wat::hashmap::assoc m
                  ":wat::core::Some" (:wat::core::Vector :- [:wat::core::String] "value"))
                ":wat::core::Option::Some" (:wat::core::Vector :- [:wat::core::String] "value"))
              ":wat::core::None" (:wat::core::Vector :- [:wat::core::String]))
            ":wat::core::Option::None" (:wat::core::Vector :- [:wat::core::String]))
          ":wat::core::Ok" (:wat::core::Vector :- [:wat::core::String] "value"))
        ":wat::core::Result::Ok" (:wat::core::Vector :- [:wat::core::String] "value"))
      ":wat::core::Err" (:wat::core::Vector :- [:wat::core::String] "error"))
    ":wat::core::Result::Err" (:wat::core::Vector :- [:wat::core::String] "error")))

(:wat::core::defn :user::add-variant
  [m <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
   enum-name <- :wat::core::String
   vname <- :wat::core::String
   fields <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
  (:wat::hashmap::assoc m
    (:wat::string::concat enum-name
      (:wat::string::concat "::"
        (:wat::string::subs vname 1 (:wat::string::length vname))))
    fields))

(:wat::core::defn :user::eat-variants
  [m <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
   enum-name <- :wat::core::String
   items <- (:wat::core::Vector :- [:wat::WatAST])]
  -> (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
  (:wat::core::if (:wat::core::empty? items)
    m
    (:wat::core::let [h (:wat::core::first items) tl (:wat::core::rest items)]
      (:wat::core::if (:wat::core::= (:wat::core::ast-kind h) "keyword")
        (:wat::core::if (:wat::core::empty? tl)
          (:user::add-variant m enum-name (:wat::core::ast-name h) (:wat::core::Vector :- [:wat::core::String]))
          (:wat::core::let [n1 (:wat::core::first tl)]
            (:wat::core::if (:wat::core::= (:wat::core::ast-kind n1) "vector")
              (:user::eat-variants
                (:user::add-variant m enum-name (:wat::core::ast-name h) (:user::fieldvec-names n1))
                enum-name
                (:wat::core::rest tl))
              (:user::eat-variants
                (:user::add-variant m enum-name (:wat::core::ast-name h) (:wat::core::Vector :- [:wat::core::String]))
                enum-name
                tl))))
        (:user::eat-variants m enum-name tl)))))

(:wat::core::defn :user::add-defenum
  [m <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
   node <- :wat::WatAST]
  -> (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
  (:wat::core::if (:wat::fix::calls-to? node ":wat::core::defenum")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::< (:wat::core::length ch) 3)
        m
        (:wat::core::let [nm (:wat::core::Option/expect (:wat::core::get ch 1) "defenum name")]
          (:wat::core::if (:wat::core::= (:wat::core::ast-kind nm) "keyword")
            (:wat::core::let
              [enum-name (:user::strip-params (:wat::core::ast-name nm))
               c2 (:wat::core::Option/expect (:wat::core::get ch 2) "defenum c2")
               rest (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind c2) "symbol")
                                      (:wat::core::= (:wat::core::ast-name c2) ":-")
                                      false)
                       (:wat::core::into [] (:wat::core::drop ch 4))
                       (:wat::core::into [] (:wat::core::drop ch 3)))]
              (:user::eat-variants m enum-name rest))
            m))))
    m))

(:wat::core::defn :user::collect-node
  [m <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
   node <- :wat::WatAST]
  -> (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
  (:wat::core::let [m2 (:user::add-defenum m node)]
    (:wat::core::if (:wat::fix::structural? node)
      (:wat::core::foldl :user::collect-node m2 (:wat::core::ast->children node))
      m2)))

(:wat::core::defn :user::collect-src
  [m <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
   src <- :wat::core::String]
  -> (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
  (:wat::core::match (:wat::core::read-string src)
    [:wat::core::ReadOutcome::Forms {:forms tree}
      (:wat::core::foldl :user::collect-node m (:wat::core::ast->children tree))]
    [:wat::core::ReadOutcome::Malformed {:cause __cause} m]))

(:wat::core::defn :user::collect-paths
  [m <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
   paths <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
  (:wat::core::if (:wat::core::empty? paths)
    m
    (:user::collect-paths
      (:user::collect-src m (:wat::io::read-file (:wat::core::first paths)))
      (:wat::core::rest paths))))

;; ── arm rewrite ──────────────────────────────────────────────────────────────

(:wat::core::defn :user::fq-unit [kw <- :wat::core::String] -> :wat::core::String
  (:wat::core::if (:wat::core::= kw ":None") ":wat::core::None" kw))

(:wat::core::defn :user::binder-names-from
  [pch <- (:wat::core::Vector :- [:wat::WatAST])]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) b <- :wat::WatAST]
      -> (:wat::core::Vector :- [:wat::core::String])
      (:wat::core::if (:wat::core::= (:wat::core::ast-kind b) "symbol")
        (:wat::core::let [nm (:wat::core::ast-name b)]
          (:wat::core::if (:wat::core::= nm "_")
            (:wat::core::conj acc "_")
            (:wat::core::conj acc nm)))
        (:wat::core::if (:user::unquote-form? b)
          (:wat::core::conj acc (:user::unquote-inner-name b))
          (:wat::core::conj acc "nested"))))
    (:wat::core::Vector :- [:wat::core::String])
    (:wat::core::into [] (:wat::core::drop pch 1))))

(:wat::core::defn :user::map-text
  [fields  <- (:wat::core::Vector :- [:wat::core::String])
   binders <- (:wat::core::Vector :- [:wat::WatAST])
   src     <- :wat::core::String
   lines   <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::String
  (:wat::core::if (:wat::core::empty? fields)
    "{}"
    (:wat::core::let
      [entries (:wat::core::foldl
                 (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) i <- :wat::core::i64]
                   -> (:wat::core::Vector :- [:wat::core::String])
                   (:wat::core::conj acc
                     (:wat::string::concat ":"
                       (:wat::string::concat
                         (:wat::core::Option/expect (:wat::core::get fields i) "map-text field")
                         (:wat::string::concat " "
                           (:user::node-text
                             (:wat::core::Option/expect (:wat::core::get binders i) "map-text binder")
                             src lines))))))
                 (:wat::core::Vector :- [:wat::core::String])
                 (:wat::core::range 0 (:wat::core::length fields)))]
      (:wat::string::concat "{" (:wat::string::concat (:user::join-with-space entries) "}")))))

(:wat::core::defn :user::paren-flip-edits
  [arm   <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::let
    [start (:wat::fix::node-start-offset arm lines)
     end   (:wat::fix::node-end-offset arm lines)]
    (:wat::core::Vector :- [:wat::fix::Edit]
      (:wat::core::Tuple start "(" "[")
      (:wat::core::Tuple (:wat::core::- end 1) ")" "]"))))

(:wat::core::defn :user::opener-of
  [node  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::String
  (:wat::core::let [start (:wat::fix::node-start-offset node lines)]
    (:wat::string::subs src start (:wat::core::+ start 1))))

(:wat::core::defn :user::unquote-form? [n <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::= (:wat::fix::head-name n) ":wat::core::unquote"))

(:wat::core::defn :user::unquote-splicing-form? [n <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::= (:wat::fix::head-name n) ":wat::core::unquote-splicing"))

(:wat::core::defn :user::unquote-inner-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::ast-name
    (:wat::core::Option/expect
      (:wat::core::get (:wat::core::ast->children n) 1)
      "unquote inner")))

(:wat::core::defn :user::remaining-binders
  [pch <- (:wat::core::Vector :- [:wat::WatAST])]
  -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::into [] (:wat::core::drop pch 1)))

(:wat::core::defn :user::splice-remaining?
  [pch <- (:wat::core::Vector :- [:wat::WatAST])] -> :wat::core::bool
  (:wat::core::let [rest (:user::remaining-binders pch)]
    (:wat::core::if (:wat::core::not (:wat::core::= (:wat::core::length rest) 1))
      false
      (:user::unquote-splicing-form?
        (:wat::core::Option/expect (:wat::core::get rest 0) "splice rest")))))

(:wat::core::defn :user::single-unquote-remaining?
  [pch <- (:wat::core::Vector :- [:wat::WatAST])] -> :wat::core::bool
  (:wat::core::let [rest (:user::remaining-binders pch)]
    (:wat::core::if (:wat::core::not (:wat::core::= (:wat::core::length rest) 1))
      false
      (:user::unquote-form?
        (:wat::core::Option/expect (:wat::core::get rest 0) "unquote rest")))))

(:wat::core::defn :user::unit-unquote-arm-edits
  [arm   <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::let
    [pattern (:wat::core::Option/expect
               (:wat::core::get (:wat::core::ast->children arm) 0)
               "unit-unquote pattern")
     ptxt    (:user::node-text pattern src lines)]
    (:wat::core::concat
      (:user::paren-flip-edits arm src lines)
      (:wat::core::Vector :- [:wat::fix::Edit]
        (:wat::core::Tuple
          (:wat::fix::node-start-offset pattern lines)
          ptxt
          (:wat::string::concat ptxt " {}"))))))

(:wat::core::defn :user::splice-arm-edits
  [arm   <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::let
    [pattern (:wat::core::Option/expect
               (:wat::core::get (:wat::core::ast->children arm) 0)
               "splice pattern")
     head    (:wat::core::Option/expect
               (:wat::core::get (:wat::core::ast->children pattern) 0)
               "splice head")]
    (:wat::core::concat
      (:user::paren-flip-edits arm src lines)
      (:wat::core::Vector :- [:wat::fix::Edit]
        (:wat::core::Tuple
          (:wat::fix::node-start-offset pattern lines)
          (:user::node-text pattern src lines)
          (:wat::string::concat
            (:user::node-text head src lines)
            " ~init-arg-map-ast"))))))

(:wat::core::defn :user::tagged-template-pattern? [pattern <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::not (:wat::core::= (:wat::core::ast-kind pattern) "list"))
    false
    ;; `` `(~@step ~a) `` is a splice CALL (thread-last), not a match arm.
    ;; `` `((~ctor binders) body) `` is a spliced ARM (serve-op-arms). Discriminator:
    ;; the pattern's head is unquote (ctor), never unquote-splicing.
    (:wat::core::if (:user::unquote-splicing-form? pattern)
      false
      (:wat::core::let [pch (:wat::core::ast->children pattern)]
        (:wat::core::if (:wat::core::empty? pch)
          false
          (:wat::core::let [head (:wat::core::first pch)]
            (:wat::core::if (:user::unquote-form? head)
              true
              (:wat::core::if (:wat::core::= (:wat::core::ast-kind head) "keyword")
                (:wat::core::not (:wat::core::= (:wat::fix::kw-name head) ":wat::core::unquote"))
                false))))))))

(:wat::core::defn :user::builtin-or-mapped-fields
  [vpath <- :wat::core::String
   pch   <- (:wat::core::Vector :- [:wat::WatAST])
   fmap  <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::let [fopt (:wat::hashmap::get fmap vpath)]
    (:wat::core::match fopt
      [:wat::core::Some {:value fields} fields]
      [:wat::core::None {} (:user::binder-names-from pch)])))

(:wat::core::defn :user::variant-pattern-list?
  [node <- :wat::WatAST
   src  <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::bool
  (:wat::core::if (:wat::core::not (:wat::core::= (:wat::core::ast-kind node) "list"))
    false
    (:wat::core::if (:wat::core::not (:wat::core::= (:user::opener-of node src lines) "("))
      false
      (:wat::core::let [ch (:wat::core::ast->children node)]
        (:wat::core::if (:wat::core::empty? ch)
          false
          (:wat::core::= (:wat::core::ast-kind (:wat::core::first ch)) "keyword"))))))

;; Nested variant in a map-pattern VALUE: `(Variant binders…)` → `[Variant {:k v}]` (no body).
(:wat::core::defn :user::nested-variant-text
  [node  <- :wat::WatAST
   fmap  <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::String
  (:wat::core::let
    [pch     (:wat::core::ast->children node)
     head    (:wat::core::Option/expect (:wat::core::get pch 0) "nested variant head")
     binders (:wat::core::into [] (:wat::core::drop pch 1))
     vpath   (:wat::core::ast-name head)
     fields  (:user::builtin-or-mapped-fields vpath pch fmap)
     btexts  (:wat::core::foldl
               (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) b <- :wat::WatAST]
                 -> (:wat::core::Vector :- [:wat::core::String])
                 (:wat::core::conj acc
                   (:wat::core::if (:user::variant-pattern-list? b src lines)
                     (:user::nested-variant-text b fmap src lines)
                     (:user::node-text b src lines))))
               (:wat::core::Vector :- [:wat::core::String])
               binders)
     nfields (:wat::core::if (:wat::core::= (:wat::core::length fields) (:wat::core::length btexts))
               fields
               (:user::binder-names-from pch))]
    (:wat::string::concat
      (:user::node-text head src lines)
      (:wat::string::concat " "
        (:wat::core::if (:wat::core::empty? nfields)
          "{}"
          (:wat::core::let
            [entries (:wat::core::foldl
                       (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) i <- :wat::core::i64]
                         -> (:wat::core::Vector :- [:wat::core::String])
                         (:wat::core::conj acc
                           (:wat::string::concat ":"
                             (:wat::string::concat
                               (:wat::core::Option/expect (:wat::core::get nfields i) "nested field")
                               (:wat::string::concat " "
                                 (:wat::core::Option/expect (:wat::core::get btexts i) "nested binder"))))))
                       (:wat::core::Vector :- [:wat::core::String])
                       (:wat::core::range 0 (:wat::core::length nfields)))]
            (:wat::string::concat "{" (:wat::string::concat (:user::join-with-space entries) "}"))))))))

(:wat::core::defn :user::nested-pattern-edits
  [node  <- :wat::WatAST
   fmap  <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:user::variant-pattern-list? node src lines)
    (:wat::core::Vector :- [:wat::fix::Edit]
      (:wat::core::Tuple
        (:wat::fix::node-start-offset node lines)
        (:user::node-text node src lines)
        (:wat::string::concat "["
          (:wat::string::concat (:user::nested-variant-text node fmap src lines) "]"))))
    (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "map")
      (:user::walk-nested-pattern-seq (:wat::core::ast->children node) fmap src lines)
      (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
        ;; tuple pattern `(a b c)` — recurse in case a slot is a nested variant
        (:user::walk-nested-pattern-seq (:wat::core::ast->children node) fmap src lines)
        (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "vector")
          (:user::walk-nested-pattern-seq (:wat::core::ast->children node) fmap src lines)
          (:wat::core::Vector :- [:wat::fix::Edit]))))))

(:wat::core::defn :user::walk-nested-pattern-seq
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   fmap  <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :- [:wat::fix::Edit])
    (:wat::core::concat
      (:user::nested-pattern-edits (:wat::core::first items) fmap src lines)
      (:user::walk-nested-pattern-seq (:wat::core::rest items) fmap src lines))))

(:wat::core::defn :user::template-arm?
  [arm   <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::bool
  (:wat::core::if (:wat::core::not (:wat::core::= (:wat::core::ast-kind arm) "list"))
    false
    (:wat::core::if (:wat::core::not (:wat::core::= (:user::opener-of arm src lines) "("))
      false
      (:wat::core::let [ch (:wat::core::ast->children arm)]
        (:wat::core::if (:wat::core::not (:wat::core::= (:wat::core::length ch) 2))
          false
          (:user::tagged-template-pattern?
            (:wat::core::Option/expect (:wat::core::get ch 0) "template-arm pattern")))))))

(:wat::core::defn :user::variant-arm-edits
  [arm    <- :wat::WatAST
   fields <- (:wat::core::Vector :- [:wat::core::String])
   src    <- :wat::core::String
   lines  <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::let
    [ch      (:wat::core::ast->children arm)
     pattern (:wat::core::Option/expect (:wat::core::get ch 0) "variant-arm pattern")
     flip    (:user::paren-flip-edits arm src lines)]
    (:wat::core::if (:wat::core::= (:wat::core::ast-kind pattern) "keyword")
      (:wat::core::concat flip
        (:wat::core::Vector :- [:wat::fix::Edit]
          (:wat::core::Tuple
            (:wat::fix::node-start-offset pattern lines)
            (:user::node-text pattern src lines)
            (:wat::string::concat (:user::fq-unit (:wat::core::ast-name pattern)) " {}"))))
      (:wat::core::let
        [pch     (:wat::core::ast->children pattern)
         head    (:wat::core::Option/expect (:wat::core::get pch 0) "variant head")
         binders (:wat::core::into [] (:wat::core::drop pch 1))]
        (:wat::core::if (:wat::core::= (:wat::core::length binders) (:wat::core::length fields))
          (:wat::core::concat flip
            (:wat::core::Vector :- [:wat::fix::Edit]
              (:wat::core::Tuple
                (:wat::fix::node-start-offset pattern lines)
                (:user::node-text pattern src lines)
                (:wat::string::concat
                  (:user::node-text head src lines)
                  (:wat::string::concat " " (:user::map-text fields binders src lines))))))
          (:wat::kernel::assertion-failed!
            (:wat::string::concat
              "match-arm-to-bracket-map-pattern STOP-4: binder count disagrees with defenum in "
              (:wat::core::write-forms arm))
            :wat::core::None :wat::core::None))))))

(:wat::core::defn :user::arm-edits
  [arm    <- :wat::WatAST
   fmap   <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
   src    <- :wat::core::String
   lines  <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind arm) "vector")
    ;; already migrated — convert nested variant patterns in the PATTERN
    ;; slots only (map values / tuple slots). Do NOT walk the body as a
    ;; pattern (body constructors like `(:wat::core::Some 42)` stay calls).
    (:wat::core::let [ch (:wat::core::ast->children arm)
                      n  (:wat::core::length ch)]
      (:wat::core::if (:wat::core::= n 3)
        (:wat::core::concat
          (:user::nested-pattern-edits
            (:wat::core::Option/expect (:wat::core::get ch 1) "vector-arm map")
            fmap src lines)
          (:user::walk-edits
            (:wat::core::Option/expect (:wat::core::get ch 2) "vector-arm body")
            fmap src lines))
        (:wat::core::if (:wat::core::= n 2)
          (:wat::core::concat
            (:user::nested-pattern-edits
              (:wat::core::Option/expect (:wat::core::get ch 0) "vector-arm pat")
              fmap src lines)
            (:user::walk-edits
              (:wat::core::Option/expect (:wat::core::get ch 1) "vector-arm body")
              fmap src lines))
          (:user::walk-seq-edits ch fmap src lines))))
    (:wat::core::if (:wat::core::= (:wat::core::ast-kind arm) "list")
      (:wat::core::let
        [ch (:wat::core::ast->children arm)
         start (:wat::fix::node-start-offset arm lines)
         opener (:wat::string::subs src start (:wat::core::+ start 1))]
        (:wat::core::if (:wat::core::not (:wat::core::= opener "("))
          ;; Reader-macro list (`~@arms`, `~x`) — not a paren clause. Walk only.
          (:user::walk-seq-edits ch fmap src lines)
        (:wat::core::if (:wat::core::= (:wat::core::length ch) 2)
          (:wat::core::let
            [pattern (:wat::core::Option/expect (:wat::core::get ch 0) "arm pattern")
             body    (:wat::core::Option/expect (:wat::core::get ch 1) "arm body")
             pk      (:wat::core::ast-kind pattern)
             body-eds (:user::walk-edits body fmap src lines)]
            (:wat::core::concat
              (:wat::core::if (:wat::core::= pk "keyword")
                (:user::variant-arm-edits arm (:wat::core::Vector :- [:wat::core::String]) src lines)
                (:wat::core::if (:wat::core::= pk "list")
                  (:wat::core::let
                    [pch (:wat::core::ast->children pattern)
                     head (:wat::core::Option/expect (:wat::core::get pch 0) "list-pat head")]
                    (:wat::core::if (:user::unquote-form? pattern)
                      ;; Unit unquote: `(~admin-stop-kw body)` → `[~admin-stop-kw {} body]`.
                      ;; The pattern IS the unquote list; treating unquote-kw as a
                      ;; constructor mangles the head to a bare `~`.
                      (:user::unit-unquote-arm-edits arm src lines)
                    (:wat::core::if (:wat::core::= (:wat::core::ast-kind head) "keyword")
                      (:wat::core::let
                        [vpath (:wat::core::ast-name head)
                         fopt  (:wat::hashmap::get fmap vpath)]
                        (:wat::core::match fopt
                          [:wat::core::Some {:value fields}
                            (:user::variant-arm-edits arm fields src lines)]
                          [:wat::core::None {}
                            (:user::variant-arm-edits arm
                              (:user::binder-names-from pch)
                              src lines)]))
                      ;; Generated template: `(~ctor binder…)` — field names are
                      ;; the binder symbols themselves (the defenum is also
                      ;; generated in the same quasiquote; no FQDN to look up).
                      ;; Splice-only remaining (`~@init-arg-names`) cannot invent
                      ;; a key-first map; unquote the generator's Map AST.
                      ;; A single unquote remaining is the Op `req` field.
                      (:wat::core::if (:user::splice-remaining? pch)
                        (:wat::core::let
                          [spnm (:user::unquote-inner-name
                                  (:wat::core::Option/expect
                                    (:wat::core::get (:user::remaining-binders pch) 0)
                                    "splice binder"))]
                          (:wat::core::if (:wat::core::= spnm "init-arg-names")
                            (:user::splice-arm-edits arm src lines)
                            (:wat::kernel::assertion-failed!
                              (:wat::string::concat
                                "match-arm-to-bracket-map-pattern STOP-4: unquote-splicing binder `"
                                (:wat::string::concat spnm "` is not init-arg-names"))
                              :wat::core::None :wat::core::None)))
                      (:wat::core::if (:user::single-unquote-remaining? pch)
                        (:wat::core::let
                          [unm (:user::unquote-inner-name
                                 (:wat::core::Option/expect
                                   (:wat::core::get (:user::remaining-binders pch) 0)
                                   "unquote binder"))
                           fields (:wat::core::if (:wat::core::= unm "req-binder")
                                    (:wat::core::Vector :- [:wat::core::String] "req")
                                    (:wat::core::Vector :- [:wat::core::String] unm))]
                          (:user::variant-arm-edits arm fields src lines))
                        (:user::variant-arm-edits arm (:user::binder-names-from pch) src lines))))))
                  ;; wildcard / binding / hash-destructure / literal: delimiter only
                  (:user::paren-flip-edits arm src lines)))
              body-eds))
          (:user::walk-seq-edits ch fmap src lines))))
      (:user::walk-edits arm fmap src lines))))

(:wat::core::defn :user::walk-edits
  [node  <- :wat::WatAST
   fmap  <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:wat::core::or
                    (:wat::fix::calls-to? node ":wat::core::match")
                    (:wat::fix::calls-to? node ":wat::rete::core::match"))
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::< (:wat::core::length ch) 2)
        (:wat::core::Vector :- [:wat::fix::Edit])
        (:wat::core::concat
          (:user::walk-edits (:wat::core::Option/expect (:wat::core::get ch 1) "match scrut") fmap src lines)
          (:wat::core::foldl
            (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::fix::Edit]) arm <- :wat::WatAST]
              -> (:wat::core::Vector :- [:wat::fix::Edit])
              (:wat::core::concat acc (:user::arm-edits arm fmap src lines)))
            (:wat::core::Vector :- [:wat::fix::Edit])
            (:wat::core::into [] (:wat::core::drop ch 2))))))
    (:wat::core::if (:wat::fix::structural? node)
      (:wat::core::let [ch (:wat::core::ast->children node)]
        (:wat::core::if (:wat::core::if (:wat::core::= (:wat::fix::head-name node) ":wat::core::quasiquote")
                          (:wat::core::if (:wat::core::= (:wat::core::length ch) 2)
                            (:user::template-arm?
                              (:wat::core::Option/expect (:wat::core::get ch 1) "qq inner")
                              src lines)
                            false)
                          false)
          ;; Syntax-quoted 2-list used as a match-arm template (serve-op-arms):
          ;; not a child of `match` in the source, spliced in later via `~@`.
          ;; Reader-macro `` `form `` is a 2-child list [quasiquote-kw, form].
          (:user::arm-edits
            (:wat::core::Option/expect (:wat::core::get ch 1) "qq arm")
            fmap src lines)
          (:user::walk-seq-edits ch fmap src lines)))
      (:wat::core::Vector :- [:wat::fix::Edit]))))

(:wat::core::defn :user::walk-seq-edits
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   fmap  <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :- [:wat::fix::Edit])
    (:wat::core::concat
      (:user::walk-edits (:wat::core::first items) fmap src lines)
      (:user::walk-seq-edits (:wat::core::rest items) fmap src lines))))

(:wat::core::defn :user::migrate
  [src  <- :wat::core::String
   fmap <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])]
  -> :wat::core::String
  (:wat::core::let
    [lines (:wat::string::split src "\n")
     tree  (:wat::core::match (:wat::core::read-string src)
             [:wat::core::ReadOutcome::Forms {:forms __forms} __forms]
             [:wat::core::ReadOutcome::Malformed {:cause __cause}
               (:wat::kernel::assertion-failed!
                 (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)])
     edits (:user::walk-seq-edits (:wat::core::ast->children tree) fmap src lines)]
    (:wat::fix::fix-text-apply src (:wat::core::reverse (:wat::core::sort edits)))))

(:wat::core::defn :user::rewrite-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])
   fmap  <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])]
  -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::if (:wat::core::= path "tests/wat_lang/probe_arc109_match_arm__positional_control.wat")
        (:wat::core::do
          (:wat::kernel::println (:wat::string::concat "[match-arm] skip positional-control " path))
          (:user::rewrite-each (:wat::core::rest paths) fmap))
        (:wat::core::do
          (:wat::io::write-file path (:user::migrate (:wat::io::read-file path) fmap))
          (:wat::kernel::println (:wat::string::concat "[match-arm] " path))
          (:user::rewrite-each (:wat::core::rest paths) fmap))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [paths (:wat::core::match (:wat::kernel::readln)
             [:wat::kernel::ReadlnOutcome::Datum {:v __datum} __datum]
             [:wat::kernel::ReadlnOutcome::Eof {}
               (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)]
             [:wat::kernel::ReadlnOutcome::Stopped {}
               (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)])
     fmap  (:user::collect-paths (:user::seed-builtins
              (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])]))
            paths)]
    (:user::rewrite-each paths fmap)))
