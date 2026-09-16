;; wat-scripts/fixes/positional-ctor-to-map.wat — arc 296 M2.
;; SCOPE: corpus
;;
;; Self-hosted, comment-faithful. Rewrites every positional VARIANT CONSTRUCTION:
;;
;;   (:ns::E::V a b)   ->  (:ns::E::V {:f1 a :f2 b})
;;   (:ns::E::Unit)    ->  (:ns::E::Unit {})
;;
;; Field names come from `:wat::fix::enum-fields` (declared-types on this
;; program's forms, then is-type?/type-of for stdlib). A tagged ctor whose
;; fields cannot be resolved is REPORTED, never guessed — when the head's
;; parent is a known enum and the leaf is not among its variants; never by
;; character case. Each `(:wat::core::forms …)` literal is its own program:
;; the door is asked on its children and that map applies inside that
;; subtree only. A `holon::defrecord` constructor and a kwargs `defn`'s
;; `::Kwargs` do not poison neighbouring enums. An UNREGISTERABLE form is
;; dropped and reported; the rest of the file's types still resolve.
;;
;; Idempotent: a single Map argument is already migrated. Accessors (`:ns::E/f`),
;; type applications `(:T :- […])`, and type references are untouched.
;;
;;   cat WORKLIST-296-M2-paths.edn | ./target/release/wat ./wat-scripts/fixes/positional-ctor-to-map.wat

(:wat::core::defn :user::node-text
  [node  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)
                      h  (:wat::core::if (:wat::core::empty? ch) "" (:wat::fix::kw-name (:wat::core::first ch)))]
      (:wat::core::if (:wat::core::or
                        (:wat::core::= h ":wat::core::unquote")
                        (:wat::core::= h ":wat::core::unquote-splicing"))
        (:wat::core::let
          [inner (:wat::core::Option/expect (:wat::core::get ch 1) "unquote inner")
           k (:wat::core::ast-kind inner)]
          (:wat::core::if (:wat::core::or (:wat::core::= k "symbol")
                            (:wat::core::or (:wat::core::= k "keyword") (:wat::core::= k "string")))
            (:wat::string::concat
              (:wat::core::if (:wat::core::= h ":wat::core::unquote") "~" "~@")
              (:wat::core::ast-name inner))
            (:wat::string::subs src
              (:wat::fix::node-start-offset node lines)
              (:wat::fix::node-end-offset node lines))))
        (:wat::string::subs src
          (:wat::fix::node-start-offset node lines)
          (:wat::fix::node-end-offset node lines))))
    (:wat::string::subs src
      (:wat::fix::node-start-offset node lines)
      (:wat::fix::node-end-offset node lines))))

;; ── field names from the door (2a2) ──────────────────────────────────────────
;; alias-enum stays (shared with match-arm / bare-variant-to-qualified).

(:wat::core::defn :user::join-with-space-sep
  [xs  <- (:wat::core::Vector :- [:wat::core::String])
   sep <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::if (:wat::core::empty? xs)
    ""
    (:wat::core::let [h (:wat::core::first xs) tl (:wat::core::rest xs)]
      (:wat::core::if (:wat::core::empty? tl)
        h
        (:wat::string::concat h (:wat::string::concat sep (:user::join-with-space-sep tl sep)))))))

(:wat::core::defn :user::parent-path [vpath <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [parts (:wat::string::split vpath "::")
                    n     (:wat::core::length parts)]
    (:wat::core::if (:wat::core::< n 2)
      vpath
      (:user::join-with-space-sep (:wat::core::into [] (:wat::core::take parts (:wat::i64::- n 1))) "::"))))

(:wat::core::defn :user::leaf-of [vpath <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [parts (:wat::string::split vpath "::")]
    (:wat::core::Option/expect
      (:wat::core::get parts (:wat::i64::- (:wat::core::length parts) 1))
      "variant leaf")))

(:wat::core::defn :user::alias-enum [leaf <- :wat::core::String] -> (:wat::core::Option :- [:wat::core::String])
  (:wat::core::if (:wat::core::or (:wat::core::= leaf "Some") (:wat::core::= leaf "None"))
    (:wat::core::Option.Some {:value ":wat::core::Option"})
    (:wat::core::if (:wat::core::or (:wat::core::= leaf "Ok") (:wat::core::= leaf "Err"))
      (:wat::core::Option.Some {:value ":wat::core::Result"})
      :wat::core::Option.None)))

(:wat::core::defn :user::report [msg <- :wat::core::String] -> :wat::core::nil
  (:wat::kernel::println (:wat::string::concat "[positional-ctor] UNRESOLVED " msg)))

(:wat::core::defn :user::node-line [node <- :wat::WatAST] -> :wat::core::String
  (:wat::i64::to-string
    (:wat::core::Option/expect
      (:wat::hashmap::get (:wat::core::ast-span node) :line)
      "node-line")))

(:wat::core::defn :user::report-site
  [head  <- :wat::WatAST
   node  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])
   path  <- :wat::core::String]
  -> :wat::core::nil
  (:user::report
    (:wat::string::concat
      (:wat::core::if (:wat::core::= (:wat::core::ast-kind head) "keyword")
        (:wat::core::ast-name head)
        (:user::node-text head src lines))
      (:wat::string::concat " "
        (:wat::string::concat path
          (:wat::string::concat ":" (:user::node-line node)))))))

(:wat::core::defn :user::has-colon-colon? [s <- :wat::core::String] -> :wat::core::bool
  (:wat::core::> (:wat::core::length (:wat::string::split s "::")) 1))

(:wat::core::defn :user::has-slash? [s <- :wat::core::String] -> :wat::core::bool
  (:wat::core::> (:wat::core::length (:wat::string::split s "/")) 1))

(:wat::core::defn :user::conj-unique
  [acc <- (:wat::core::Vector :- [:wat::core::String])
   s   <- :wat::core::String]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::if (:wat::vec::contains? acc s) acc (:wat::core::conj acc s)))

(:wat::core::defn :user::names-eq?
  [a <- (:wat::core::Vector :- [:wat::core::String])
   b <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::bool
  (:wat::core::if (:wat::core::not (:wat::core::= (:wat::core::length a) (:wat::core::length b)))
    false
    (:wat::core::foldl
      (:wat::core::fn [ok <- :wat::core::bool i <- :wat::core::i64] -> :wat::core::bool
        (:wat::core::if ok
          (:wat::core::=
            (:wat::core::Option/expect (:wat::core::get a i) "names-eq a")
            (:wat::core::Option/expect (:wat::core::get b i) "names-eq b"))
          false))
      true
      (:wat::core::range 0 (:wat::core::length a)))))

(:wat::core::defn :user::index-leaf
  [m      <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
   leaf   <- :wat::core::String
   fields <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
  (:wat::core::match (:wat::hashmap::get m leaf)
    [:wat::core::Option.None {} (:wat::hashmap::assoc m leaf fields)]
    [:wat::core::Option.Some {:value existing}
      (:wat::core::if (:user::names-eq? existing fields)
        m
        m)]
    [_ (:wat::hashmap::assoc m leaf fields)]))

(:wat::core::defn :user::collect-keywords
  [acc  <- (:wat::core::Vector :- [:wat::core::String])
   node <- :wat::WatAST]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "keyword")
    (:wat::core::let [nm (:wat::core::ast-name node)]
      (:wat::core::if (:wat::core::if (:user::has-colon-colon? nm)
                        (:wat::core::not (:user::has-slash? nm))
                        false)
        (:user::conj-unique acc nm)
        acc))
    (:wat::core::if (:wat::fix::structural? node)
      (:wat::core::foldl :user::collect-keywords acc (:wat::core::ast->children node))
      acc)))

(:wat::core::defn :user::enum-paths-of
  [vpaths <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) vp <- :wat::core::String]
      -> (:wat::core::Vector :- [:wat::core::String])
      (:wat::core::let [acc2 (:user::conj-unique acc vp)
                        acc3 (:user::conj-unique acc2 (:user::parent-path vp))]
        (:wat::core::match (:user::alias-enum (:user::leaf-of vp))
          [:wat::core::Option.Some {:value ep} (:user::conj-unique acc3 ep)]
          [:wat::core::Option.None {} acc3]
          [_ acc3])))
    (:wat::core::Vector :- [:wat::core::String])
    vpaths))

(:wat::core::defn :user::corpus-invariant-name? [nm <- :wat::core::String] -> :wat::core::bool
  (:wat::core::or
    (:wat::string::starts-with? nm ":wat::")
    (:wat::string::starts-with? nm ":rust::")))

(:wat::core::defn :user::report-ctor?
  [head  <- :wat::WatAST
   node  <- :wat::WatAST
   fmap  <- :wat::fix::EnumFields
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])
   path  <- :wat::core::String]
  -> :wat::core::bool
  (:wat::core::if (:user::unquote-form? head)
    true
    (:wat::core::if (:wat::core::not (:wat::core::= (:wat::core::ast-kind head) "keyword"))
      false
      (:wat::core::let [nm (:wat::core::ast-name head)]
        (:wat::core::if (:wat::core::or (:user::has-slash? nm)
                          (:wat::core::not (:user::has-colon-colon? nm)))
          false
          (:wat::fix::known-enum? fmap (:user::parent-path nm)))))))

(:wat::core::defn :user::invariant-eps-of
  [epaths <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) ep <- :wat::core::String]
      -> (:wat::core::Vector :- [:wat::core::String])
      (:wat::core::if (:user::corpus-invariant-name? ep)
        (:user::conj-unique acc ep)
        acc))
    (:wat::core::Vector :- [:wat::core::String])
    epaths))

(:wat::core::defn :user::local-eps-of
  [epaths <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) ep <- :wat::core::String]
      -> (:wat::core::Vector :- [:wat::core::String])
      (:wat::core::if (:user::corpus-invariant-name? ep)
        acc
        (:user::conj-unique acc ep)))
    (:wat::core::Vector :- [:wat::core::String])
    epaths))

(:wat::core::defn :user::merge-fmap
  [base   <- :wat::fix::EnumFields
   file-m <- :wat::fix::EnumFields]
  -> :wat::fix::EnumFields
  (:wat::core::let
    [b-fields (:wat::fix::EnumFields/fields base)
     f-fields (:wat::fix::EnumFields/fields file-m)
     keys (:wat::hashmap::keys f-fields)
     m (:wat::core::foldl
         (:wat::core::fn
           [acc <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
            k   <- :wat::core::String]
           -> (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
           (:wat::core::match (:wat::hashmap::get f-fields k)
             [:wat::core::Option.Some {:value v} (:wat::hashmap::assoc acc k v)]
             [:wat::core::Option.None {} acc]))
         b-fields
         keys)]
    (:wat::fix::EnumFields
      :fields m
      :answered (:user::union-eps
                  (:wat::fix::EnumFields/answered base)
                  (:wat::fix::EnumFields/answered file-m)))))

(:wat::core::defn :user::fmap-for-src
  [src   <- :wat::core::String
   path  <- :wat::core::String
   base  <- :wat::fix::EnumFields
   world <- :wat::fix::StdlibWorld]
  -> :wat::fix::EnumFields
  (:wat::core::let
    [tree (:wat::core::match (:wat::core::read-string src)
             [:wat::core::ReadOutcome.Forms {:forms f} f]
             [:wat::core::ReadOutcome.Malformed {:cause c}
               (:wat::kernel::assertion-failed! :message (:wat::core::Error/message c))])
     forms (:wat::core::ast->children tree)
     vpaths (:user::collect-keywords (:wat::core::Vector :- [:wat::core::String]) tree)
     epaths (:user::enum-paths-of vpaths)
     local (:wat::fix::enum-fields-in "positional-ctor" path world forms (:user::local-eps-of epaths))
     merged (:user::merge-fmap base local)
     fields2 (:user::bind-kw-ctors (:wat::fix::EnumFields/fields merged) tree)]
    (:wat::fix::EnumFields
      :fields fields2
      :answered (:wat::fix::EnumFields/answered merged))))

;; Derive unquote-ctor bindings from let-bound `*-kw` names whose value
;; interpolates a `::Leaf` (STOP-4: not a hand-list). Field names still
;; come from type-of, indexed by that leaf.

(:wat::core::defn :user::last-colon-leaf [s <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [parts (:wat::string::split s "::")
                    n     (:wat::core::length parts)]
    (:wat::core::if (:wat::core::< n 2)
      ""
      (:wat::core::let [leaf (:wat::core::Option/expect (:wat::core::get parts (:wat::i64::- n 1)) "leaf")]
        (:wat::core::if (:wat::core::> (:wat::core::length (:wat::string::split leaf "{")) 1)
          ""
          leaf)))))

;; Prefer `Status::Stopped` over `Stopped` so RecvOutcome::Stopped (unit) does
;; not steal Status::Stopped's fields (STOP-4: derived from the interpolate).
(:wat::core::defn :user::last-two-colon [s <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [parts (:wat::string::split s "::")
                    n     (:wat::core::length parts)]
    (:wat::core::if (:wat::core::< n 3)
      ""
      (:wat::core::let
        [a (:wat::core::Option/expect (:wat::core::get parts (:wat::i64::- n 2)) "parent")
         b (:wat::core::Option/expect (:wat::core::get parts (:wat::i64::- n 1)) "leaf")]
        (:wat::core::if (:wat::core::or
                          (:wat::core::> (:wat::core::length (:wat::string::split a "{")) 1)
                          (:wat::core::> (:wat::core::length (:wat::string::split b "{")) 1))
          ""
          (:wat::string::concat a (:wat::string::concat "::" b)))))))

(:wat::core::defn :user::string-lits
  [acc  <- (:wat::core::Vector :- [:wat::core::String])
   node <- :wat::WatAST]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "string")
    (:wat::core::conj acc (:wat::core::ast-name node))
    (:wat::core::if (:wat::fix::structural? node)
      (:wat::core::foldl :user::string-lits acc (:wat::core::ast->children node))
      acc)))

(:wat::core::defn :user::leaf-in-form [node <- :wat::WatAST] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [best <- :wat::core::String s <- :wat::core::String] -> :wat::core::String
      (:wat::core::let [two (:user::last-two-colon s)]
        (:wat::core::if (:wat::core::not (:wat::core::= two ""))
          two
          (:wat::core::let [leaf (:user::last-colon-leaf s)]
            (:wat::core::if (:wat::core::= leaf "") best leaf)))))
    ""
    (:user::string-lits (:wat::core::Vector :- [:wat::core::String]) node)))

(:wat::core::defn :user::bind-let-vec
  [m   <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
   vec <- :wat::WatAST]
  -> (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
  (:wat::core::let [ch (:wat::core::ast->children vec)
                    n  (:wat::core::length ch)]
    (:wat::core::foldl
      (:wat::core::fn
        [acc <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
         i   <- :wat::core::i64]
        -> (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
        (:wat::core::if (:wat::core::not (:wat::core::= (:wat::i64::rem i 2) 0))
          acc
          (:wat::core::if (:wat::core::>= (:wat::i64::+ i 1) n)
            acc
            (:wat::core::let
              [nm-node (:wat::core::Option/expect (:wat::core::get ch i) "let name")
               val     (:wat::core::Option/expect (:wat::core::get ch (:wat::i64::+ i 1)) "let val")]
              (:wat::core::if (:wat::core::not (:wat::core::= (:wat::core::ast-kind nm-node) "symbol"))
                acc
                (:wat::core::let [nm (:wat::core::ast-name nm-node)]
                  (:wat::core::if (:wat::core::not (:wat::string::ends-with? nm "-kw"))
                    acc
                    (:wat::core::let [leaf (:user::leaf-in-form val)]
                      (:wat::core::if (:wat::core::= leaf "")
                        acc
                        (:wat::core::match (:wat::hashmap::get acc leaf)
                          [:wat::core::Option.Some {:value fields} (:wat::hashmap::assoc acc nm fields)]
                          [:wat::core::Option.None {} acc]
                          [_ acc]))))))))))
      m
      (:wat::core::range 0 n))))

(:wat::core::defn :user::bind-kw-ctors
  [m    <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
   node <- :wat::WatAST]
  -> (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
  (:wat::core::if (:wat::core::not (:wat::fix::structural? node))
    m
    (:wat::core::let [ch (:wat::core::ast->children node)
                      m2 (:wat::core::if (:wat::core::if (:wat::core::= (:wat::fix::head-name node) ":wat::core::let")
                                           (:wat::core::if (:wat::core::> (:wat::core::length ch) 1)
                                             (:wat::core::= (:wat::core::ast-kind (:wat::core::Option/expect (:wat::core::get ch 1) "let vec")) "vector")
                                             false)
                                           false)
                         (:user::bind-let-vec m (:wat::core::Option/expect (:wat::core::get ch 1) "let vec"))
                         m)]
      (:wat::core::foldl :user::bind-kw-ctors m2 ch))))

(:wat::core::defn :user::unquote-ctor-leaf [inner <- :wat::core::String] -> (:wat::core::Option :- [:wat::core::String])
  (:wat::core::if (:wat::core::= inner "")
    :wat::core::Option.None
    (:wat::core::Option.Some {:value inner})))

(:wat::core::defn :user::unquote-form? [n <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::= (:wat::fix::head-name n) ":wat::core::unquote"))

(:wat::core::defn :user::unquote-splicing-form? [n <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::= (:wat::fix::head-name n) ":wat::core::unquote-splicing"))

(:wat::core::defn :user::has-splice? [args <- (:wat::core::Vector :- [:wat::WatAST])] -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::bool a <- :wat::WatAST] -> :wat::core::bool
      (:wat::core::or acc (:user::unquote-splicing-form? a)))
    false
    args))

(:wat::core::defn :user::unquote-inner-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::let
    [inner (:wat::core::Option/expect
             (:wat::core::get (:wat::core::ast->children n) 1)
             "unquote inner")
     k (:wat::core::ast-kind inner)]
    (:wat::core::if (:wat::core::or (:wat::core::= k "symbol")
                      (:wat::core::or (:wat::core::= k "keyword") (:wat::core::= k "string")))
      (:wat::core::ast-name inner)
      "")))

(:wat::core::defn :user::alias-ctor-name? [nm <- :wat::core::String] -> :wat::core::bool
  (:wat::core::or
    (:wat::core::= nm ":wat::core::Option::Some")
    (:wat::core::or
      (:wat::core::= nm ":wat::core::Option::None")
      (:wat::core::or
        (:wat::core::= nm ":wat::core::Ok")
        (:wat::core::= nm ":wat::core::Err")))))

(:wat::core::defn :user::map-value-node [mapn <- :wat::WatAST] -> (:wat::core::Option :- [:wat::WatAST])
  (:wat::core::let [ch (:wat::core::ast->children mapn)
                    n  (:wat::core::length ch)]
    (:wat::core::if (:wat::core::= n 2)
      (:wat::core::get ch 1)
      :wat::core::Option.None)))

(:wat::core::defn :user::unwrap-alias-edits
  [node  <- :wat::WatAST
   args  <- (:wat::core::Vector :- [:wat::WatAST])
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:wat::core::not (:user::already-map? args))
    (:wat::core::Vector :- [:wat::fix::Edit])
    (:wat::core::let [m (:wat::core::first args)]
      (:wat::core::match (:user::map-value-node m)
        [:wat::core::Option.Some {:value v}
          (:wat::core::Vector :- [:wat::fix::Edit]
            (:wat::core::Tuple
              (:wat::fix::node-start-offset m lines)
              (:user::node-text m src lines)
              (:user::node-text v src lines)))]
        [:wat::core::Option.None {}
          ;; unit None: `(:wat::core::Option::None {})` → drop the map, leave `(:wat::core::Option::None)`
          (:wat::core::Vector :- [:wat::fix::Edit]
            (:wat::core::Tuple
              (:wat::fix::node-start-offset m lines)
              (:user::node-text m src lines)
              ""))]))))

(:wat::core::defn :user::already-map? [args <- (:wat::core::Vector :- [:wat::WatAST])] -> :wat::core::bool
  (:wat::core::if (:wat::core::not (:wat::core::= (:wat::core::length args) 1))
    false
    (:wat::core::= (:wat::core::ast-kind (:wat::core::first args)) "map")))

(:wat::core::defn :user::type-app? [args <- (:wat::core::Vector :- [:wat::WatAST])] -> :wat::core::bool
  (:wat::core::if (:wat::core::empty? args)
    false
    (:wat::core::let [a (:wat::core::first args)
                      k (:wat::core::ast-kind a)]
      (:wat::core::if (:wat::core::or (:wat::core::= k "symbol") (:wat::core::= k "keyword"))
        (:wat::core::= (:wat::core::ast-name a) ":-")
        false))))

;; Insert-only wrap: `(:V a b)` → inserts `{:`field0` ` at a, `:`fieldi` ` at later args, `}` after last.
;; Nested ctors keep disjoint offsets so high-offset-first apply is sound.

(:wat::core::defn :user::tagged-edits
  [args   <- (:wat::core::Vector :- [:wat::WatAST])
   fields <- (:wat::core::Vector :- [:wat::core::String])
   lines  <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::let
    [n (:wat::core::length args)
     first-arg (:wat::core::Option/expect (:wat::core::get args 0) "tagged first")
     last-arg  (:wat::core::Option/expect (:wat::core::get args (:wat::i64::- n 1)) "tagged last")
     open (:wat::string::concat "{:"
            (:wat::string::concat
              (:wat::core::Option/expect (:wat::core::get fields 0) "tagged f0")
              " "))
     open-ed (:wat::core::Tuple (:wat::fix::node-start-offset first-arg lines) "" open)
     close-ed (:wat::core::Tuple (:user::arg-end-offset last-arg lines) "" "}")
     mid (:wat::core::foldl
           (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::fix::Edit]) i <- :wat::core::i64]
             -> (:wat::core::Vector :- [:wat::fix::Edit])
             (:wat::core::if (:wat::core::= i 0)
               acc
               (:wat::core::let
                 [arg (:wat::core::Option/expect (:wat::core::get args i) "tagged mid arg")
                  f   (:wat::core::Option/expect (:wat::core::get fields i) "tagged mid f")
                  ins (:wat::string::concat ":" (:wat::string::concat f " "))]
                 (:wat::core::conj acc
                   (:wat::core::Tuple (:wat::fix::node-start-offset arg lines) "" ins)))))
           (:wat::core::Vector :- [:wat::fix::Edit])
           (:wat::core::range 0 n))]
    (:wat::core::conj (:wat::core::conj mid open-ed) close-ed)))

;; Unquote `~x` — the list span often ends at `~`, not after `x`. Use the inner name's end.
(:wat::core::defn :user::arg-end-offset
  [arg   <- :wat::WatAST
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::core::if (:wat::core::or (:user::unquote-form? arg) (:user::unquote-splicing-form? arg))
    (:wat::fix::node-end-offset
      (:wat::core::Option/expect
        (:wat::core::get (:wat::core::ast->children arg) 1)
        "unquote inner end")
      lines)
    (:wat::fix::node-end-offset arg lines)))

(:wat::core::defn :user::variant-like-head? [head <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind head) "keyword")
    (:wat::core::let [nm (:wat::core::ast-name head)]
      (:wat::core::if (:user::has-slash? nm)
        false
        (:wat::core::if (:user::has-colon-colon? nm)
          (:wat::core::or
            (:wat::core::> (:wat::core::length (:wat::string::split nm "::")) 2)
            (:wat::core::or
              (:wat::core::= (:user::leaf-of nm) "Some")
              (:wat::core::or
                (:wat::core::= (:user::leaf-of nm) "None")
                (:wat::core::or
                  (:wat::core::= (:user::leaf-of nm) "Ok")
                  (:wat::core::= (:user::leaf-of nm) "Err")))))
          false)))
    (:user::unquote-form? head)))

(:wat::core::defn :user::unit-edits
  [node  <- :wat::WatAST
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::Vector :- [:wat::fix::Edit]
    (:wat::core::Tuple (:wat::i64::- (:wat::fix::node-end-offset node lines) 1) "" " {}")))

(:wat::core::defn :user::fields-for-head
  [head <- :wat::WatAST
   fmap <- :wat::fix::EnumFields]
  -> (:wat::core::Option :- [(:wat::core::Vector :- [:wat::core::String])])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind head) "keyword")
    (:wat::core::let [nm (:wat::core::ast-name head)]
      (:wat::core::if (:user::has-slash? nm)
        :wat::core::Option.None
        (:wat::fix::enum-fields-get fmap nm)))
    (:wat::core::if (:user::unquote-form? head)
      (:wat::core::match (:user::unquote-ctor-leaf (:user::unquote-inner-name head))
        [:wat::core::Option.Some {:value leaf} (:wat::fix::enum-fields-get fmap leaf)]
        [:wat::core::Option.None {} :wat::core::Option.None]
        [_ :wat::core::Option.None])
      :wat::core::Option.None)))

(:wat::core::defn :user::ctor-edits
  [node  <- :wat::WatAST
   fmap  <- :wat::fix::EnumFields
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])
   path  <- :wat::core::String]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:wat::core::not (:wat::core::= (:wat::core::ast-kind node) "list"))
    (:wat::core::Vector :- [:wat::fix::Edit])
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch)
        (:wat::core::Vector :- [:wat::fix::Edit])
        (:wat::core::let
          [head (:wat::core::first ch)
           args (:wat::core::into [] (:wat::core::rest ch))]
          (:wat::core::if (:user::type-app? args)
            (:user::walk-seq (:wat::core::into [] (:wat::core::drop args 1)) fmap src lines path)
            (:wat::core::if (:user::already-map? args)
              (:user::walk-seq args fmap src lines path)
              (:wat::core::if (:user::has-splice? args)
                (:wat::core::do
                  (:user::report-site head node src lines path)
                  (:user::walk-seq args fmap src lines path))
              (:wat::core::match (:user::fields-for-head head fmap)
                [:wat::core::Option.None {}
                  (:wat::core::do
                    (:wat::core::if (:user::report-ctor? head node fmap src lines path)
                      (:user::report-site head node src lines path)
                      nil)
                    (:wat::core::concat
                      (:user::walk-edits head fmap src lines path)
                      (:user::walk-seq args fmap src lines path)))]
                [:wat::core::Option.Some {:value fields}
                  (:wat::core::if (:wat::core::= (:wat::core::length args) (:wat::core::length fields))
                    (:wat::core::concat
                      (:wat::core::if (:wat::core::empty? fields)
                        (:user::unit-edits node lines)
                        (:user::tagged-edits args fields lines))
                      (:user::walk-seq args fmap src lines path))
                    (:wat::core::do
                      (:user::report-site head node src lines path)
                      (:user::walk-seq args fmap src lines path)))]
                [_ (:user::walk-seq args fmap src lines path)])))))))))

;; A defenum variant-name keyword is a declaration slot, not a constructor.
;; Walk field vectors; skip the name. See wat/fix.wat defenum-variant-start.
(:wat::core::defn :user::walk-defenum-variants
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   fmap  <- :wat::fix::EnumFields
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])
   path  <- :wat::core::String]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :- [:wat::fix::Edit])
    (:wat::core::let [h  (:wat::core::first items)
                      tl (:wat::core::rest items)]
      (:wat::core::if (:wat::core::= (:wat::core::ast-kind h) "keyword")
        (:wat::core::if (:wat::core::if (:wat::core::not (:wat::core::empty? tl))
                            (:wat::core::= (:wat::core::ast-kind (:wat::core::first tl)) "vector")
                            false)
          (:wat::core::concat
            (:user::walk-edits (:wat::core::first tl) fmap src lines path)
            (:user::walk-defenum-variants (:wat::core::rest tl) fmap src lines path))
          (:user::walk-defenum-variants tl fmap src lines path))
        (:wat::core::concat
          (:user::walk-edits h fmap src lines path)
          (:user::walk-defenum-variants tl fmap src lines path))))))

(:wat::core::defn :user::walk-edits
  [node  <- :wat::WatAST
   fmap  <- :wat::fix::EnumFields
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])
   path  <- :wat::core::String]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::if (:wat::core::= (:wat::fix::head-name node) ":wat::core::defenum")
      (:wat::core::let
        [ch    (:wat::core::ast->children node)
         start (:wat::fix::defenum-variant-start ch)
         pre   (:wat::core::into [] (:wat::core::take ch start))
         body  (:wat::core::into [] (:wat::core::drop ch start))]
        (:wat::core::concat
          (:user::walk-seq pre fmap src lines path)
          (:user::walk-defenum-variants body fmap src lines path)))
      (:wat::core::if (:wat::fix::calls-to? node ":wat::core::forms")
        (:wat::core::let
          [vpaths (:user::collect-keywords (:wat::core::Vector :- [:wat::core::String]) node)
           epaths (:user::enum-paths-of vpaths)
           child (:wat::fix::enum-fields-for-forms "positional-ctor" path node epaths)
           fields2 (:user::bind-kw-ctors (:wat::fix::EnumFields/fields child) node)
           child2 (:wat::fix::EnumFields
                    :fields fields2
                    :answered (:wat::fix::EnumFields/answered child))]
          (:user::walk-seq (:wat::core::ast->children node) child2 src lines path))
        (:user::ctor-edits node fmap src lines path)))
    (:wat::core::if (:wat::fix::structural? node)
      (:user::walk-seq (:wat::core::ast->children node) fmap src lines path)
      (:wat::core::Vector :- [:wat::fix::Edit]))))

(:wat::core::defn :user::walk-seq
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   fmap  <- :wat::fix::EnumFields
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])
   path  <- :wat::core::String]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :- [:wat::fix::Edit])
    (:wat::core::concat
      (:user::walk-edits (:wat::core::first items) fmap src lines path)
      (:user::walk-seq (:wat::core::rest items) fmap src lines path))))

(:wat::core::defn :user::migrate
  [src  <- :wat::core::String
   fmap <- :wat::fix::EnumFields
   path <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::let
    [lines (:wat::string::split src "\n")
     tree  (:wat::core::match (:wat::core::read-string src)
             [:wat::core::ReadOutcome.Forms {:forms __forms} __forms]
             [:wat::core::ReadOutcome.Malformed {:cause __cause}
               (:wat::kernel::assertion-failed! :message (:wat::core::Error/message __cause))])
     edits (:user::walk-seq (:wat::core::ast->children tree) fmap src lines path)]
    (:wat::fix::fix-text-apply src (:wat::core::reverse (:wat::core::sort edits)))))

(:wat::core::defn :user::skip-path? [path <- :wat::core::String] -> :wat::core::bool
  (:wat::core::or
    (:wat::core::= path "tests/types/probe_arc296_enum_map_ctor__positional.wat")
    (:wat::core::or
      (:wat::core::= path "wat-scripts/fixes/positional-ctor-to-map.wat")
      (:wat::core::or
        (:wat::core::= path "wat/service.wat")
        (:wat::core::or
          (:wat::core::= path "tests/cli/grep_smoke_target.wat")
          (:wat::core::if (:wat::core::> (:wat::string::length path) 17)
            (:wat::core::= (:wat::string::subs path 0 18) "wat-scripts/fixes/")
            false))))))


;; A′ — :wat:: / :rust:: resolve ONCE; every other name PER FILE.
;; Locals colliding across files is legal input (STOP-2): do not halt.

(:wat::core::defn :user::src-tree [src <- :wat::core::String] -> :wat::WatAST
  (:wat::core::match (:wat::core::read-string src)
    [:wat::core::ReadOutcome.Forms {:forms f} f]
    [:wat::core::ReadOutcome.Malformed {:cause c}
      (:wat::kernel::assertion-failed! :message (:wat::core::Error/message c))]))

(:wat::core::defn :user::union-eps
  [eps    <- (:wat::core::Vector :- [:wat::core::String])
   epaths <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::if (:wat::core::empty? epaths)
    eps
    (:user::union-eps
      (:user::conj-unique eps (:wat::core::first epaths))
      (:wat::core::rest epaths))))

(:wat::core::defn :user::rewrite-each
  [paths  <- (:wat::core::Vector :- [:wat::core::String])
   frozen <- :wat::fix::EnumFields
   world  <- :wat::fix::StdlibWorld]
  -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::if (:user::skip-path? path)
        (:wat::core::do
          (:wat::kernel::println (:wat::string::concat "[positional-ctor] skip positional-control " path))
          (:user::rewrite-each (:wat::core::rest paths) frozen world))
        (:wat::core::let [src (:wat::io::read-file path)
                          fmap (:user::fmap-for-src src path frozen world)]
          (:wat::core::do
            (:wat::io::write-file path (:user::migrate src fmap path))
            (:wat::kernel::println (:wat::string::concat "[positional-ctor] " path))
            (:user::rewrite-each (:wat::core::rest paths) frozen world)))))))

;; PASS 1: unique corpus-invariant epaths, resolved once against stdlib
;; (empty decls). PASS 2: kept locals against THIS file's decls (fmap-for-src).
(:wat::core::defn :user::collect-pass
  [scan    <- (:wat::core::Vector :- [:wat::core::String])
   all     <- (:wat::core::Vector :- [:wat::core::String])
   inv-eps <- (:wat::core::Vector :- [:wat::core::String])
   kept-n  <- :wat::core::i64
   skip-n  <- :wat::core::i64
   audit   <- :wat::core::bool
   base    <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
   world   <- :wat::fix::StdlibWorld]
  -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? scan)
    (:wat::core::do
      (:wat::kernel::println
        (:wat::string::concat
          "[positional-ctor] resolve corpus-invariant="
          (:wat::string::concat
            (:wat::i64::to-string (:wat::core::length inv-eps))
            (:wat::string::concat
              " per-file="
              (:wat::string::concat
                (:wat::i64::to-string (:wat::i64::+ kept-n skip-n))
                (:wat::string::concat
                  " kept-local="
                  (:wat::string::concat
                    (:wat::i64::to-string kept-n)
                    (:wat::string::concat " skipped-local=" (:wat::i64::to-string skip-n)))))))))
      ;; 2a4d — PASS 1's corpus-invariant map is answered from the SET's ONE stdlib
      ;; world (empty world ⇒ the baked snapshot, exactly what `"<stdlib>"` asked).
      (:user::rewrite-each all
        (:wat::fix::enum-fields-of-world world inv-eps)
        world))
    (:wat::core::let [path (:wat::core::first scan)]
      (:wat::core::if (:user::skip-path? path)
        (:user::collect-pass (:wat::core::rest scan) all inv-eps kept-n skip-n audit base world)
        (:wat::core::let
          [src    (:wat::io::read-file path)
           tree   (:user::src-tree src)
           vpaths (:user::collect-keywords (:wat::core::Vector :- [:wat::core::String]) tree)
           feps   (:user::enum-paths-of vpaths)
           inv2   (:user::union-eps inv-eps (:user::invariant-eps-of feps))
           local  (:user::local-eps-of feps)]
          (:user::collect-pass
            (:wat::core::rest scan)
            all
            inv2
            (:wat::i64::+ kept-n (:wat::core::length local))
            skip-n
            audit
            base
            world))))))

;; 2a4d — the SET's stdlib files are ONE world. The door is asked ONCE, over the forms
;; of every `wat/…` member of this set together, and every member is answered from that
;; one world. Membership is the substrate's own `:wat::fix::stdlib-source-path?` rule —
;; derived from the set, never a hand list. Built from the paths the CALLER handed over
;; (`wat/service.wat`, which `ensure-path` appends for the scan, is not read here: it is
;; not part of the caller's set unless the caller named it).
(:wat::core::defn :user::stdlib-world-of
  [paths <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::fix::StdlibWorld
  (:wat::core::let
    [members (:wat::core::into []
               (:wat::core::filter
                 (:wat::core::fn [p <- :wat::core::String] -> :wat::core::bool
                   (:wat::fix::stdlib-source-path? p))
                 paths))
     srcs    (:wat::core::into []
               (:wat::core::map
                 (:wat::core::fn [p <- :wat::core::String] -> :wat::core::String
                   (:wat::io::read-file p))
                 members))]
    (:wat::fix::stdlib-world "positional-ctor" members srcs)))

(:wat::core::defn :user::ensure-path
  [paths <- (:wat::core::Vector :- [:wat::core::String])
   extra <- :wat::core::String]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::if (:wat::vec::contains? paths extra)
    paths
    (:wat::core::conj paths extra)))

(:wat::core::defn :user::drop-audit
  [paths <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) p <- :wat::core::String]
      -> (:wat::core::Vector :- [:wat::core::String])
      (:wat::core::if (:wat::core::= p "--audit") acc (:wat::core::conj acc p)))
    (:wat::core::Vector :- [:wat::core::String])
    paths))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [paths (:wat::core::match (:wat::kernel::readln)
             [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum]
             [:wat::kernel::ReadlnOutcome.Eof {}
               (:wat::kernel::assertion-failed! :message "readln: end of input")]
             [:wat::kernel::ReadlnOutcome.Stopped {}
               (:wat::kernel::assertion-failed! :message "readln: stop requested")])
     audit (:wat::vec::contains? paths "--audit")
     paths1 (:user::drop-audit paths)
     paths2 (:user::ensure-path paths1 "wat/service.wat")
     base (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])]
    (:user::collect-pass
      paths2
      paths2
      (:wat::core::Vector :- [:wat::core::String])
      0
      0
      audit
      base
      (:user::stdlib-world-of paths1))))
