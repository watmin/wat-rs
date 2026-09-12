;; wat-scripts/fixes/positional-ctor-to-map.wat — arc 296 M2.
;;
;; Self-hosted, comment-faithful. Rewrites every positional VARIANT CONSTRUCTION:
;;
;;   (:ns::E::V a b)   ->  (:ns::E::V {:f1 a :f2 b})
;;   (:ns::E::Unit)    ->  (:ns::E::Unit {})
;;
;; Field names come from `:wat::runtime::type-of` (declared fields, declaration
;; order). In-file types are registered via eval-with-defs! of declaration forms
;; only (no defn bodies). A tagged ctor whose fields cannot be resolved is
;; REPORTED, never guessed. 32 of 60 affected enums are generated (defservice /
;; defsurface) — a byte-observing map cannot see them.
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

;; ── field names from type-of (RELAND 4 / M2) ─────────────────────────────────

(:wat::core::defn :user::decl-head? [h <- :wat::core::String] -> :wat::core::bool
  (:wat::core::or
    (:wat::core::= h ":wat::core::defenum")
    (:wat::core::or
      (:wat::core::= h ":wat::core::defrecord")
      (:wat::core::or
        (:wat::core::= h ":wat::core::defstruct")
        (:wat::core::or
          (:wat::core::= h ":wat::core::defsurface")
          (:wat::core::or
            (:wat::core::= h ":wat::core::newtype")
            (:wat::core::or
              (:wat::core::= h ":wat::core::typealias")
              (:wat::core::or
                (:wat::core::= h ":wat::core::typeunion")
                (:wat::core::or
                  (:wat::core::= h ":wat::service::defservice")
                  (:wat::core::or
                    (:wat::core::= h ":wat::query::sift-rules-defsvc")
                    (:wat::core::= h ":wat::core::defmacro")))))))))))

(:wat::core::defn :user::file-decls [tree <- :wat::WatAST] -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::into []
    (:wat::core::filter
      (:wat::core::fn [n <- :wat::WatAST] -> :wat::core::bool
        (:user::decl-head? (:wat::fix::head-name n)))
      (:wat::core::ast->children tree))))

(:wat::core::defn :user::type-of-form [enum-path <- :wat::core::String] -> :wat::WatAST
  (:wat::core::match
    (:wat::core::read-string
      (:wat::string::concat "(:wat::runtime::type-of " (:wat::string::concat enum-path ")")))
    [:wat::core::ReadOutcome.Forms {:forms f}
      (:wat::core::first (:wat::core::ast->children f))]
    [:wat::core::ReadOutcome.Malformed {:cause c}
      (:wat::kernel::assertion-failed! :message (:wat::core::Error/message c))]))

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

(:wat::core::defn :user::kw-name-text [k <- :wat::core::keyword] -> :wat::core::String
  (:wat::keyword::to-string k))

(:wat::core::defn :user::variant-field-names
  [v <- :wat::runtime::TypeVariant]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::into []
    (:wat::core::map
      (:wat::core::fn [f <- :wat::runtime::TypeField] -> :wat::core::String
        (:user::kw-name-text (:wat::runtime::TypeField/name f)))
      (:wat::runtime::TypeVariant/fields v))))

;; defservice/defsurface in the same file can poison eval-with-defs! of a
;; neighbouring defenum (dead_child: EchoRequest's `:wat::query::Reason` field
;; made the whole decls vector fail, so `:probe::Outcome` was UNRESOLVED).
;; Retry with only the type-declaration forms before falling back to stdlib.
(:wat::core::defn :user::simple-type-head? [h <- :wat::core::String] -> :wat::core::bool
  (:wat::core::or
    (:wat::core::= h ":wat::core::defenum")
    (:wat::core::or
      (:wat::core::= h ":wat::core::defrecord")
      (:wat::core::or
        (:wat::core::= h ":wat::core::defstruct")
        (:wat::core::or
          (:wat::core::= h ":wat::core::newtype")
          (:wat::core::or
            (:wat::core::= h ":wat::core::typealias")
            (:wat::core::= h ":wat::core::typeunion")))))))

(:wat::core::defn :user::simple-type-decls
  [decls <- (:wat::core::Vector :- [:wat::WatAST])]
  -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::into []
    (:wat::core::filter
      (:wat::core::fn [n <- :wat::WatAST] -> :wat::core::bool
        (:user::simple-type-head? (:wat::fix::head-name n)))
      decls)))

(:wat::core::defn :user::without-defservice
  [decls <- (:wat::core::Vector :- [:wat::WatAST])]
  -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::into []
    (:wat::core::filter
      (:wat::core::fn [n <- :wat::WatAST] -> :wat::core::bool
        (:wat::core::not (:wat::core::= (:wat::fix::head-name n) ":wat::service::defservice")))
      decls)))

(:wat::core::defn :user::try-type-of
  [enum-path <- :wat::core::String
   decls     <- (:wat::core::Vector :- [:wat::WatAST])]
  -> (:wat::core::Option :- [:wat::runtime::TypeInfo])
  (:wat::core::match
    (:wat::eval-with-defs! :- [:wat::runtime::TypeInfo]
      (:user::type-of-form enum-path) decls)
    [:wat::eval::FormOutcome.Evaluated {:value v}
      (:wat::core::match (:wat::runtime::TypeInfo/kind v)
        [:wat::runtime::TypeKind.Enum {} (:wat::core::Option.Some {:value v})]
        [_ :wat::core::Option.None])]
    [_ (:wat::core::if (:wat::core::empty? decls)
         :wat::core::Option.None
         (:wat::core::let [nosvc (:user::without-defservice decls)]
           (:wat::core::if (:wat::core::< (:wat::core::length nosvc) (:wat::core::length decls))
             (:user::try-type-of enum-path nosvc)
             (:wat::core::let [simple (:user::simple-type-decls decls)]
               (:wat::core::if (:wat::core::if (:wat::core::not (:wat::core::empty? simple))
                                  (:wat::core::not (:wat::core::= (:wat::core::length simple) (:wat::core::length decls)))
                                  false)
                 (:user::try-type-of enum-path simple)
                 (:user::try-type-of enum-path (:wat::core::Vector :- [:wat::WatAST])))))))]))

(:wat::core::defn :user::report [msg <- :wat::core::String] -> :wat::core::nil
  (:wat::kernel::println (:wat::string::concat "[positional-ctor] UNRESOLVED " msg)))

(:wat::core::defn :user::node-line [node <- :wat::WatAST] -> :wat::core::String
  (:wat::i64::to-string
    (:wat::core::Option/expect
      (:wat::hashmap::get (:wat::core::ast-span node) :line)
      "node-line")))

(:wat::core::defn :user::pascal-leaf? [nm <- :wat::core::String] -> :wat::core::bool
  (:wat::core::if (:user::has-slash? nm)
    false
    (:wat::core::let [leaf (:user::leaf-of nm)]
      (:wat::core::if (:wat::core::= leaf "")
        false
        (:wat::core::let [c (:wat::string::subs leaf 0 1)]
          (:wat::core::= c (:wat::string::to-uppercase c)))))))

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

(:wat::core::defn :user::fill-enum
  [m         <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
   info      <- :wat::runtime::TypeInfo
   enum-path <- :wat::core::String]
  -> (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
  (:wat::core::match (:wat::runtime::TypeInfo/body info)
    [:wat::runtime::TypeBody.Enum {:purity _ :variants vs}
      (:wat::core::foldl
        (:wat::core::fn
          [acc <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
           v   <- :wat::runtime::TypeVariant]
          -> (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
          (:wat::core::let
            [leaf   (:user::kw-name-text (:wat::runtime::TypeVariant/name v))
             fields (:user::variant-field-names v)
             fq     (:wat::string::concat enum-path (:wat::string::concat "::" leaf))
             acc2   (:wat::hashmap::assoc acc fq fields)
             acc3   (:user::index-leaf acc2 leaf fields)
             short  (:wat::string::concat (:user::leaf-of enum-path) (:wat::string::concat "::" leaf))
             acc4   (:user::index-leaf acc3 short fields)]
            acc4))
        m vs)]
    [_ m]))

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

(:wat::core::defn :user::seed-paths [] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::Vector :- [:wat::core::String]
    ":wat::core::Option"
    ":wat::core::Result"
    ":wat::kernel::RecvOutcome"
    ":wat::kernel::SendOutcome"
    ":wat::service::Outcome"
    ":wat::cache::Cache::GetResponse"
    ":wat::cache::Cache::PutResponse"
    ":wat::cache::Cache::GetResult"
    ":wat::cache::Cache::Op"
    ":wat::cache::Cache::Reply"
    ":wat::cache::lru-svc::Admin"
    ":wat::cache::lru-svc::Status"
    ":wat::query::Store::PutResponse"
    ":wat::query::Store::Reply"
    ":wat::telemetry::Journal::WriteMetricsResponse"
    ":wat::telemetry::Journal::Reply"
    ":wat::kernel::StdIn::ReadFrameResponse"
    ":wat::kernel::ReadFrameOutcome"))

(:wat::core::defn :user::fill-paths
  [m     <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
   epaths <- (:wat::core::Vector :- [:wat::core::String])
   decls  <- (:wat::core::Vector :- [:wat::WatAST])]
  -> (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn
      [acc <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
       ep  <- :wat::core::String]
      -> (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
      (:wat::core::let [seen (:wat::string::concat "#seen#" ep)]
        (:wat::core::match (:wat::hashmap::get acc seen)
          [:wat::core::Option.Some {:value _} acc]
          [:wat::core::Option.None {}
            (:wat::core::let [acc2 (:wat::hashmap::assoc acc seen (:wat::core::Vector :- [:wat::core::String]))]
              (:wat::core::match (:user::try-type-of ep decls)
                [:wat::core::Option.Some {:value info} (:user::fill-enum acc2 info ep)]
                [:wat::core::Option.None {} acc2]
                [_ acc2]))]
          [_ acc])))
    m epaths))

(:wat::core::defn :user::stdlib-fmap []
  -> (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
  (:user::fill-paths
    (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
    (:user::seed-paths)
    (:wat::core::Vector :- [:wat::WatAST])))

;; Not `is_reserved_prefix`. RESERVED_PREFIXES (src/resolve/reserved.rs:14) is
;; `:wat::`, `:rust::`, AND `:$bound::`. The third is binder unforgeability —
;; per-scope, not corpus-invariant. Caching `$bound` across files is option-A
;; dishonesty. `gate` (src/resolve/registration.rs:165) checks
;; Existing::Equivalent → NoOp BEFORE Reserved (tests at :317-319), so a user
;; re-declaration of a `:wat::`/`:rust::` name is either the same answer or
;; refused. That is why these two prefixes, and only these two, are cacheable.
(:wat::core::defn :user::corpus-invariant-name? [nm <- :wat::core::String] -> :wat::core::bool
  (:wat::core::or
    (:wat::string::starts-with? nm ":wat::")
    (:wat::string::starts-with? nm ":rust::")))

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

;; (c) — a local path is worth try-type-of only if THIS FILE could declare it.
;; Macro-shaped decls generate types from their arguments; when one is present
;; keep every local path (STOP-1). Otherwise keep iff the path equals, or is
;; prefixed by on a `::` / `.` boundary, a plain type decl's name. Filter in
;; place (STOP-2): do not reorder. try-type-of only shrinks decls, and the
;; empty retry cannot resolve a local (stdlib's non-reserved decls are all
;; ~placeholders in macro templates).

(:wat::core::defn :user::macro-decl-head? [h <- :wat::core::String] -> :wat::core::bool
  (:wat::core::or
    (:wat::core::= h ":wat::service::defservice")
    (:wat::core::or
      (:wat::core::= h ":wat::query::sift-rules-defsvc")
      (:wat::core::= h ":wat::core::defmacro"))))

(:wat::core::defn :user::plain-type-head? [h <- :wat::core::String] -> :wat::core::bool
  (:wat::core::or
    (:wat::core::= h ":wat::core::defenum")
    (:wat::core::or
      (:wat::core::= h ":wat::core::defrecord")
      (:wat::core::or
        (:wat::core::= h ":wat::core::defstruct")
        (:wat::core::or
          (:wat::core::= h ":wat::core::defsurface")
          (:wat::core::or
            (:wat::core::= h ":wat::core::newtype")
            (:wat::core::or
              (:wat::core::= h ":wat::core::typealias")
              (:wat::core::= h ":wat::core::typeunion"))))))))

(:wat::core::defn :user::decl-type-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::let [ch (:wat::core::ast->children n)]
    (:wat::core::if (:wat::core::< (:wat::core::length ch) 2)
      ""
      (:wat::core::let [a (:wat::core::Option/expect (:wat::core::get ch 1) "decl name")]
        (:wat::core::if (:wat::core::= (:wat::core::ast-kind a) "keyword")
          (:wat::core::ast-name a)
          "")))))

(:wat::core::defn :user::has-macro-decl?
  [decls <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::bool d <- :wat::WatAST] -> :wat::core::bool
      (:wat::core::or acc (:user::macro-decl-head? (:wat::fix::head-name d))))
    false
    decls))

(:wat::core::defn :user::plain-type-names
  [decls <- (:wat::core::Vector :- [:wat::WatAST])]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) d <- :wat::WatAST]
      -> (:wat::core::Vector :- [:wat::core::String])
      (:wat::core::if (:user::plain-type-head? (:wat::fix::head-name d))
        (:wat::core::let [nm (:user::decl-type-name d)]
          (:wat::core::if (:wat::core::= nm "") acc (:wat::core::conj acc nm)))
        acc))
    (:wat::core::Vector :- [:wat::core::String])
    decls))

(:wat::core::defn :user::under-name? [ep <- :wat::core::String nm <- :wat::core::String] -> :wat::core::bool
  (:wat::core::or
    (:wat::core::= ep nm)
    (:wat::core::or
      (:wat::string::starts-with? ep (:wat::string::concat nm "::"))
      (:wat::string::starts-with? ep (:wat::string::concat nm ".")))))

(:wat::core::defn :user::under-any-plain?
  [ep    <- :wat::core::String
   names <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::bool nm <- :wat::core::String] -> :wat::core::bool
      (:wat::core::or acc (:user::under-name? ep nm)))
    false
    names))

(:wat::core::defn :user::keep-local-ep? [decls <- (:wat::core::Vector :- [:wat::WatAST]) ep <- :wat::core::String] -> :wat::core::bool
  (:wat::core::if (:user::has-macro-decl? decls)
    true
    (:user::under-any-plain? ep (:user::plain-type-names decls))))

;; Filter in place — same relative order as `local`. Never reorder.
(:wat::core::defn :user::keep-local-eps
  [decls <- (:wat::core::Vector :- [:wat::WatAST])
   local <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) ep <- :wat::core::String]
      -> (:wat::core::Vector :- [:wat::core::String])
      (:wat::core::if (:user::keep-local-ep? decls ep)
        (:wat::core::conj acc ep)
        acc))
    (:wat::core::Vector :- [:wat::core::String])
    local))

(:wat::core::defn :user::skip-local-eps
  [decls <- (:wat::core::Vector :- [:wat::WatAST])
   local <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) ep <- :wat::core::String]
      -> (:wat::core::Vector :- [:wat::core::String])
      (:wat::core::if (:user::keep-local-ep? decls ep)
        acc
        (:wat::core::conj acc ep)))
    (:wat::core::Vector :- [:wat::core::String])
    local))

(:wat::core::defn :user::audit-skipped
  [decls   <- (:wat::core::Vector :- [:wat::WatAST])
   skipped <- (:wat::core::Vector :- [:wat::core::String])
   path    <- :wat::core::String]
  -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? skipped)
    nil
    (:wat::core::let [ep (:wat::core::first skipped)]
      (:wat::core::match (:user::try-type-of ep decls)
        [:wat::core::Option.Some {:value _}
          (:wat::kernel::assertion-failed!
            :message
            (:wat::string::concat
              "STOP-6 skipped-that-resolved-to-Some "
              (:wat::string::concat ep (:wat::string::concat " " path))))]
        [:wat::core::Option.None {}
          (:user::audit-skipped decls (:wat::core::rest skipped) path)]
        [_ (:user::audit-skipped decls (:wat::core::rest skipped) path)]))))

(:wat::core::defn :user::fmap-for-src
  [src  <- :wat::core::String
   base <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])]
  -> (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
  (:wat::core::let
    [tree (:wat::core::match (:wat::core::read-string src)
             [:wat::core::ReadOutcome.Forms {:forms f} f]
             [:wat::core::ReadOutcome.Malformed {:cause c}
               (:wat::kernel::assertion-failed! :message (:wat::core::Error/message c))])
     decls (:user::file-decls tree)
     vpaths (:user::collect-keywords (:wat::core::Vector :- [:wat::core::String]) tree)
     epaths (:user::enum-paths-of vpaths)
     local  (:user::local-eps-of epaths)
     kept   (:user::keep-local-eps decls local)
     filled (:user::fill-paths base kept decls)]
    (:user::bind-kw-ctors filled tree)))

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
   fmap <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])]
  -> (:wat::core::Option :- [(:wat::core::Vector :- [:wat::core::String])])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind head) "keyword")
    (:wat::core::let [nm (:wat::core::ast-name head)]
      (:wat::core::if (:user::has-slash? nm)
        :wat::core::Option.None
        (:wat::hashmap::get fmap nm)))
    (:wat::core::if (:user::unquote-form? head)
      (:wat::core::match (:user::unquote-ctor-leaf (:user::unquote-inner-name head))
        [:wat::core::Option.Some {:value leaf} (:wat::hashmap::get fmap leaf)]
        [:wat::core::Option.None {} :wat::core::Option.None]
        [_ :wat::core::Option.None])
      :wat::core::Option.None)))

(:wat::core::defn :user::ctor-edits
  [node  <- :wat::WatAST
   fmap  <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
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
                    (:wat::core::if (:wat::core::or (:user::unquote-form? head)
                                      (:wat::core::if (:wat::core::= (:wat::core::ast-kind head) "keyword")
                                        (:user::pascal-leaf? (:wat::core::ast-name head))
                                        false))
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
   fmap  <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
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
   fmap  <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
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
      (:user::ctor-edits node fmap src lines path))
    (:wat::core::if (:wat::fix::structural? node)
      (:user::walk-seq (:wat::core::ast->children node) fmap src lines path)
      (:wat::core::Vector :- [:wat::fix::Edit]))))

(:wat::core::defn :user::walk-seq
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   fmap  <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
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
   fmap <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
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
   frozen <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])]
  -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::if (:user::skip-path? path)
        (:wat::core::do
          (:wat::kernel::println (:wat::string::concat "[positional-ctor] skip positional-control " path))
          (:user::rewrite-each (:wat::core::rest paths) frozen))
        (:wat::core::let [src (:wat::io::read-file path)
                          fmap (:user::fmap-for-src src frozen)]
          (:wat::core::do
            (:wat::io::write-file path (:user::migrate src fmap path))
            (:wat::kernel::println (:wat::string::concat "[positional-ctor] " path))
            (:user::rewrite-each (:wat::core::rest paths) frozen)))))))

;; PASS 1: unique corpus-invariant epaths, resolved once against stdlib
;; (empty decls). PASS 2: kept locals against THIS file's decls (fmap-for-src).
(:wat::core::defn :user::collect-pass
  [scan    <- (:wat::core::Vector :- [:wat::core::String])
   all     <- (:wat::core::Vector :- [:wat::core::String])
   inv-eps <- (:wat::core::Vector :- [:wat::core::String])
   kept-n  <- :wat::core::i64
   skip-n  <- :wat::core::i64
   audit   <- :wat::core::bool
   base    <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])]
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
      (:user::rewrite-each all
        (:user::fill-paths base inv-eps (:wat::core::Vector :- [:wat::WatAST]))))
    (:wat::core::let [path (:wat::core::first scan)]
      (:wat::core::if (:user::skip-path? path)
        (:user::collect-pass (:wat::core::rest scan) all inv-eps kept-n skip-n audit base)
        (:wat::core::let
          [src    (:wat::io::read-file path)
           tree   (:user::src-tree src)
           decls  (:user::file-decls tree)
           vpaths (:user::collect-keywords (:wat::core::Vector :- [:wat::core::String]) tree)
           feps   (:user::enum-paths-of vpaths)
           inv2   (:user::union-eps inv-eps (:user::invariant-eps-of feps))
           local  (:user::local-eps-of feps)
           kept   (:user::keep-local-eps decls local)
           skipped (:user::skip-local-eps decls local)]
          (:wat::core::do
            (:wat::core::if audit
              (:user::audit-skipped decls skipped path)
              nil)
            (:user::collect-pass
              (:wat::core::rest scan)
              all
              inv2
              (:wat::i64::+ kept-n (:wat::core::length kept))
              (:wat::i64::+ skip-n (:wat::core::length skipped))
              audit
              base)))))))

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
     base (:user::stdlib-fmap)]
    (:user::collect-pass
      paths2
      paths2
      (:wat::core::Vector :- [:wat::core::String])
      0
      0
      audit
      base)))
