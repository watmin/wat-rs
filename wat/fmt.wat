;; wat/fmt.wat — layout engine: Break + a DUMB emitter. Rules assert Breaks; this file
;; holds no style opinion. Arc 277 STONE indent-is-structural.
;;
;; A node with a Break starts a new line. `:block` indents one level (2) from its
;; form's indent; `:align` sits one past the container's emitted opening delimiter.
;; A line comment PINS A NEWLINE after itself. Spans locate comments; they never
;; decide an indent.

;; kind is a String, not a keyword: rete RHS may insert a string literal but
;; refuses a keyword literal (`RhsUnresolvableOperand`). `"block"` | `"align"`.
(:wat::core::defrecord :wat::fmt::Break
  [id   <- :wat::core::i64
   kind <- :wat::core::String])

(:wat::core::defrecord :wat::fmt::Comment
  [text     <- :wat::core::String
   line     <- :wat::core::i64
   col      <- :wat::core::i64
   end-line <- :wat::core::i64
   end-col  <- :wat::core::i64])

;; A rule claims exactly the node it dispatched on and positions that node's
;; immediate children. The fallback (R11) fires where the parent is unclaimed.
;; The wall: applying a Break for X requires X's parent to be owned — Claim
;; (a specific rule) or Fallback (R11). R11 cannot assert Claim: that is
;; `not Claim -> Claim` and it races the per-child Breaks.
(:wat::core::defrecord :wat::fmt::Claim
  [form <- :wat::core::i64])

(:wat::core::defrecord :wat::fmt::Fallback
  [node <- :wat::core::i64])

;; Vertical separation, a different axis from Break.kind. A node may carry both.
(:wat::core::defrecord :wat::fmt::BlankBefore
  [id <- :wat::core::i64])

;; This form's broken children align their second token. The emitter
;; computes the pad from this pass's first tokens. No rule names a width.
(:wat::core::defrecord :wat::fmt::AlignPairs
  [form <- :wat::core::i64])

;; Pad child i so child i+1 lines up, i stepping by stride.
;; 2 = key/value (AlignPairs). 3 = name / binder / type.
(:wat::core::defrecord :wat::fmt::AlignStride
  [form   <- :wat::core::i64
   stride <- :wat::core::i64])

;; A list form's grouping identity: head name plus either the trailing
;; pair-run key sequence or positional arity ("#N").
(:wat::core::defrecord :wat::fmt::FormSig
  [form <- :wat::core::i64
   head <- :wat::core::String
   keys <- :wat::core::String])

;; A member of an adjacent same-sig group. group is the first member's id.
(:wat::core::defrecord :wat::fmt::TableRow
  [form  <- :wat::core::i64
   group <- :wat::core::i64])

;; One-line rendered width. From the walk, not rete. leaf = length(source);
;; interior = Σ children + n + 1.
(:wat::core::defrecord :wat::fmt::Width
  [id <- :wat::core::i64
   w  <- :wat::core::i64])

;; Pair collection whose every child is an atom. The emitter inlines it
;; when indent + Width fits the budget.
(:wat::core::defrecord :wat::fmt::AllAtoms
  [form <- :wat::core::i64])

;; After this node, write ` []`. A bare enum variant: both spellings are
;; legal and equivalent, so the insert is value-preserving. Idempotent —
;; a tag that already has a vector sibling does not get this fact.
(:wat::core::defrecord :wat::fmt::EmptyVecAfter
  [id <- :wat::core::i64])

(:wat::core::defrecord :wat::fmt::Acc
  [out      <- :wat::core::String
   next-id  <- :wat::core::i64
   comments <- (:wat::core::PersistentVector :- [:wat::fmt::Comment])
   col      <- :wat::core::i64])

(:wat::core::defrecord :wat::fmt::SigAcc
  [next-id <- :wat::core::i64
   sigs    <- (:wat::core::PersistentVector :- [:wat::fmt::FormSig])])

(:wat::core::defrecord :wat::fmt::WAcc
  [next-id <- :wat::core::i64
   last-w  <- :wat::core::i64
   facts   <- (:wat::core::PersistentVector :- [:wat::fmt::Width])])

(:wat::rete::defquery :wat::fmt::q-break
  :params []
  :when [(?b <- :wat::fmt::Break)])

(:wat::rete::defquery :wat::fmt::q-claim
  :params []
  :when [(?c <- :wat::fmt::Claim)])

(:wat::rete::defquery :wat::fmt::q-fallback
  :params []
  :when [(?f <- :wat::fmt::Fallback)])

(:wat::rete::defquery :wat::fmt::q-blank
  :params []
  :when [(?bl <- :wat::fmt::BlankBefore)])

(:wat::rete::defquery :wat::fmt::q-align
  :params []
  :when [(?ap <- :wat::fmt::AlignPairs)])

(:wat::rete::defquery :wat::fmt::q-stride
  :params []
  :when [(?st <- :wat::fmt::AlignStride)])

(:wat::rete::defquery :wat::fmt::q-table
  :params []
  :when [(?t <- :wat::fmt::TableRow)])

(:wat::rete::defquery :wat::fmt::q-atoms
  :params []
  :when [(?aa <- :wat::fmt::AllAtoms)])

(:wat::rete::defquery :wat::fmt::q-empty-vec
  :params []
  :when [(?ev <- :wat::fmt::EmptyVecAfter)])

(:wat::core::defn :wat::fmt::spaces [n <- :wat::core::i64] -> :wat::core::String
  (:wat::core::if (:wat::i64::<= n 0)
    ""
    (:wat::string::concat " " (:wat::fmt::spaces (:wat::i64::- n 1)))))

;; Drop trailing spaces/tabs on the current line. A newline after pad or
;; the inter-token space would otherwise leave R9 whitespace.
(:wat::core::defn :wat::fmt::rstrip-ws
  [s <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::if (:wat::string::empty? s)
    s
    (:wat::core::if (:wat::core::or (:wat::string::ends-with? s " ")
                                   (:wat::string::ends-with? s "\t"))
      (:wat::fmt::rstrip-ws (:wat::string::subs s 0 (:wat::i64::- (:wat::string::length s) 1)))
      s)))

(:wat::core::defn :wat::fmt::ensure-nl [s <- :wat::core::String] -> :wat::core::String
  (:wat::core::if (:wat::string::empty? s)
    s
    (:wat::core::if (:wat::string::ends-with? s "\n")
      s
      (:wat::string::concat (:wat::fmt::rstrip-ws s) "\n"))))

;; Column of the next write, given the previous column and a suffix just appended.
;; Derived from EMITTED text, never from a source span.
(:wat::core::defn :wat::fmt::col-after
  [col <- :wat::core::i64  s <- :wat::core::String]
  -> :wat::core::i64
  (:wat::core::if (:wat::string::empty? s)
    col
    (:wat::core::if (:wat::string::ends-with? s "\n")
      0
      (:wat::core::if (:wat::string::contains? s "\n")
        (:wat::core::let [lines (:wat::string::split s "\n")
                          n     (:wat::core::length lines)]
          (:wat::string::length (:wat::core::nth lines (:wat::i64::- n 1))))
        (:wat::i64::+ col (:wat::string::length s))))))

(:wat::core::defn :wat::fmt::write
  [acc <- :wat::fmt::Acc  s <- :wat::core::String]
  -> :wat::fmt::Acc
  (:wat::fmt::Acc
    :out      (:wat::string::concat (:wat::fmt::Acc/out acc) s)
    :next-id  (:wat::fmt::Acc/next-id acc)
    :comments (:wat::fmt::Acc/comments acc)
    :col      (:wat::fmt::col-after (:wat::fmt::Acc/col acc) s)))

(:wat::core::defn :wat::fmt::write-nl [acc <- :wat::fmt::Acc] -> :wat::fmt::Acc
  (:wat::core::let [s  (:wat::fmt::Acc/out acc)
                    s2 (:wat::fmt::ensure-nl s)]
    (:wat::core::if (:wat::core::= s s2)
      acc
      (:wat::fmt::Acc
        :out      s2
        :next-id  (:wat::fmt::Acc/next-id acc)
        :comments (:wat::fmt::Acc/comments acc)
        :col      0))))

(:wat::core::defn :wat::fmt::comment-before?
  [c <- :wat::fmt::Comment  line <- :wat::core::i64  col <- :wat::core::i64]
  -> :wat::core::bool
  (:wat::core::if (:wat::i64::< (:wat::fmt::Comment/line c) line)
    true
    (:wat::core::if (:wat::i64::= (:wat::fmt::Comment/line c) line)
      (:wat::i64::<= (:wat::fmt::Comment/col c) col)
      false)))

;; True when the current line is only the indent just written — writing a
;; newline now would leave a blank line of spaces.
(:wat::core::defn :wat::fmt::pending-indent?
  [acc <- :wat::fmt::Acc]
  -> :wat::core::bool
  (:wat::core::let [col (:wat::fmt::Acc/col acc)]
    (:wat::core::if (:wat::i64::<= col 0)
      false
      (:wat::core::let [out (:wat::fmt::Acc/out acc)
                        n   (:wat::string::length out)]
        (:wat::core::= (:wat::string::subs out (:wat::i64::- n col) n)
                       (:wat::fmt::spaces col))))))

(:wat::core::defn :wat::fmt::drop-pending-indent
  [acc <- :wat::fmt::Acc]
  -> :wat::fmt::Acc
  (:wat::core::let [col (:wat::fmt::Acc/col acc)
                    out (:wat::fmt::Acc/out acc)
                    n   (:wat::string::length out)]
    (:wat::fmt::Acc
      :out      (:wat::string::subs out 0 (:wat::i64::- n col))
      :next-id  (:wat::fmt::Acc/next-id acc)
      :comments (:wat::fmt::Acc/comments acc)
      :col      0)))

;; Start a comment on a fresh line without leaving spaces on the previous one.
(:wat::core::defn :wat::fmt::open-comment-line
  [acc <- :wat::fmt::Acc]
  -> :wat::fmt::Acc
  (:wat::core::if (:wat::i64::= (:wat::fmt::Acc/col acc) 0)
    acc
    (:wat::core::if (:wat::fmt::pending-indent? acc)
      (:wat::fmt::drop-pending-indent acc)
      (:wat::fmt::write-nl acc))))

(:wat::core::defn :wat::fmt::flush-comments
  [acc     <- :wat::fmt::Acc
   line    <- :wat::core::i64
   col     <- :wat::core::i64
   indent  <- :wat::core::i64
   restore <- :wat::core::bool]
  -> :wat::fmt::Acc
  (:wat::core::if (:wat::core::empty? (:wat::fmt::Acc/comments acc))
    acc
    (:wat::core::let [c (:wat::core::first (:wat::fmt::Acc/comments acc))]
      (:wat::core::if (:wat::fmt::comment-before? c line col)
        (:wat::fmt::flush-comments
          (:wat::core::let
            [opened (:wat::fmt::open-comment-line acc)
             written (:wat::fmt::write opened
                       (:wat::string::concat
                         (:wat::fmt::spaces indent)
                         (:wat::string::concat (:wat::fmt::Comment/text c) "\n")))
             restored (:wat::core::if restore
                        (:wat::fmt::write written (:wat::fmt::spaces indent))
                        written)]
            (:wat::fmt::Acc
              :out      (:wat::fmt::Acc/out restored)
              :next-id  (:wat::fmt::Acc/next-id restored)
              :comments (:wat::core::rest (:wat::fmt::Acc/comments acc))
              :col      (:wat::fmt::Acc/col restored)))
          line col indent restore)
        acc))))

;; Exactly one blank line at the end of `out` (two trailing newlines).
;; Missing → insert. Already present → leave. Extra → trim. Never spaces.
(:wat::core::defn :wat::fmt::trim-extra-nl
  [s <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::if (:wat::string::ends-with? s "\n\n\n")
    (:wat::fmt::trim-extra-nl
      (:wat::string::subs s 0 (:wat::i64::- (:wat::string::length s) 1)))
    s))

(:wat::core::defn :wat::fmt::ensure-blank
  [acc <- :wat::fmt::Acc]
  -> :wat::fmt::Acc
  (:wat::core::if (:wat::string::empty? (:wat::fmt::Acc/out acc))
    acc
    (:wat::core::let
      [s1 (:wat::fmt::ensure-nl (:wat::fmt::Acc/out acc))
       s2 (:wat::fmt::trim-extra-nl s1)
       s3 (:wat::core::if (:wat::string::ends-with? s2 "\n\n")
            s2
            (:wat::string::concat s2 "\n"))]
      (:wat::fmt::Acc
        :out      s3
        :next-id  (:wat::fmt::Acc/next-id acc)
        :comments (:wat::fmt::Acc/comments acc)
        :col      0))))

(:wat::core::defn :wat::fmt::open-of [kind <- :wat::core::String] -> :wat::core::String
  (:wat::core::if (:wat::core::= kind "list") "("
    (:wat::core::if (:wat::core::= kind "vector") "["
      (:wat::core::if (:wat::core::= kind "set") "#{"
        "{"))))

(:wat::core::defn :wat::fmt::close-of [kind <- :wat::core::String] -> :wat::core::String
  (:wat::core::if (:wat::core::= kind "list") ")"
    (:wat::core::if (:wat::core::= kind "vector") "]"
      "}")))

(:wat::core::defn :wat::fmt::pad-break
  [acc      <- :wat::fmt::Acc
   bk       <- :wat::core::String
   indent   <- :wat::core::i64
   open-col <- :wat::core::i64]
  -> :wat::fmt::Acc
  (:wat::core::if (:wat::core::or (:wat::core::= bk "block")
                                 (:wat::core::= bk "align"))
    (:wat::core::let [n (:wat::core::if (:wat::core::= bk "block")
                        (:wat::i64::+ indent 2)
                        (:wat::i64::+ open-col 1))]
      (:wat::fmt::write (:wat::fmt::write-nl acc) (:wat::fmt::spaces n)))
    (:wat::kernel::assertion-failed!
      "fmt: Break.kind must be block or align"
      :wat::core::None
      :wat::core::None)))

(:wat::core::defn :wat::fmt::claimed?
  [claims    <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
   parent-id <- :wat::core::i64]
  -> :wat::core::bool
  (:wat::core::if (:wat::i64::= parent-id 0)
    true
    (:wat::core::match (:wat::core::get claims parent-id)
      ((:wat::core::Some _) true)
      (:wat::core::None false))))

(:wat::core::defn :wat::fmt::apply-blank
  [acc    <- :wat::fmt::Acc
   id     <- :wat::core::i64
   blanks <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])]
  -> :wat::fmt::Acc
  (:wat::core::match (:wat::core::get blanks id)
    ((:wat::core::Some _)
      (:wat::fmt::write (:wat::fmt::write-nl acc) "\n"))
    (:wat::core::None acc)))

(:wat::core::defn :wat::fmt::apply-break
  [acc       <- :wat::fmt::Acc
   bk        <- :wat::core::String
   indent    <- :wat::core::i64
   open-col  <- :wat::core::i64
   id        <- :wat::core::i64
   parent-id <- :wat::core::i64
   claims    <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])]
  -> :wat::fmt::Acc
  (:wat::core::if (:wat::fmt::claimed? claims parent-id)
    (:wat::fmt::pad-break acc bk indent open-col)
    (:wat::kernel::assertion-failed!
      (:wat::string::interpolate
        "fmt: rule positioned a grandchild — node {n}'s parent is unclaimed"
        :n (:wat::i64::to-string id))
      :wat::core::None
      :wat::core::None)))

;; Child 1 is the symbol/keyword `:-` and child 2 is a vector. Shared by
;; type DECLARATIONS (arity 3, atomic) and CONSTRUCTORS (arity > 3, glue then explode).
(:wat::core::defn :wat::fmt::colon-args?
  [node <- :wat::WatAST]
  -> :wat::core::bool
  (:wat::core::if (:wat::core::not (:wat::core::= (:wat::core::ast-kind node) "list"))
    false
    (:wat::core::let [kids (:wat::core::ast->children node)]
      (:wat::core::if (:wat::i64::< (:wat::core::length kids) 3)
        false
        (:wat::core::let [c1 (:wat::core::nth kids 1)
                          c2 (:wat::core::nth kids 2)
                          k1 (:wat::core::ast-kind c1)]
          (:wat::core::if (:wat::core::not (:wat::core::or (:wat::core::= k1 "symbol")
                                                          (:wat::core::= k1 "keyword")))
            false
            (:wat::core::if (:wat::core::not (:wat::core::= (:wat::core::ast-name c1) ":-"))
              false
              (:wat::core::= (:wat::core::ast-kind c2) "vector"))))))))

(:wat::core::defn :wat::fmt::type-application?
  [node <- :wat::WatAST]
  -> :wat::core::bool
  (:wat::core::if (:wat::core::not (:wat::fmt::colon-args? node))
    false
    (:wat::i64::= (:wat::core::length (:wat::core::ast->children node)) 3)))

(:wat::core::defn :wat::fmt::type-constructor?
  [node <- :wat::WatAST]
  -> :wat::core::bool
  (:wat::core::if (:wat::core::not (:wat::fmt::colon-args? node))
    false
    (:wat::i64::> (:wat::core::length (:wat::core::ast->children node)) 3)))

(:wat::core::defn :wat::fmt::subtree-size
  [node <- :wat::WatAST]
  -> :wat::core::i64
  (:wat::core::if (:wat::core::not (:wat::grep::structural? node))
    1
    (:wat::core::foldl
      (:wat::core::fn [n <- :wat::core::i64  child <- :wat::WatAST] -> :wat::core::i64
        (:wat::i64::+ n (:wat::fmt::subtree-size child)))
      1
      (:wat::core::ast->children node))))

(:wat::core::defn :wat::fmt::count-colon-args
  [node <- :wat::WatAST]
  -> :wat::core::i64
  (:wat::core::let [here (:wat::core::if (:wat::fmt::colon-args? node) 1 0)]
    (:wat::core::if (:wat::core::not (:wat::grep::structural? node))
      here
      (:wat::core::foldl
        (:wat::core::fn [n <- :wat::core::i64  child <- :wat::WatAST] -> :wat::core::i64
          (:wat::i64::+ n (:wat::fmt::count-colon-args child)))
        here
        (:wat::core::ast->children node)))))

(:wat::core::defn :wat::fmt::count-type-apps
  [node <- :wat::WatAST]
  -> :wat::core::i64
  (:wat::core::let [here (:wat::core::if (:wat::fmt::type-application? node) 1 0)]
    (:wat::core::if (:wat::core::not (:wat::grep::structural? node))
      here
      (:wat::core::foldl
        (:wat::core::fn [n <- :wat::core::i64  child <- :wat::WatAST] -> :wat::core::i64
          (:wat::i64::+ n (:wat::fmt::count-type-apps child)))
        here
        (:wat::core::ast->children node)))))

(:wat::core::defn :wat::fmt::kw-key?
  [n <- :wat::WatAST]
  -> :wat::core::bool
  (:wat::core::if (:wat::core::not (:wat::core::= (:wat::core::ast-kind n) "keyword"))
    false
    (:wat::core::let [nm (:wat::core::ast-name n)]
      (:wat::core::if (:wat::core::= nm ":-")
        false
        (:wat::core::not (:wat::core::= nm "->"))))))

(:wat::core::defn :wat::fmt::even-keys?
  [kids <- (:wat::core::Vector :- [:wat::WatAST])
   i    <- :wat::core::i64
   lim  <- :wat::core::i64]
  -> :wat::core::bool
  (:wat::core::if (:wat::i64::>= i lim)
    true
    (:wat::core::if (:wat::fmt::kw-key? (:wat::core::nth kids i))
      (:wat::fmt::even-keys? kids (:wat::i64::+ i 2) lim)
      false)))

(:wat::core::defn :wat::fmt::run-from?
  [kids <- (:wat::core::Vector :- [:wat::WatAST])
   s    <- :wat::core::i64]
  -> :wat::core::bool
  (:wat::core::let [n   (:wat::core::length kids)
                    len (:wat::i64::- n s)]
    (:wat::core::if (:wat::i64::< len 2)
      false
      (:wat::core::if (:wat::core::not (:wat::i64::= (:wat::i64::rem len 2) 0))
        false
        (:wat::fmt::even-keys? kids s n)))))

(:wat::core::defn :wat::fmt::find-run
  [kids <- (:wat::core::Vector :- [:wat::WatAST])
   s    <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::if (:wat::i64::>= s (:wat::core::length kids))
    -1
    (:wat::core::if (:wat::fmt::run-from? kids s)
      s
      (:wat::fmt::find-run kids (:wat::i64::+ s 1)))))

(:wat::core::defn :wat::fmt::join-from
  [kids <- (:wat::core::Vector :- [:wat::WatAST])
   i    <- :wat::core::i64
   lim  <- :wat::core::i64
   acc  <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::if (:wat::i64::>= i lim)
    acc
    (:wat::core::let [nm   (:wat::core::ast-name (:wat::core::nth kids i))
                      acc2 (:wat::core::if (:wat::string::empty? acc)
                             nm
                             (:wat::string::concat acc (:wat::string::concat "|" nm)))]
      (:wat::fmt::join-from kids (:wat::i64::+ i 2) lim acc2))))

(:wat::core::defn :wat::fmt::form-keys
  [kids <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::core::String
  (:wat::core::let [s (:wat::fmt::find-run kids 1)]
    (:wat::core::if (:wat::i64::< s 0)
      (:wat::string::concat "#" (:wat::i64::to-string (:wat::i64::- (:wat::core::length kids) 1)))
      (:wat::fmt::join-from kids s (:wat::core::length kids) ""))))

(:wat::core::defn :wat::fmt::sigs-walk
  [acc  <- :wat::fmt::SigAcc
   node <- :wat::WatAST]
  -> :wat::fmt::SigAcc
  (:wat::core::let
    [id   (:wat::fmt::SigAcc/next-id acc)
     sigs (:wat::fmt::SigAcc/sigs acc)
     kind (:wat::core::ast-kind node)
     sigs2 (:wat::core::if (:wat::core::not (:wat::core::= kind "list"))
             sigs
             (:wat::core::let [kids (:wat::core::ast->children node)]
               (:wat::core::if (:wat::i64::<= (:wat::core::length kids) 0)
                 sigs
                 (:wat::core::if (:wat::core::not (:wat::grep::nameable? (:wat::core::nth kids 0)))
                   sigs
                   (:wat::vector::conj sigs
                     (:wat::fmt::FormSig
                       :form id
                       :head (:wat::core::ast-name (:wat::core::nth kids 0))
                       :keys (:wat::fmt::form-keys kids)))))))
     acc1 (:wat::fmt::SigAcc :next-id (:wat::i64::+ id 1) :sigs sigs2)]
    (:wat::core::if (:wat::grep::structural? node)
      (:wat::core::foldl
        (:wat::core::fn [a <- :wat::fmt::SigAcc  child <- :wat::WatAST] -> :wat::fmt::SigAcc
          (:wat::fmt::sigs-walk a child))
        acc1
        (:wat::core::ast->children node))
      acc1)))

(:wat::core::defn :wat::fmt::form-sigs-of
  [forms <- :wat::WatAST]
  -> (:wat::core::PersistentVector :- [:wat::fmt::FormSig])
  (:wat::fmt::SigAcc/sigs
    (:wat::core::foldl
      (:wat::core::fn [a <- :wat::fmt::SigAcc  form <- :wat::WatAST] -> :wat::fmt::SigAcc
        (:wat::fmt::sigs-walk a form))
      (:wat::fmt::SigAcc
        :next-id 1
        :sigs (:wat::core::PersistentVector :- [:wat::fmt::FormSig]))
      (:wat::core::ast->children forms))))

(:wat::core::defn :wat::fmt::width-walk
  [acc  <- :wat::fmt::WAcc
   node <- :wat::WatAST]
  -> :wat::fmt::WAcc
  (:wat::core::let
    [id   (:wat::fmt::WAcc/next-id acc)
     kids (:wat::core::ast->children node)]
    (:wat::core::if (:wat::core::empty? kids)
      (:wat::core::let [w (:wat::string::length (:wat::core::ast->source node))]
        (:wat::fmt::WAcc
          :next-id (:wat::i64::+ id 1)
          :last-w  w
          :facts   (:wat::vector::conj (:wat::fmt::WAcc/facts acc)
                     (:wat::fmt::Width :id id :w w))))
      (:wat::core::let
        [acc1 (:wat::fmt::WAcc
                :next-id (:wat::i64::+ id 1)
                :last-w  0
                :facts   (:wat::fmt::WAcc/facts acc))
         acc2 (:wat::core::foldl
                (:wat::core::fn [a <- :wat::fmt::WAcc  child <- :wat::WatAST] -> :wat::fmt::WAcc
                  (:wat::core::let [a2 (:wat::fmt::width-walk a child)]
                    (:wat::fmt::WAcc
                      :next-id (:wat::fmt::WAcc/next-id a2)
                      :last-w  (:wat::i64::+ (:wat::fmt::WAcc/last-w a) (:wat::fmt::WAcc/last-w a2))
                      :facts   (:wat::fmt::WAcc/facts a2))))
                acc1
                kids)
         n (:wat::core::length kids)
         w (:wat::i64::+ (:wat::fmt::WAcc/last-w acc2) (:wat::i64::+ n 1))]
        (:wat::fmt::WAcc
          :next-id (:wat::fmt::WAcc/next-id acc2)
          :last-w  w
          :facts   (:wat::vector::conj (:wat::fmt::WAcc/facts acc2)
                     (:wat::fmt::Width :id id :w w)))))))

(:wat::core::defn :wat::fmt::widths-of
  [forms <- :wat::WatAST]
  -> (:wat::core::PersistentVector :- [:wat::fmt::Width])
  (:wat::fmt::WAcc/facts
    (:wat::core::foldl
      (:wat::core::fn [a <- :wat::fmt::WAcc  form <- :wat::WatAST] -> :wat::fmt::WAcc
        (:wat::fmt::width-walk a form))
      (:wat::fmt::WAcc
        :next-id 1
        :last-w  0
        :facts   (:wat::core::PersistentVector :- [:wat::fmt::Width]))
      (:wat::core::ast->children forms))))

(:wat::core::defn :wat::fmt::widths-map
  [ws <- (:wat::core::PersistentVector :- [:wat::fmt::Width])]
  -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::foldl
    (:wat::core::fn [m <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
                     x <- :wat::fmt::Width]
      -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
      (:wat::hashmap::assoc m (:wat::fmt::Width/id x) (:wat::fmt::Width/w x)))
    (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
    ws))

(:wat::core::defn :wat::fmt::token-widths
  [node <- :wat::WatAST]
  -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::foldl
    (:wat::core::fn [v <- (:wat::core::PersistentVector :- [:wat::core::i64])
                     c <- :wat::WatAST]
      -> (:wat::core::PersistentVector :- [:wat::core::i64])
      (:wat::vector::conj v (:wat::string::length (:wat::core::ast->source c))))
    (:wat::core::PersistentVector :- [:wat::core::i64])
    (:wat::core::ast->children node)))

(:wat::core::defn :wat::fmt::max-vec
  [a   <- (:wat::core::PersistentVector :- [:wat::core::i64])
   b   <- (:wat::core::PersistentVector :- [:wat::core::i64])
   i   <- :wat::core::i64
   acc <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let [na (:wat::core::length a)
                    nb (:wat::core::length b)]
    (:wat::core::if (:wat::core::if (:wat::i64::>= i na) (:wat::i64::>= i nb) false)
      acc
      (:wat::core::let
        [va (:wat::core::if (:wat::i64::< i na) (:wat::core::nth a i) 0)
         vb (:wat::core::if (:wat::i64::< i nb) (:wat::core::nth b i) 0)
         v  (:wat::core::if (:wat::i64::> va vb) va vb)]
        (:wat::fmt::max-vec a b (:wat::i64::+ i 1) (:wat::vector::conj acc v))))))

(:wat::core::defn :wat::fmt::merge-widths
  [gw <- (:wat::core::HashMap :- [:wat::core::i64 (:wat::core::PersistentVector :- [:wat::core::i64])])
   g  <- :wat::core::i64
   tw <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> (:wat::core::HashMap :- [:wat::core::i64 (:wat::core::PersistentVector :- [:wat::core::i64])])
  (:wat::core::match (:wat::core::get gw g)
    (:wat::core::None (:wat::hashmap::assoc gw g tw))
    ((:wat::core::Some prev)
      (:wat::hashmap::assoc gw g
        (:wat::fmt::max-vec prev tw 0 (:wat::core::PersistentVector :- [:wat::core::i64]))))))

;; '(' + tokens + one space between + ')'. The emitter consults 120;
;; table.wat names no budget.
(:wat::core::defn :wat::fmt::sum-i64
  [v   <- (:wat::core::PersistentVector :- [:wat::core::i64])
   i   <- :wat::core::i64
   acc <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::if (:wat::i64::>= i (:wat::core::length v))
    acc
    (:wat::fmt::sum-i64 v (:wat::i64::+ i 1) (:wat::i64::+ acc (:wat::core::nth v i)))))

(:wat::core::defn :wat::fmt::table-line-width
  [cols <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [n (:wat::core::length cols)]
    (:wat::core::if (:wat::i64::= n 0)
      2
      (:wat::i64::+ (:wat::fmt::sum-i64 cols 0 0) (:wat::i64::+ n 1)))))

(:wat::core::defn :wat::fmt::table-row-fits?
  [tables <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
   gw     <- (:wat::core::HashMap :- [:wat::core::i64 (:wat::core::PersistentVector :- [:wat::core::i64])])
   id     <- :wat::core::i64
   indent <- :wat::core::i64]
  -> :wat::core::bool
  (:wat::core::match (:wat::core::get tables id)
    (:wat::core::None false)
    ((:wat::core::Some g)
      (:wat::core::match (:wat::core::get gw g)
        (:wat::core::None false)
        ((:wat::core::Some cols)
          (:wat::i64::<= (:wat::i64::+ indent (:wat::fmt::table-line-width cols)) 120))))))

(:wat::core::defn :wat::fmt::gw-walk
  [node   <- :wat::WatAST
   id     <- :wat::core::i64
   tables <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
   gw     <- (:wat::core::HashMap :- [:wat::core::i64 (:wat::core::PersistentVector :- [:wat::core::i64])])]
  -> (:wat::core::HashMap :- [:wat::core::i64 (:wat::core::PersistentVector :- [:wat::core::i64])])
  (:wat::core::let
    [gw1 (:wat::core::match (:wat::core::get tables id)
            ((:wat::core::Some g) (:wat::fmt::merge-widths gw g (:wat::fmt::token-widths node)))
            (:wat::core::None gw))]
    (:wat::core::if (:wat::fmt::type-application? node)
      gw1
      (:wat::core::if (:wat::grep::structural? node)
        (:wat::fmt::gw-kids (:wat::core::ast->children node) 0 (:wat::i64::+ id 1) tables gw1)
        gw1))))

(:wat::core::defn :wat::fmt::gw-kids
  [kids   <- (:wat::core::Vector :- [:wat::WatAST])
   i      <- :wat::core::i64
   id     <- :wat::core::i64
   tables <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
   gw     <- (:wat::core::HashMap :- [:wat::core::i64 (:wat::core::PersistentVector :- [:wat::core::i64])])]
  -> (:wat::core::HashMap :- [:wat::core::i64 (:wat::core::PersistentVector :- [:wat::core::i64])])
  (:wat::core::if (:wat::i64::>= i (:wat::core::length kids))
    gw
    (:wat::core::let [child (:wat::core::nth kids i)
                      gw2   (:wat::fmt::gw-walk child id tables gw)]
      (:wat::fmt::gw-kids kids (:wat::i64::+ i 1)
        (:wat::i64::+ id (:wat::fmt::subtree-size child)) tables gw2))))

;; Widest first-token among broken children whose NEXT sibling has no
;; Break (a key with a riding value). Prefix compounds do not contribute:
;; their next sibling is also broken. From THIS pass's source spelling.
(:wat::core::defn :wat::fmt::broken-key-width
  [kids    <- (:wat::core::Vector :- [:wat::WatAST])
   i       <- :wat::core::i64
   id      <- :wat::core::i64
   breaks  <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::String])
   empties <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
   acc     <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::if (:wat::i64::>= i (:wat::core::length kids))
    acc
    (:wat::core::let
      [child (:wat::core::nth kids i)
       size  (:wat::fmt::subtree-size child)
       nid   (:wat::i64::+ id size)
       ride? (:wat::core::match (:wat::core::get empties id)
               ((:wat::core::Some _) true)
               (:wat::core::None
                 (:wat::core::if (:wat::i64::< (:wat::i64::+ i 1) (:wat::core::length kids))
                   (:wat::core::match (:wat::core::get breaks nid)
                     ((:wat::core::Some _) false)
                     (:wat::core::None true))
                   false)))
       w     (:wat::core::match (:wat::core::get breaks id)
               ((:wat::core::Some _)
                 (:wat::core::if ride?
                   (:wat::core::let [n (:wat::string::length (:wat::core::ast->source child))]
                     (:wat::core::if (:wat::i64::> n acc) n acc))
                   acc))
               (:wat::core::None acc))]
      (:wat::fmt::broken-key-width kids (:wat::i64::+ i 1) nid breaks empties w))))

;; Max first-token among children at indices 0, stride, 2*stride, …
;; so the next token (`<-`) lines up. Includes the unbroken first name.
(:wat::core::defn :wat::fmt::stride-name-width
  [kids   <- (:wat::core::Vector :- [:wat::WatAST])
   i      <- :wat::core::i64
   stride <- :wat::core::i64
   acc    <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::if (:wat::i64::>= i (:wat::core::length kids))
    acc
    (:wat::core::let
      [n (:wat::string::length (:wat::core::ast->source (:wat::core::nth kids i)))
       w (:wat::core::if (:wat::core::if (:wat::i64::> stride 0)
                           (:wat::i64::= (:wat::i64::rem i stride) 0)
                           false)
           (:wat::core::if (:wat::i64::> n acc) n acc)
           acc)]
      (:wat::fmt::stride-name-width kids (:wat::i64::+ i 1) stride w))))

(:wat::core::defn :wat::fmt::emit-kids
  [acc        <- :wat::fmt::Acc
   kids       <- (:wat::core::Vector :- [:wat::WatAST])
   i          <- :wat::core::i64
   ctor?      <- :wat::core::bool
   breaks     <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::String])
   claims     <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
   blanks     <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
   aligns     <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
   tables     <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
   gw         <- (:wat::core::HashMap :- [:wat::core::i64 (:wat::core::PersistentVector :- [:wat::core::i64])])
   atoms      <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
   widths     <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
   strides    <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
   empties    <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
   indent     <- :wat::core::i64
   open-col   <- :wat::core::i64
   parent-id  <- :wat::core::i64
   pair-width <- :wat::core::i64
   stride     <- :wat::core::i64
   tblw       <- (:wat::core::PersistentVector :- [:wat::core::i64])
   skip-br?   <- :wat::core::bool]
  -> :wat::fmt::Acc
  (:wat::core::if (:wat::i64::>= i (:wat::core::length kids))
    acc
    (:wat::core::let
      [child (:wat::core::nth kids i)
       cid   (:wat::fmt::Acc/next-id acc)
       o     (:wat::fmt::Acc/out acc)
       first-kid? (:wat::core::or
                    (:wat::string::ends-with? o "(")
                    (:wat::core::or
                      (:wat::string::ends-with? o "[")
                      (:wat::core::or
                        (:wat::string::ends-with? o "{")
                        (:wat::string::ends-with? o "\n"))))
       acc2 (:wat::fmt::emit-node acc child breaks claims blanks aligns tables gw atoms widths strides empties indent open-col first-kid? parent-id
               (:wat::core::if ctor? (:wat::i64::= i 2) false) skip-br?)
       nid  (:wat::i64::+ cid (:wat::fmt::subtree-size child))
       ride? (:wat::core::match (:wat::core::get empties cid)
               ((:wat::core::Some _) true)
               (:wat::core::None
                 (:wat::core::if (:wat::i64::< (:wat::i64::+ i 1) (:wat::core::length kids))
                   (:wat::core::match (:wat::core::get breaks nid)
                     ((:wat::core::Some _) false)
                     (:wat::core::None true))
                   false)))
       acc3 (:wat::core::if
              (:wat::core::if (:wat::i64::> pair-width 0)
                (:wat::core::if (:wat::i64::> stride 0)
                  (:wat::core::if (:wat::i64::< (:wat::i64::+ i 1) (:wat::core::length kids))
                    (:wat::i64::= (:wat::i64::rem i stride) 0)
                    false)
                  (:wat::core::if
                    (:wat::core::match (:wat::core::get breaks cid)
                      ((:wat::core::Some _) true)
                      (:wat::core::None false))
                    ride?
                    false))
                false)
              (:wat::fmt::write acc2
                (:wat::fmt::spaces
                  (:wat::i64::- pair-width
                    (:wat::string::length (:wat::core::ast->source child)))))
              acc2)
       acc4 (:wat::core::if
              (:wat::core::if (:wat::i64::< i (:wat::core::length tblw))
                (:wat::i64::< (:wat::i64::+ i 1) (:wat::core::length kids))
                false)
              (:wat::fmt::write acc3
                (:wat::fmt::spaces
                  (:wat::i64::- (:wat::core::nth tblw i)
                    (:wat::string::length (:wat::core::ast->source child)))))
              acc3)
       acc5 (:wat::core::match (:wat::core::get empties cid)
              ((:wat::core::Some _) (:wat::fmt::write acc4 " []"))
              (:wat::core::None acc4))]
      (:wat::fmt::emit-kids acc5 kids (:wat::i64::+ i 1) ctor? breaks claims blanks aligns tables gw atoms widths strides empties indent open-col parent-id pair-width stride tblw skip-br?))))

(:wat::core::defn :wat::fmt::emit-node
  [acc        <- :wat::fmt::Acc
   node       <- :wat::WatAST
   breaks     <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::String])
   claims     <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
   blanks     <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
   aligns     <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
   tables     <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
   gw         <- (:wat::core::HashMap :- [:wat::core::i64 (:wat::core::PersistentVector :- [:wat::core::i64])])
   atoms      <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
   widths     <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
   strides    <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
   empties    <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
   indent     <- :wat::core::i64
   open-col   <- :wat::core::i64
   first?     <- :wat::core::bool
   parent-id  <- :wat::core::i64
   force-leaf <- :wat::core::bool
   skip-br?   <- :wat::core::bool]
  -> :wat::fmt::Acc
  (:wat::core::let
    [id        (:wat::fmt::Acc/next-id acc)
     node-kind (:wat::core::ast-kind node)
     x         (:wat::grep::extent-of node)
     src-line  (:wat::grep::Extent/line x)
     src-col   (:wat::grep::Extent/col x)
     br        (:wat::core::if skip-br?
                 :wat::core::None
                 (:wat::core::if (:wat::fmt::table-row-fits? tables gw parent-id indent)
                   :wat::core::None
                   (:wat::core::get breaks id)))
     acc-b     (:wat::fmt::Acc
                 :out      (:wat::fmt::Acc/out acc)
                 :next-id  (:wat::i64::+ id 1)
                 :comments (:wat::fmt::Acc/comments acc)
                 :col      (:wat::fmt::Acc/col acc))
     acc-bl    (:wat::fmt::apply-blank acc-b id blanks)
     acc-pad   (:wat::core::match br
                 ((:wat::core::Some bk)
                   (:wat::fmt::apply-break acc-bl bk indent open-col id parent-id claims))
                 (:wat::core::None
                   (:wat::core::if first?
                     acc-bl
                     (:wat::core::if (:wat::string::empty? (:wat::fmt::Acc/out acc-bl))
                       acc-bl
                       (:wat::fmt::write acc-bl " ")))))
     this-indent (:wat::core::if (:wat::i64::= parent-id 0)
                    0
                    (:wat::fmt::Acc/col acc-pad))
     acc-com     (:wat::fmt::flush-comments acc-pad src-line src-col this-indent true)
     acc1        (:wat::core::if (:wat::i64::= parent-id 0)
                    (:wat::core::if (:wat::fmt::pending-indent? acc-com)
                      (:wat::fmt::drop-pending-indent acc-com)
                      acc-com)
                    acc-com)]
    (:wat::core::if (:wat::core::or (:wat::fmt::type-application? node) force-leaf)
      (:wat::core::let
        [acc2 (:wat::fmt::write acc1 (:wat::core::ast->source node))
         acc3 (:wat::fmt::Acc
                :out      (:wat::fmt::Acc/out acc2)
                :next-id  (:wat::i64::+ id (:wat::fmt::subtree-size node))
                :comments (:wat::fmt::Acc/comments acc2)
                :col      (:wat::fmt::Acc/col acc2))]
        (:wat::fmt::flush-comments acc3
          (:wat::grep::Extent/end-line x)
          (:wat::grep::Extent/end-col x)
          this-indent
          false))
    (:wat::core::if (:wat::grep::structural? node)
      (:wat::core::let
        [this-open (:wat::fmt::Acc/col acc1)
         acc2      (:wat::fmt::write acc1 (:wat::fmt::open-of node-kind))
         kids      (:wat::core::ast->children node)
         skip-kids (:wat::core::if (:wat::fmt::table-row-fits? tables gw id this-indent)
                     true
                     (:wat::core::match (:wat::core::get atoms id)
                       (:wat::core::None false)
                       ((:wat::core::Some _)
                         (:wat::core::match (:wat::core::get widths id)
                           (:wat::core::None false)
                           ((:wat::core::Some w)
                             (:wat::i64::<= (:wat::i64::+ this-indent w) 120))))))
         st        (:wat::core::match (:wat::core::get strides id)
                     ((:wat::core::Some s) s)
                     (:wat::core::None 0))
         pw        (:wat::core::if skip-kids
                     0
                     (:wat::core::if (:wat::i64::> st 0)
                       (:wat::fmt::stride-name-width kids 0 st 0)
                       (:wat::core::match (:wat::core::get aligns id)
                         ((:wat::core::Some _)
                           (:wat::fmt::broken-key-width kids 0 (:wat::i64::+ id 1) breaks empties 0))
                         (:wat::core::None 0))))
         tblw      (:wat::core::if (:wat::fmt::table-row-fits? tables gw id this-indent)
                     (:wat::core::match (:wat::core::get tables id)
                       ((:wat::core::Some g)
                         (:wat::core::match (:wat::core::get gw g)
                           ((:wat::core::Some v) v)
                           (:wat::core::None (:wat::core::PersistentVector :- [:wat::core::i64]))))
                       (:wat::core::None (:wat::core::PersistentVector :- [:wat::core::i64])))
                     (:wat::core::PersistentVector :- [:wat::core::i64]))
         acc3      (:wat::fmt::emit-kids acc2 kids 0
                     (:wat::fmt::type-constructor? node)
                     breaks claims blanks aligns tables gw atoms widths strides empties this-indent this-open id pw st tblw skip-kids)
         acc4 (:wat::fmt::write acc3 (:wat::fmt::close-of node-kind))]
        (:wat::fmt::flush-comments acc4
          (:wat::grep::Extent/end-line x)
          (:wat::grep::Extent/end-col x)
          this-indent
          false))
      (:wat::core::let
        [acc2 (:wat::fmt::write acc1 (:wat::core::ast->source node))]
        (:wat::fmt::flush-comments acc2
          (:wat::grep::Extent/end-line x)
          (:wat::grep::Extent/end-col x)
          this-indent
          false))))))

(:wat::core::defn :wat::fmt::emit
  [forms    <- :wat::WatAST
   comments <- (:wat::core::PersistentVector :- [:wat::fmt::Comment])
   breaks   <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::String])
   claims   <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
   blanks   <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
   aligns   <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
   tables   <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
   atoms    <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
   widths   <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
   strides  <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
   empties  <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])]
  -> :wat::core::String
  (:wat::core::let
    [top  (:wat::core::ast->children forms)
     gw   (:wat::fmt::gw-kids top 0 1 tables
            (:wat::core::HashMap :- [:wat::core::i64 (:wat::core::PersistentVector :- [:wat::core::i64])]))
     acc0 (:wat::fmt::Acc :out "" :next-id 1 :comments comments :col 0)
     acc1 (:wat::core::foldl
            (:wat::core::fn [acc <- :wat::fmt::Acc  form <- :wat::WatAST] -> :wat::fmt::Acc
              (:wat::core::let [acc-b (:wat::core::if (:wat::string::empty? (:wat::fmt::Acc/out acc))
                                     acc
                                     (:wat::fmt::ensure-blank acc))]
                (:wat::fmt::emit-node acc-b form breaks claims blanks aligns tables gw atoms widths strides empties 0 0 true 0 false false)))
            acc0
            top)
     acc2 (:wat::fmt::write-nl acc1)
     acc3 (:wat::core::foldl
            (:wat::core::fn [acc <- :wat::fmt::Acc  c <- :wat::fmt::Comment] -> :wat::fmt::Acc
              (:wat::fmt::write
                (:wat::fmt::write-nl acc)
                (:wat::string::concat (:wat::fmt::Comment/text c) "\n")))
            acc2
            (:wat::fmt::Acc/comments acc2))]
    (:wat::fmt::Acc/out acc3)))

(:wat::core::defn :wat::fmt::breaks-map
  [session <- :wat::rete::Session]
  -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [m <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::String])
                     binding <- :wat::core::PersistentMap]
      -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::String])
      (:wat::core::let [b (:wat::core::Option/expect
                            (:wat::map::get binding "?b")
                            "fmt::breaks-map: no ?b")
                        id (:wat::fmt::Break/id b)
                        k  (:wat::fmt::Break/kind b)]
        (:wat::core::match (:wat::core::get m id)
          (:wat::core::None
            (:wat::hashmap::assoc m id k))
          ((:wat::core::Some prev)
            (:wat::core::if (:wat::core::= prev k)
              (:wat::hashmap::assoc m id k)
              (:wat::kernel::assertion-failed!
                (:wat::string::interpolate
                  "fmt: conflicting Breaks for node {n} — {a} vs {b}"
                  :n (:wat::i64::to-string id)
                  :a prev
                  :b k)
                :wat::core::None
                :wat::core::None))))))
    (:wat::core::HashMap :- [:wat::core::i64 :wat::core::String])
    (:wat::rete::query session (:wat::fmt::q-break))))

(:wat::core::defn :wat::fmt::claims-set
  [session <- :wat::rete::Session]
  -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
  (:wat::core::foldl
    (:wat::core::fn [m <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
                     binding <- :wat::core::PersistentMap]
      -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
      (:wat::core::let [c (:wat::core::Option/expect
                            (:wat::map::get binding "?c")
                            "fmt::claims-set: no ?c")]
        (:wat::hashmap::assoc m (:wat::fmt::Claim/form c) true)))
    (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
    (:wat::rete::query session (:wat::fmt::q-claim))))

(:wat::core::defn :wat::fmt::owned-set
  [session <- :wat::rete::Session]
  -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
  (:wat::core::foldl
    (:wat::core::fn [m <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
                     binding <- :wat::core::PersistentMap]
      -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
      (:wat::core::let [f (:wat::core::Option/expect
                            (:wat::map::get binding "?f")
                            "fmt::owned-set: no ?f")]
        (:wat::hashmap::assoc m (:wat::fmt::Fallback/node f) true)))
    (:wat::fmt::claims-set session)
    (:wat::rete::query session (:wat::fmt::q-fallback))))

(:wat::core::defn :wat::fmt::blanks-set
  [session <- :wat::rete::Session]
  -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
  (:wat::core::foldl
    (:wat::core::fn [m <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
                     binding <- :wat::core::PersistentMap]
      -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
      (:wat::core::let [bl (:wat::core::Option/expect
                             (:wat::map::get binding "?bl")
                             "fmt::blanks-set: no ?bl")]
        (:wat::hashmap::assoc m (:wat::fmt::BlankBefore/id bl) true)))
    (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
    (:wat::rete::query session (:wat::fmt::q-blank))))

(:wat::core::defn :wat::fmt::aligns-set
  [session <- :wat::rete::Session]
  -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
  (:wat::core::foldl
    (:wat::core::fn [m <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
                     binding <- :wat::core::PersistentMap]
      -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
      (:wat::core::let [ap (:wat::core::Option/expect
                             (:wat::map::get binding "?ap")
                             "fmt::aligns-set: no ?ap")]
        (:wat::hashmap::assoc m (:wat::fmt::AlignPairs/form ap) true)))
    (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
    (:wat::rete::query session (:wat::fmt::q-align))))

(:wat::core::defn :wat::fmt::tables-map
  [session <- :wat::rete::Session]
  -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::foldl
    (:wat::core::fn [m <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
                     binding <- :wat::core::PersistentMap]
      -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
      (:wat::core::let [tr (:wat::core::Option/expect
                             (:wat::map::get binding "?t")
                             "fmt::tables-map: no ?t")]
        (:wat::hashmap::assoc m (:wat::fmt::TableRow/form tr) (:wat::fmt::TableRow/group tr))))
    (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
    (:wat::rete::query session (:wat::fmt::q-table))))

(:wat::core::defn :wat::fmt::strides-map
  [session <- :wat::rete::Session]
  -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::foldl
    (:wat::core::fn [m <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
                     binding <- :wat::core::PersistentMap]
      -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
      (:wat::core::let [st (:wat::core::Option/expect
                             (:wat::map::get binding "?st")
                             "fmt::strides-map: no ?st")]
        (:wat::hashmap::assoc m (:wat::fmt::AlignStride/form st) (:wat::fmt::AlignStride/stride st))))
    (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
    (:wat::rete::query session (:wat::fmt::q-stride))))

(:wat::core::defn :wat::fmt::empties-set
  [session <- :wat::rete::Session]
  -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
  (:wat::core::foldl
    (:wat::core::fn [m <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
                     binding <- :wat::core::PersistentMap]
      -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
      (:wat::core::let [ev (:wat::core::Option/expect
                             (:wat::map::get binding "?ev")
                             "fmt::empties-set: no ?ev")]
        (:wat::hashmap::assoc m (:wat::fmt::EmptyVecAfter/id ev) true)))
    (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
    (:wat::rete::query session (:wat::fmt::q-empty-vec))))

(:wat::core::defn :wat::fmt::atoms-set
  [session <- :wat::rete::Session]
  -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
  (:wat::core::foldl
    (:wat::core::fn [m <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
                     binding <- :wat::core::PersistentMap]
      -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
      (:wat::core::let [aa (:wat::core::Option/expect
                             (:wat::map::get binding "?aa")
                             "fmt::atoms-set: no ?aa")]
        (:wat::hashmap::assoc m (:wat::fmt::AllAtoms/form aa) true)))
    (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
    (:wat::rete::query session (:wat::fmt::q-atoms))))

(:wat::core::defn :wat::fmt::format-source
  [path  <- :wat::core::String
   src   <- :wat::core::String
   rules <- (:wat::core::PersistentVector :- [:wat::rete::Rule])]
  -> :wat::core::String
  (:wat::core::match (:wat::core::read-string-with-comments src)
    ((:wat::core::ReadWithCommentsOutcome::Forms forms comments)
      (:wat::core::let
        [facts   (:wat::grep::facts-of path src)
         rec0    (:wat::grep::facts-as-records facts)
         records (:wat::core::foldl
                   (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])
                                    s   <- :wat::fmt::FormSig]
                     -> (:wat::core::PersistentVector :- [:wat::core::Record])
                     (:wat::vector::conj acc s))
                   rec0
                   (:wat::fmt::form-sigs-of forms))
         queries (:wat::core::PersistentVector :- [:wat::rete::Query]
                   (:wat::fmt::q-break)
                   (:wat::fmt::q-claim)
                   (:wat::fmt::q-fallback)
                   (:wat::fmt::q-blank)
                   (:wat::fmt::q-align)
                   (:wat::fmt::q-table)
                   (:wat::fmt::q-atoms)
                   (:wat::fmt::q-stride)
                   (:wat::fmt::q-empty-vec))]
        (:wat::rete::with-overlay rules queries
          (:wat::core::fn [overlay <- :wat::rete::Overlay]
            -> :wat::core::String
            (:wat::core::let [fired (overlay records)]
              (:wat::fmt::emit forms comments
                (:wat::fmt::breaks-map fired)
                (:wat::fmt::owned-set fired)
                (:wat::fmt::blanks-set fired)
                (:wat::fmt::aligns-set fired)
                (:wat::fmt::tables-map fired)
                (:wat::fmt::atoms-set fired)
                (:wat::fmt::widths-map (:wat::fmt::widths-of forms))
                (:wat::fmt::strides-map fired)
                (:wat::fmt::empties-set fired)))))))
    ((:wat::core::ReadWithCommentsOutcome::Malformed cause)
      (:wat::kernel::assertion-failed! (:wat::core::Error/message cause) :wat::core::None :wat::core::None))))
