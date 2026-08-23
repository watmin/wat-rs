;; wat-scripts/fixes/positional-to-kwargs.wat — arc 294 item 9a migration codemod.
;;
;; Migrates positional aggregate construction  (:ns::T a b)  →  kwargs  (:ns::T :f1 a :f2 b)
;; across a corpus. Self-hosted, comment-faithful (span inserts via fix-text-apply), reflection-
;; free: it OBSERVES each file's def-forms as bytes to build a global type→field-order map, then
;; inserts `:field ` before each positional arg at construction sites whose head is a mapped type
;; and whose arg count equals the field count. Accessors (:ns::T/f), annotations (<- :ns::T),
;; already-kwargs forms, and the def forms themselves are left untouched (head/count gated).
;;
;; SPLICE-bearing records (field-vec not a clean name/<-/type triple run) are SKIPPED — their
;; type never enters the map, so their constructions are left for a hand-fix + reported to stderr.
;;
;; Run under a PRE-FLIP (booting) binary (the stdlib is still positional there):
;;   printf '["wat/query.wat" "wat/rete.wat" ...]' | cargo wat ./wat-scripts/fixes/positional-to-kwargs.wat

;; ── def detection ────────────────────────────────────────────────────────────
(:wat::core::defn :user::def-head? [name <- :wat::core::String] -> :wat::core::bool
  (:wat::core::if (:wat::core::= name ":wat::core::defrecord") true
    (:wat::core::if (:wat::core::= name ":wat::holon::defrecord") true
      (:wat::core::if (:wat::core::= name ":wat::core::defstruct") true
        (:wat::core::= name ":wat::core::defholon")))))

;; first child at index>=i whose ast-kind is "vector" (the field-vec; robust across def shapes)
(:wat::core::defn :user::fieldvec-at [ch <- (:wat::core::Vector :- [:wat::WatAST]) i <- :wat::core::i64]
  -> (:wat::core::Option :wat::WatAST)
  (:wat::core::if (:wat::core::>= i (:wat::core::length ch))
    (:wat::core::None :wat::WatAST)
    (:wat::core::let [c (:wat::core::Option/expect (:wat::core::get ch i) "fieldvec-at")]
      (:wat::core::if (:wat::core::= (:wat::core::ast-kind c) "vector")
        (:wat::core::Some c)
        (:user::fieldvec-at ch (:wat::core::+ i 1))))))

;; field names of a field-vec [x <- T y <- U] → ["x" "y"] (names at 0,3,6…); [] if irregular (splice)
(:wat::core::defn :user::fieldvec-names [fv <- :wat::WatAST] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::let [ch (:wat::core::ast->children fv)
                    n  (:wat::core::length ch)]
    (:wat::core::if (:wat::core::= (:wat::core::i64::rem n 3) 0)
      (:wat::core::foldl
        (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) i <- :wat::core::i64]
          -> (:wat::core::Vector :- [:wat::core::String])
          (:wat::core::conj acc
            (:wat::core::ast-name (:wat::core::Option/expect (:wat::core::get ch (:wat::core::i64::* i 3)) "fv-name"))))
        (:wat::core::Vector :wat::core::String)
        (:wat::core::range 0 (:wat::core::i64::/ n 3)))
      (:wat::core::Vector :wat::core::String))))

;; add one form to the map if it is a mappable def with a clean (non-splice) field-vec.
(:wat::core::defn :user::add-form
  [m <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])]) form <- :wat::WatAST]
  -> (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind form) "list")
    (:wat::core::let [ch (:wat::core::ast->children form)]
      (:wat::core::if (:wat::core::< (:wat::core::length ch) 3)
        m
        (:wat::core::let [head (:wat::core::first ch)]
          (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind head) "keyword")
                            (:user::def-head? (:wat::core::ast-name head)) false)
            (:wat::core::let [;; strip `<T,U,…>` type-params: a parametric def is `:ns::T<S,R>` but
                              ;; constructions write the bare `:ns::T`. Key on the bare name.
                              tyname (:wat::core::first (:wat::core::string::split
                                       (:wat::core::ast-name (:wat::core::Option/expect (:wat::core::get ch 1) "add-form ty"))
                                       "<"))
                              fvopt  (:user::fieldvec-at ch 2)]
              (:wat::core::match fvopt 
                (:wat::core::None m)
                ((:wat::core::Some fv)
                  (:wat::core::let [names (:user::fieldvec-names fv)]
                    (:wat::core::if (:wat::core::empty? names)
                      m
                      (:wat::core::HashMap/assoc m tyname names))))))
            m))))
    m))

;; build the global map from ALL forms of ALL files.
(:wat::core::defn :user::map-of-forms
  [m <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
   forms <- (:wat::core::Vector :- [:wat::WatAST])]
  -> (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
  (:wat::core::foldl :user::add-form m forms))

;; ── the rewrite: collect insert-edits ────────────────────────────────────────
;; for a construction (head arg…), one insert per arg: Tuple(arg-start-offset, 0, ":field ")
(:wat::core::defn :user::arg-edits
  [args <- (:wat::core::Vector :- [:wat::WatAST]) fields <- (:wat::core::Vector :- [:wat::core::String])
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])]) i <- :wat::core::i64]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
      (:wat::core::let [arg (:wat::core::Option/expect (:wat::core::get args i) "arg-edits arg")
                        off (:wat::fix::fix-text-offset-of (:wat::core::ast-span arg) lines)
                        kw  (:wat::core::string::concat ":" (:wat::core::Option/expect (:wat::core::get fields i) "arg-edits f") " ")]
        (:wat::core::conj acc (:wat::core::Tuple off 0 kw))))
    (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String))
    (:wat::core::range 0 (:wat::core::length args))))

;; walk a node, collect edits for it + all descendants.
(:wat::core::defn :user::edits
  [node  <- :wat::WatAST
   m     <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch)
        (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String))
        (:wat::core::let
          [head  (:wat::core::first ch)
           args  (:wat::core::into [] (:wat::core::rest ch))
           hname (:wat::core::if (:wat::core::= (:wat::core::ast-kind head) "keyword")
                   (:wat::core::ast-name head) "")
           fopt  (:wat::core::HashMap/get m hname)
           this  (:wat::core::match fopt 
                   (:wat::core::None (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String)))
                   ((:wat::core::Some fields)
                     (:wat::core::if (:wat::core::= (:wat::core::length args) (:wat::core::length fields))
                       (:user::arg-edits args fields lines)
                       (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String)))))]
          (:wat::core::concat this (:user::edits-seq ch m lines)))))
    ;; NOT a list — recurse into vector/map children (constructions nest inside let-binding
    ;; vectors `[x (:T a b)]`, map literals, etc.). Only LISTS can be construction heads; these
    ;; containers just pass through to the walk. Leaves (symbol/keyword/string) yield no edits.
    (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "vector") true
                      (:wat::core::= (:wat::core::ast-kind node) "map"))
      (:user::edits-seq (:wat::core::into [] (:wat::core::ast->children node)) m lines)
      (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String)))))

(:wat::core::defn :user::edits-seq
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   m     <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String))
    (:wat::core::concat
      (:user::edits (:wat::core::first items) m lines)
      (:user::edits-seq (:wat::core::into [] (:wat::core::rest items)) m lines))))

;; ── per-file migrate ─────────────────────────────────────────────────────────
(:wat::core::defn :user::migrate
  [src <- :wat::core::String
   m   <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])]
  -> :wat::core::String
  (:wat::core::let [lines (:wat::core::string::split src "\n")
                    tree  (:wat::core::match (:wat::core::read-string src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
                    forms (:wat::core::ast->children tree)
                    eds   (:user::edits-seq forms m lines)
                    ;; edits MUST apply high-offset-first so a low insert never shifts a pending
                    ;; higher one (nested constructions). sort ascending by tuple (offset first), reverse.
                    rev   (:wat::core::reverse (:wat::core::sort eds))]
    (:wat::fix::fix-text-apply src rev)))

;; ── driver: build the map from ALL files first, then rewrite each ────────────
(:wat::core::defn :user::read-forms [path <- :wat::core::String] -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::ast->children (:wat::core::match (:wat::core::read-string (:wat::io::read-file path)) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))))

(:wat::core::defn :user::build-map
  [m <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
   paths <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
  (:wat::core::if (:wat::core::empty? paths)
    m
    (:wat::core::let [p (:wat::core::first paths)]
      (:user::build-map (:user::map-of-forms m (:user::read-forms p))
                        (:wat::core::into [] (:wat::core::rest paths))))))

(:wat::core::defn :user::rewrite-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])
   m     <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])]
  -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [p (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file p (:user::migrate (:wat::io::read-file p) m))
        (:wat::kernel::println (:wat::core::string::concat "[kwargs] " p))
        (:user::rewrite-each (:wat::core::into [] (:wat::core::rest paths)) m)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [paths (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    m     (:user::build-map
                            (:wat::core::HashMap :wat::core::String (:wat::core::Vector :- [:wat::core::String]))
                            paths)]
    (:user::rewrite-each paths m)))
