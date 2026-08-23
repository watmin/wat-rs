;; wat/deporder.wat — the stdlib load-order analyzer.
;;
;; Arc 275 Stone 275.1. A pure-wat tool: given an ordered list of
;; SourceFile{path,source} pairs, parses each file's top-level forms,
;; builds a symbol→(file,kind) map, classifies cross-file references
;; (defmacro = order-free; defn/defenum/defalias/def/defprotocol/
;; defclause/typealias/defstruct/newtype/extend-type/derive = eval-dep),
;; and returns the Violations where a file eval-depends on a later-loaded
;; file.
;;
;; The surface:
;;   (:wat::deporder::verify-stdlib) — runs against the real baked order.
;;   (:wat::deporder::verify files)  — pure function, no I/O.
;;
;; Namespace: :wat::deporder:: (English register, matching fix.wat's
;; precedent as a domain-noun-named wat dev tool).
;;
;; Worked references:
;;   wat/fix.wat — structural? + recursive AST walk
;;   wat/service.wat — HashMap foldl+range+get idiom
;;   wat/Record.wat — typed record definition

;; ─── Typed records (uncompilable on a wrong shape) ────────────────────
;; The source-unit `:wat::source::File` lifted to wat/source.wat (arc 283) — shared by every tool.

(:wat::core::defrecord :wat::deporder::SymDef
  [file <- :wat::core::String
   kind <- :wat::core::String])

(:wat::core::defrecord :wat::deporder::Violation
  [referencer     <- :wat::core::String
   referencer-pos <- :wat::core::i64
   definer        <- :wat::core::String
   definer-pos    <- :wat::core::i64
   symbol         <- :wat::core::String])

;; ─── Predicate helpers ────────────────────────────────────────────────

;; structural? — a node whose children we recurse into (list/vector/set/map).
;; Mirror of fix.wat's structural? (same predicate, deporder namespace).
;; Set membership replaces the 4-deep if/= ladder.
(:wat::core::defn :wat::deporder::structural?
  [node <- :wat::WatAST]
  -> :wat::core::bool
  (:wat::core::let [k       (:wat::core::ast-kind node)
                    kinds   (:wat::core::HashSet :wat::core::String
                              "list" "vector" "map" "set")]
    (:wat::core::contains? kinds k)))

;; qual-keyword? — a ::-namespaced keyword (any namespace).
;; These are the cross-file references we care about. We match any
;; keyword with "::" in its name — data keywords like `:else` have
;; no "::" and are left alone. In the real stdlib all defs are in
;; :wat:: namespaces, so this broadens the predicate to cover test
;; fixtures that use short namespaces like :t:: .
(:wat::core::defn :wat::deporder::qual-keyword?
  [node <- :wat::WatAST]
  -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "keyword")
    (:wat::core::string::contains? (:wat::core::ast-name node) "::")
    false))

;; is-def-head? — true if the ast-name of the form head is a recognized
;; definition form head. We recognize ALL def-heads present in the real
;; stdlib to avoid false STOP-2 triggers.
;; Set membership replaces the 13-deep if/= ladder.
(:wat::core::defn :wat::deporder::is-def-head?
  [nm <- :wat::core::String]
  -> :wat::core::bool
  (:wat::core::let [heads (:wat::core::HashSet :wat::core::String
                             ":wat::core::defn"
                             ":wat::core::defmacro"
                             ":wat::core::defenum"
                             ":wat::core::defalias"
                             ":wat::core::def"
                             ":wat::core::defprotocol"
                             ":wat::core::defclause"
                             ":wat::core::typealias"
                             ":wat::core::defstruct"
                             ;; Arc 293.2-parity — structtype is the low-level primitive defstruct (macro) expands to.
                             ":wat::core::structtype"
                             ":wat::core::newtype"
                             ;; NOTE: :wat::core::extend-type is intentionally NOT a def-head.
                             ;; Its child[1] is the type being EXTENDED (a REFERENCE, defined by
                             ;; defstruct/defrecord elsewhere), not a def-site. Treating it as a
                             ;; def-head recorded a phantom def-site and mis-flagged cross-file
                             ;; extend-types (target in an earlier-loading file) as forward-refs.
                             ;; Omitting it makes extend-type a pure consumer: def-form? is false,
                             ;; so collect-form-refs gathers ALL children (target + surface + body)
                             ;; as references and no def-site is recorded.
                             ":wat::core::derive"
                             ":wat::core::recordtype"
                             ;; Arc 293 decl-a — ONE type-reg primitive; nature derived from parent root.
                             ":wat::core::aggregatetype")]
    (:wat::core::contains? heads nm)))

;; def-head-kind — "defmacro" if the head is defmacro (order-free);
;; "eval-dep" for all other recognized def-heads.
(:wat::core::defn :wat::deporder::def-head-kind
  [nm <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::if (:wat::core::= nm ":wat::core::defmacro") "defmacro" "eval-dep"))

;; def-form? — true if the top-level form is a definition form
;; (a list whose head ast-name is a recognized def-head).
(:wat::core::defn :wat::deporder::def-form?
  [form <- :wat::WatAST]
  -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind form) "list")
    (:wat::core::let [ch (:wat::core::ast->children form)]
      (:wat::core::if (:wat::core::empty? ch)
        false
        (:wat::core::let [head (:wat::core::first ch)]
          (:wat::deporder::is-def-head? (:wat::core::ast-name head)))))
    false))

;; defined-name — the ast-name of child[1] of a definition form.
;; This is the symbol being defined; it must NOT be counted as a reference.
(:wat::core::defn :wat::deporder::defined-name
  [form <- :wat::WatAST]
  -> :wat::core::String
  (:wat::core::let [ch   (:wat::core::ast->children form)
                    name-node (:wat::core::nth ch 1)]
    (:wat::core::ast-name name-node)))

;; ─── Keyword reference collector ──────────────────────────────────────

;; collect-kwds — recursively walk a node, collecting the ast-name of
;; every ::-namespaced :wat:: keyword node found anywhere in the tree.
;; Returns Vector<String>. Mirrors fix.wat's structural?+recurse pattern.
(:wat::core::defn :wat::deporder::collect-kwds
  [node <- :wat::WatAST]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::if (:wat::deporder::qual-keyword? node)
    (:wat::core::Vector :wat::core::String (:wat::core::ast-name node))
    (:wat::core::if (:wat::deporder::structural? node)
      (:wat::core::foldl
        (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String])
                         child <- :wat::WatAST]
          -> (:wat::core::Vector :- [:wat::core::String])
          (:wat::core::concat acc (:wat::deporder::collect-kwds child)))
        (:wat::core::Vector :wat::core::String)
        (:wat::core::ast->children node))
      (:wat::core::Vector :wat::core::String))))

;; collect-form-refs — collect keyword references from a single form,
;; excluding the defined-name (child[1]) if the form is a definition form.
;; For def-forms: collect from all children EXCEPT child[1].
;; For other forms: collect from all children (via collect-kwds).
(:wat::core::defn :wat::deporder::collect-form-refs
  [form <- :wat::WatAST]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::if (:wat::deporder::def-form? form)
    (:wat::core::let [ch (:wat::core::ast->children form)
                      ;; child[0] = the head keyword (e.g. :wat::core::defn) — collect it
                      head-refs (:wat::core::if (:wat::core::empty? ch)
                                  (:wat::core::Vector :wat::core::String)
                                  (:wat::deporder::collect-kwds
                                    (:wat::core::first ch)))
                      ;; skip child[1] (the defined name); collect from child[2..] (the body)
                      ;; Arc 118.2a — `drop` flipped LAZY; `foldl` below (unchanged) needs it eager.
                      body-ch (:wat::core::into [] (:wat::core::drop ch 2))
                      body-refs (:wat::core::foldl
                                  (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String])
                                                   c <- :wat::WatAST]
                                    -> (:wat::core::Vector :- [:wat::core::String])
                                    (:wat::core::concat acc (:wat::deporder::collect-kwds c)))
                                  (:wat::core::Vector :wat::core::String)
                                  body-ch)]
      (:wat::core::concat head-refs body-refs))
    (:wat::deporder::collect-kwds form)))

;; ─── Pass 1: build the symbol map ─────────────────────────────────────

;; build-file-syms — update sym-map with all definitions from one file.
(:wat::core::defn :wat::deporder::build-file-syms
  [sym-map <- (:wat::core::HashMap :- [:wat::core::String :wat::deporder::SymDef])
   file    <- :wat::source::File]
  -> (:wat::core::HashMap :- [:wat::core::String :wat::deporder::SymDef])
  (:wat::core::let [tree  (:wat::core::match (:wat::core::read-string (:wat::source::File/source file)) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
                    forms (:wat::core::ast->children tree)
                    path  (:wat::source::File/path file)]
    (:wat::core::foldl
      (:wat::core::fn [m    <- (:wat::core::HashMap :- [:wat::core::String :wat::deporder::SymDef])
                       form <- :wat::WatAST]
        -> (:wat::core::HashMap :- [:wat::core::String :wat::deporder::SymDef])
        (:wat::core::if (:wat::deporder::def-form? form)
          (:wat::core::let [dname (:wat::deporder::defined-name form)
                            head-nm (:wat::core::ast-name
                                      (:wat::core::first (:wat::core::ast->children form)))
                            kind  (:wat::deporder::def-head-kind head-nm)]
            (:wat::core::HashMap/assoc m dname (:wat::deporder::SymDef :file path :kind kind)))
          m))
      sym-map
      forms)))

;; build-symbol-map — Pass 1: build the full symbol→SymDef map from all files.
(:wat::core::defn :wat::deporder::build-symbol-map
  [files <- (:wat::core::Vector :- [:wat::source::File])]
  -> (:wat::core::HashMap :- [:wat::core::String :wat::deporder::SymDef])
  (:wat::core::foldl
    :wat::deporder::build-file-syms
    (:wat::core::HashMap :wat::core::String :wat::deporder::SymDef)
    files))

;; ─── Pass 2: detect violations ────────────────────────────────────────

;; build-pos-map — path → position (i64) map for all files.
(:wat::core::defn :wat::deporder::build-pos-map
  [files <- (:wat::core::Vector :- [:wat::source::File])]
  -> (:wat::core::HashMap :- [:wat::core::String :wat::core::i64])
  (:wat::core::let [n (:wat::core::length files)]
    (:wat::core::foldl
      (:wat::core::fn [m <- (:wat::core::HashMap :- [:wat::core::String :wat::core::i64])
                       i <- :wat::core::i64]
        -> (:wat::core::HashMap :- [:wat::core::String :wat::core::i64])
        (:wat::core::let [file (:wat::core::Option/expect  
                                  (:wat::core::get files i) "build-pos-map: get")]
          (:wat::core::HashMap/assoc m (:wat::source::File/path file) i)))
      (:wat::core::HashMap :wat::core::String :wat::core::i64)
      (:wat::core::range 0 n))))

;; check-file-violations — for one file at position ref-pos, collect all
;; keyword refs from all its forms, then emit a Violation for each ref
;; that resolves to a SymDef in a different file loaded AFTER this one
;; (definer-pos > ref-pos), and whose kind is not "defmacro".
(:wat::core::defn :wat::deporder::check-file-violations
  [file    <- :wat::source::File
   ref-pos <- :wat::core::i64
   sym-map <- (:wat::core::HashMap :- [:wat::core::String :wat::deporder::SymDef])
   pos-map <- (:wat::core::HashMap :- [:wat::core::String :wat::core::i64])]
  -> (:wat::core::Vector :- [:wat::deporder::Violation])
  (:wat::core::let [tree  (:wat::core::match (:wat::core::read-string (:wat::source::File/source file)) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
                    forms (:wat::core::ast->children tree)
                    path  (:wat::source::File/path file)
                    ;; collect all keyword refs from all forms in this file
                    all-refs (:wat::core::foldl
                               (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String])
                                                form <- :wat::WatAST]
                                 -> (:wat::core::Vector :- [:wat::core::String])
                                 (:wat::core::concat acc (:wat::deporder::collect-form-refs form)))
                               (:wat::core::Vector :wat::core::String)
                               forms)]
    ;; for each ref, check if it creates a violation
    (:wat::core::foldl
      (:wat::core::fn [viols <- (:wat::core::Vector :- [:wat::deporder::Violation])
                       kwd   <- :wat::core::String]
        -> (:wat::core::Vector :- [:wat::deporder::Violation])
        (:wat::core::let [sym-opt (:wat::core::HashMap/get sym-map kwd)]
          (:wat::core::match sym-opt 
            (:wat::core::None viols)
            ((:wat::core::Some sym-def)
             ;; defined in a different file?
             (:wat::core::if (:wat::core::= (:wat::deporder::SymDef/file sym-def) path)
               viols
               ;; not defmacro (order-free)?
               (:wat::core::if (:wat::core::= (:wat::deporder::SymDef/kind sym-def) "defmacro")
                 viols
                 ;; look up definer position
                 (:wat::core::let [def-path    (:wat::deporder::SymDef/file sym-def)
                                   def-pos-opt (:wat::core::HashMap/get pos-map def-path)]
                   (:wat::core::match def-pos-opt 
                     (:wat::core::None viols)
                     ((:wat::core::Some def-pos)
                      ;; violation: definer loads AFTER referencer
                      (:wat::core::if (:wat::core::i64::> def-pos ref-pos)
                        (:wat::core::concat viols
                          (:wat::core::Vector :wat::deporder::Violation
                            (:wat::deporder::Violation :referencer path :referencer-pos ref-pos :definer def-path :definer-pos def-pos :symbol kwd)))
                        viols))))))))))
      (:wat::core::Vector :wat::deporder::Violation)
      all-refs)))

;; ─── verify — the main pure function ─────────────────────────────────

(:wat::core::defn :wat::deporder::verify
  [files <- (:wat::core::Vector :- [:wat::source::File])]
  -> (:wat::core::Vector :- [:wat::deporder::Violation])
  (:wat::core::let [sym-map (:wat::deporder::build-symbol-map files)
                    pos-map (:wat::deporder::build-pos-map files)
                    n       (:wat::core::length files)]
    (:wat::core::foldl
      (:wat::core::fn [viols <- (:wat::core::Vector :- [:wat::deporder::Violation])
                       i     <- :wat::core::i64]
        -> (:wat::core::Vector :- [:wat::deporder::Violation])
        (:wat::core::let [file (:wat::core::Option/expect  
                                  (:wat::core::get files i) "verify: get file")]
          (:wat::core::concat viols
            (:wat::deporder::check-file-violations file i sym-map pos-map))))
      (:wat::core::Vector :wat::deporder::Violation)
      (:wat::core::range 0 n))))

;; ─── stdlib surface ───────────────────────────────────────────────────

;; stdlib-sources — wraps the Rust intrinsic's [path source] pairs into
;; SourceFile records. The intrinsic returns Vector<Vector<String>> where
;; each inner Vector is [path, source] in STDLIB_FILES order.
(:wat::core::defn :wat::deporder::stdlib-sources
  []
  -> (:wat::core::Vector :- [:wat::source::File])
  (:wat::core::let [pairs (:wat::stdlib::sources)
                    n     (:wat::core::length pairs)]
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::source::File])
                       i   <- :wat::core::i64]
        -> (:wat::core::Vector :- [:wat::source::File])
        (:wat::core::let [pair   (:wat::core::Option/expect  
                                    (:wat::core::get pairs i) "stdlib-sources: get pair")
                          path   (:wat::core::Option/expect  
                                    (:wat::core::get pair 0) "stdlib-sources: get path")
                          source (:wat::core::Option/expect  
                                    (:wat::core::get pair 1) "stdlib-sources: get source")]
          (:wat::core::concat acc
            (:wat::core::Vector :wat::source::File
              (:wat::source::File :path path :source source)))))
      (:wat::core::Vector :wat::source::File)
      (:wat::core::range 0 n))))

;; verify-stdlib — the two-line surface: wrap intrinsic then verify.
(:wat::core::defn :wat::deporder::verify-stdlib
  []
  -> (:wat::core::Vector :- [:wat::deporder::Violation])
  (:wat::deporder::verify (:wat::deporder::stdlib-sources)))
