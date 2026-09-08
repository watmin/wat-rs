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
    [:wat::core::ReadOutcome::Forms {:forms f}
      (:wat::core::first (:wat::core::ast->children f))]
    [:wat::core::ReadOutcome::Malformed {:cause c}
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
    (:wat::core::Some {:value ":wat::core::Option"})
    (:wat::core::if (:wat::core::or (:wat::core::= leaf "Ok") (:wat::core::= leaf "Err"))
      (:wat::core::Some {:value ":wat::core::Result"})
      :wat::core::None)))

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

(:wat::core::defn :user::try-type-of
  [enum-path <- :wat::core::String
   decls     <- (:wat::core::Vector :- [:wat::WatAST])]
  -> (:wat::core::Option :- [:wat::runtime::TypeInfo])
  (:wat::core::match
    (:wat::eval-with-defs! :- [:wat::runtime::TypeInfo]
      (:user::type-of-form enum-path) decls)
    [:wat::eval::FormOutcome::Evaluated {:value v}
      (:wat::core::match (:wat::runtime::TypeInfo/kind v)
        [:wat::runtime::TypeKind::Enum {} (:wat::core::Some {:value v})]
        [_ :wat::core::None])]
    [_ (:wat::core::if (:wat::core::empty? decls)
         :wat::core::None
         (:user::try-type-of enum-path (:wat::core::Vector :- [:wat::WatAST])))]))

(:wat::core::defn :user::report [msg <- :wat::core::String] -> :wat::core::nil
  (:wat::kernel::println (:wat::string::concat "[positional-ctor] UNRESOLVED " msg)))

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
    [:wat::core::None {} (:wat::hashmap::assoc m leaf fields)]
    [:wat::core::Some {:value existing}
      (:wat::core::if (:user::names-eq? existing fields)
        m
        m)]))

(:wat::core::defn :user::fill-enum
  [m         <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
   info      <- :wat::runtime::TypeInfo
   enum-path <- :wat::core::String]
  -> (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
  (:wat::core::match (:wat::runtime::TypeInfo/body info)
    [:wat::runtime::TypeBody::Enum {:purity _ :variants vs}
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
             acc3   (:user::index-leaf acc2 leaf fields)]
            acc3))
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
          [:wat::core::Some {:value ep} (:user::conj-unique acc3 ep)]
          [:wat::core::None {} acc3])))
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
          [:wat::core::Some {:value _} acc]
          [:wat::core::None {}
            (:wat::core::let [acc2 (:wat::hashmap::assoc acc seen (:wat::core::Vector :- [:wat::core::String]))]
              (:wat::core::match (:user::try-type-of ep decls)
                [:wat::core::Some {:value info} (:user::fill-enum acc2 info ep)]
                [:wat::core::None {} acc2]))])))
    m epaths))

(:wat::core::defn :user::stdlib-fmap []
  -> (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
  (:user::fill-paths
    (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
    (:user::seed-paths)
    (:wat::core::Vector :- [:wat::WatAST])))

(:wat::core::defn :user::fmap-for-src
  [src  <- :wat::core::String
   base <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])]
  -> (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
  (:wat::core::let
    [tree (:wat::core::match (:wat::core::read-string src)
             [:wat::core::ReadOutcome::Forms {:forms f} f]
             [:wat::core::ReadOutcome::Malformed {:cause c}
               (:wat::kernel::assertion-failed! :message (:wat::core::Error/message c))])
     decls (:user::file-decls tree)
     vpaths (:user::collect-keywords (:wat::core::Vector :- [:wat::core::String]) tree)
     epaths (:user::enum-paths-of vpaths)]
    (:user::fill-paths base epaths decls)))

;; ── gensym → variant leaf (defservice templates in wat/service.wat) ──────────
;; The FQDN is interpolated at expand time; the VARIANT leaf is LAW. Field names
;; still come from type-of (indexed by leaf from every *Response we asked).

(:wat::core::defn :user::unquote-ctor-leaf [inner <- :wat::core::String] -> (:wat::core::Option :- [:wat::core::String])
  (:wat::core::if (:wat::core::= inner "rtl-ctor-kw")
    (:wat::core::Some {:value ":wat::cache::Cache::GetResponse::RequestTooLarge"})
    (:wat::core::if (:wat::core::= inner "rm-ctor-kw")
      (:wat::core::Some {:value ":wat::cache::Cache::GetResponse::RequestMalformed"})
      (:wat::core::if (:wat::core::= inner "reply-variant-kw")
        (:wat::core::Some {:value ":wat::cache::Cache::Reply::Get"})
        (:wat::core::if (:wat::core::= inner "op-variant-kw")
          (:wat::core::Some {:value ":wat::cache::Cache::Op::Get"})
          (:wat::core::if (:wat::core::= inner "reply-failed-kw")
            (:wat::core::Some {:value ":wat::cache::Cache::Reply::Failed"})
            (:wat::core::if (:wat::core::= inner "admin-allow-peer-kw")
              (:wat::core::Some {:value ":wat::cache::lru-svc::Admin::AllowPeer"})
              (:wat::core::if (:wat::core::= inner "admin-deny-peer-kw")
                (:wat::core::Some {:value ":wat::cache::lru-svc::Admin::DenyPeer"})
                (:wat::core::if (:wat::core::= inner "status-stopped-kw")
                  (:wat::core::Some {:value ":wat::cache::lru-svc::Status::Stopped"})
                  (:wat::core::if (:wat::core::= inner "status-hibernated-kw")
                    (:wat::core::Some {:value ":wat::cache::lru-svc::Status::Hibernated"})
                    (:wat::core::if (:wat::core::= inner "status-started-kw")
                      (:wat::core::Some {:value ":wat::cache::lru-svc::Status::Started"})
                      :wat::core::None)))))))))))

(:wat::core::defn :user::unquote-form? [n <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::= (:wat::fix::head-name n) ":wat::core::unquote"))

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
    (:wat::core::= nm ":wat::core::Some")
    (:wat::core::or
      (:wat::core::= nm ":wat::core::None")
      (:wat::core::or
        (:wat::core::= nm ":wat::core::Ok")
        (:wat::core::= nm ":wat::core::Err")))))

(:wat::core::defn :user::map-value-node [mapn <- :wat::WatAST] -> (:wat::core::Option :- [:wat::WatAST])
  (:wat::core::let [ch (:wat::core::ast->children mapn)
                    n  (:wat::core::length ch)]
    (:wat::core::if (:wat::core::= n 2)
      (:wat::core::get ch 1)
      :wat::core::None)))

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
        [:wat::core::Some {:value v}
          (:wat::core::Vector :- [:wat::fix::Edit]
            (:wat::core::Tuple
              (:wat::fix::node-start-offset m lines)
              (:user::node-text m src lines)
              (:user::node-text v src lines)))]
        [:wat::core::None {}
          ;; unit None: `(:wat::core::None {})` → drop the map, leave `(:wat::core::None)`
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
    (:wat::core::if (:wat::core::= (:wat::core::ast-kind (:wat::core::first args)) "symbol")
      (:wat::core::= (:wat::core::ast-name (:wat::core::first args)) ":-")
      false)))

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
  (:wat::core::if (:user::unquote-form? arg)
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
        :wat::core::None
        (:wat::hashmap::get fmap nm)))
    (:wat::core::if (:user::unquote-form? head)
      (:wat::core::match (:user::unquote-ctor-leaf (:user::unquote-inner-name head))
        [:wat::core::Some {:value leaf} (:wat::hashmap::get fmap leaf)]
        [:wat::core::None {} :wat::core::None])
      :wat::core::None)))

(:wat::core::defn :user::ctor-edits
  [node  <- :wat::WatAST
   fmap  <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:wat::core::not (:wat::core::= (:wat::core::ast-kind node) "list"))
    (:wat::core::Vector :- [:wat::fix::Edit])
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch)
        (:wat::core::Vector :- [:wat::fix::Edit])
        (:wat::core::let
          [head (:wat::core::first ch)
           args (:wat::core::into [] (:wat::core::rest ch))]
          (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind head) "keyword")
                            (:user::alias-ctor-name? (:wat::core::ast-name head))
                            false)
            (:wat::core::concat
              (:user::unwrap-alias-edits node args src lines)
              (:user::walk-seq args fmap src lines))
          (:wat::core::if (:user::type-app? args)
            (:user::walk-seq (:wat::core::into [] (:wat::core::drop args 1)) fmap src lines)
            (:wat::core::if (:user::already-map? args)
              (:user::walk-seq args fmap src lines)
              (:wat::core::match (:user::fields-for-head head fmap)
                [:wat::core::None {}
                  (:wat::core::concat
                    (:user::walk-edits head fmap src lines)
                    (:user::walk-seq args fmap src lines))]
                [:wat::core::Some {:value fields}
                  (:wat::core::if (:wat::core::= (:wat::core::length args) (:wat::core::length fields))
                    (:wat::core::concat
                      (:wat::core::if (:wat::core::empty? fields)
                        (:user::unit-edits node lines)
                        (:user::tagged-edits args fields lines))
                      (:user::walk-seq args fmap src lines))
                    (:wat::core::do
                      (:user::report
                        (:wat::core::if (:wat::core::= (:wat::core::ast-kind head) "keyword")
                          (:wat::core::ast-name head)
                          (:user::node-text head src lines)))
                      (:user::walk-seq args fmap src lines)))])))))))))

(:wat::core::defn :user::walk-edits
  [node  <- :wat::WatAST
   fmap  <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:user::ctor-edits node fmap src lines)
    (:wat::core::if (:wat::fix::structural? node)
      (:user::walk-seq (:wat::core::ast->children node) fmap src lines)
      (:wat::core::Vector :- [:wat::fix::Edit]))))

(:wat::core::defn :user::walk-seq
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   fmap  <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :- [:wat::fix::Edit])
    (:wat::core::concat
      (:user::walk-edits (:wat::core::first items) fmap src lines)
      (:user::walk-seq (:wat::core::rest items) fmap src lines))))

(:wat::core::defn :user::migrate
  [src  <- :wat::core::String
   fmap <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])]
  -> :wat::core::String
  (:wat::core::let
    [lines (:wat::string::split src "\n")
     tree  (:wat::core::match (:wat::core::read-string src)
             [:wat::core::ReadOutcome::Forms {:forms __forms} __forms]
             [:wat::core::ReadOutcome::Malformed {:cause __cause}
               (:wat::kernel::assertion-failed! :message (:wat::core::Error/message __cause))])
     edits (:user::walk-seq (:wat::core::ast->children tree) fmap src lines)]
    (:wat::fix::fix-text-apply src (:wat::core::reverse (:wat::core::sort edits)))))

(:wat::core::defn :user::skip-path? [path <- :wat::core::String] -> :wat::core::bool
  (:wat::core::= path "tests/types/probe_arc296_enum_map_ctor__positional.wat"))

(:wat::core::defn :user::rewrite-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])
   base  <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])]
  -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::if (:user::skip-path? path)
        (:wat::core::do
          (:wat::kernel::println (:wat::string::concat "[positional-ctor] skip positional-control " path))
          (:user::rewrite-each (:wat::core::rest paths) base))
        (:wat::core::let [src (:wat::io::read-file path)
                          fmap (:user::fmap-for-src src base)]
          (:wat::core::do
            (:wat::io::write-file path (:user::migrate src fmap))
            (:wat::kernel::println (:wat::string::concat "[positional-ctor] " path))
            (:user::rewrite-each (:wat::core::rest paths) fmap)))))))

(:wat::core::defn :user::ensure-path
  [paths <- (:wat::core::Vector :- [:wat::core::String])
   extra <- :wat::core::String]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::if (:wat::vec::contains? paths extra)
    paths
    (:wat::core::conj paths extra)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [paths (:wat::core::match (:wat::kernel::readln)
             [:wat::kernel::ReadlnOutcome::Datum {:v __datum} __datum]
             [:wat::kernel::ReadlnOutcome::Eof {}
               (:wat::kernel::assertion-failed! :message "readln: end of input")]
             [:wat::kernel::ReadlnOutcome::Stopped {}
               (:wat::kernel::assertion-failed! :message "readln: stop requested")])
     paths2 (:user::ensure-path paths "wat/service.wat")
     base (:user::stdlib-fmap)]
    (:user::rewrite-each paths2 base)))
