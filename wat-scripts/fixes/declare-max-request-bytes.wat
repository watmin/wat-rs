;; wat-scripts/fixes/declare-max-request-bytes.wat — arc 278 #16 Stone 16.3 migration codemod.
;;
;; Self-hosted, comment-faithful fix-wat codemod — NO hand-editing of .wat files, use the tool.
;;
;; Ships ALONGSIDE the Strike-1 checker rule (src/types.rs `synthesize_surface_protocol`): for a
;; `:nature :wat::kernel::Peer'` surface, EVERY op in `:features` must now EXPLICITLY declare
;; `:max-request-bytes N` — omission is a located compile error (mirrors 16.1c's RequestTooLarge
;; lock). This codemod inserts the missing declaration, per op, right after the op's `-> :Response`
;; (before any options that may already follow it):
;;
;;   (op-name [self <- :S  req <- :S::OpRequest] -> :S::OpResponse)
;;     -> (op-name [self <- :S  req <- :S::OpRequest] -> :S::OpResponse :max-request-bytes 524288)
;;
;; WHICH ops are serviceable is DISCOVERED, not hardcoded: any `(:wat::core::defsurface :Name
;; :nature :wat::kernel::Peer' ... :features [...])` form, found by walking the WHOLE form tree
;; (not just top-level — this reaches a `defsurface` embedded inside a `defmacro`'s quasiquoted
;; template too, e.g. `wat/query.wat`'s `sift-rules-defsvc`: a backtick form reads as an ordinary
;; `(:wat::core::quasiquote ...)` List, so the generic walk descends into it exactly like any other
;; nested List — no special-casing needed). A surface whose `:nature` value keyword-name is anything
;; other than exactly `:wat::kernel::Peer'` (Struct/Record/HolonRecord surfaces, or a macro's
;; unresolved `~nature-expr`) is left byte-untouched: only serviceable ops carry a wire budget.
;;
;; VALUE per (surface, op) — a small hardcoded exception map, default 524288 (512 KiB, made
;; explicit) for everything else. Keyed on (surface-name, op-name) exactly, never op-name alone (a
;; `put` on a small surface must NOT get the bulk value):
;;   (:wat::telemetry::Journal, write-metrics) -> 10485760   (10 MiB bulk telemetry write)
;;   (:wat::telemetry::Journal, write-logs)    -> 10485760   (10 MiB bulk telemetry write)
;;   (:wat::query::Store,       put)           -> 10485760   (10 MiB bulk store write)
;;   (:probe::Big,              put)           -> 1048576    (1 MiB test-fixture bulk op)
;;   everything else                           -> 524288     (512 KiB default, made explicit)
;; A surface whose own name isn't resolvable as a literal keyword at this form (the macro-embedded
;; `~surface-kw` case) can never match an exception pair — it falls through to the 512 KiB default,
;; which is correct: none of the 4 exception ops are macro-generated.
;;
;; Comment/format faithful (span edits via fix-text-apply). Idempotent (re-run = 0 edits: an op
;; that already carries a `:max-request-bytes` key anywhere in its option tail is left alone).
;;
;; Usage (one EDN vector of paths on stdin) — see wat/fix.wat's STASH-DANCE header for why this
;; ships stashed against the Strike-1 checker rule:
;;   printf '["wat/telemetry.wat" "wat/query.wat" ...] \n' \
;;     | cargo wat ./wat-scripts/fixes/declare-max-request-bytes.wat

;; ── small helpers (mirrors wat-scripts/fixes/response-record-to-enum.wat) ──────────────────
(:wat::core::defn :user::strip-params [name <- :wat::core::String] -> :wat::core::String
  (:wat::core::first (:wat::core::string::split name "<")))

(:wat::core::defn :user::end-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-end-span n) lines))

;; real-end-off — like end-off, but sees THROUGH a reader-macro wrapper. `~x` parses to
;; `(:wat::core::unquote x)`, and the parser deliberately gives that synthesized List the
;; NARROW span of just the `~` character — "the inner form keeps its own [span]"
;; (crates/wat-reader/src/parser.rs `parse_reader_macro`). So `ast-end-span` on the outer
;; unquote List ends right after `~`, NOT after the wrapped symbol — using it directly would
;; insert our text mid-token (`~ :max-request-bytes 524288resp-kw`, verified via dry-run on
;; wat/query.wat's macro-embedded `-> ~resp-kw`). Recurse into the wrapped inner form for
;; `:wat::core::unquote`/`:wat::core::unquote-splicing` heads to find the TRUE end.
(:wat::core::defn :user::real-end-off
  [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "list")
    (:wat::core::let [ch (:wat::core::ast->children n)]
      (:wat::core::if (:wat::core::>= (:wat::core::length ch) 2)
        (:wat::core::let
          [h (:user::kw-name (:wat::core::Option/expect (:wat::core::get ch 0) "reo head"))]
          (:wat::core::if
            (:wat::core::if (:wat::core::= h ":wat::core::unquote") true
              (:wat::core::= h ":wat::core::unquote-splicing"))
            (:user::real-end-off (:wat::core::Option/expect (:wat::core::get ch 1) "reo inner") lines)
            (:user::end-off n lines)))
        (:user::end-off n lines)))
    (:user::end-off n lines)))

(:wat::core::defn :user::kw-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "keyword")
    (:wat::core::ast-name n) ""))

;; find-kw-value — first `val` in `ch` immediately following an element whose kw-name is `kwname`
;; (order-independent kwargs marker lookup, e.g. `:nature`/`:features` in a defsurface's arg list).
(:wat::core::defn :user::find-kw-value
  [ch <- (:wat::core::Vector :- [:wat::WatAST])  kwname <- :wat::core::String]
  -> (:wat::core::Option :- [:wat::WatAST])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Option :- [:wat::WatAST])  i <- :wat::core::i64]
      -> (:wat::core::Option :- [:wat::WatAST])
      (:wat::core::match acc 
        ((:wat::core::Some v) (:wat::core::Some v))
        (:wat::core::None
          (:wat::core::if
            (:wat::core::= (:user::kw-name (:wat::core::Option/expect (:wat::core::get ch i) "fkv cur")) kwname)
            (:wat::core::get ch (:wat::core::+ i 1))
            :wat::core::None))))
    :wat::core::None
    (:wat::core::range 0 (:wat::core::length ch))))

;; has-max-bytes? — true iff any element of `ch` at index >= 4 (past name/argvec/arrow/rettype) is
;; the `:max-request-bytes` keyword (idempotency gate — options are order-independent kwargs).
(:wat::core::defn :user::has-max-bytes? [ch <- (:wat::core::Vector :- [:wat::WatAST])] -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::bool  i <- :wat::core::i64] -> :wat::core::bool
      (:wat::core::if acc true
        (:wat::core::= (:user::kw-name (:wat::core::Option/expect (:wat::core::get ch i) "hmb")) ":max-request-bytes")))
    false
    (:wat::core::range 4 (:wat::core::length ch))))

;; budget-for — the (surface-name, op-name) exception map; default "524288" (512 KiB, explicit).
(:wat::core::defn :user::budget-for
  [surface-name <- :wat::core::String  op-name <- :wat::core::String] -> :wat::core::String
  (:wat::core::cond
    ((:wat::core::if (:wat::core::= surface-name ":wat::telemetry::Journal")
       (:wat::core::= op-name "write-metrics") false) "10485760")
    ((:wat::core::if (:wat::core::= surface-name ":wat::telemetry::Journal")
       (:wat::core::= op-name "write-logs") false) "10485760")
    ((:wat::core::if (:wat::core::= surface-name ":wat::query::Store")
       (:wat::core::= op-name "put") false) "10485760")
    ((:wat::core::if (:wat::core::= surface-name ":probe::Big")
       (:wat::core::= op-name "put") false) "1048576")
    (:else "524288")))

;; ── per-op edit: insert " :max-request-bytes <N>" right after the return-type node (child[3]) ──
(:wat::core::defn :user::op-edit
  [op <- :wat::WatAST  surface-name <- :wat::core::String  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind op) "list")
    (:wat::core::let [ch (:wat::core::ast->children op)]
      (:wat::core::if (:wat::core::< (:wat::core::length ch) 4)
        (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))
        (:wat::core::if (:user::has-max-bytes? ch)
          (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))
          (:wat::core::let
            [name-node (:wat::core::Option/expect (:wat::core::get ch 0) "op name")
             op-name   (:user::strip-params (:wat::core::ast-name name-node))
             ret-node  (:wat::core::Option/expect (:wat::core::get ch 3) "op ret")
             end       (:user::real-end-off ret-node lines)
             val       (:user::budget-for surface-name op-name)]
            (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])
              (:wat::core::Tuple end 0 (:wat::core::string::concat " :max-request-bytes " val)))))))
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))))

(:wat::core::defn :user::ops-edits
  [ops <- (:wat::core::Vector :- [:wat::WatAST])  surface-name <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])  op <- :wat::WatAST]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
      (:wat::core::concat acc (:user::op-edit op surface-name lines)))
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))
    ops))

;; ── per-defsurface: gate on `:nature :wat::kernel::Peer'`, then walk its `:features` vector ──
(:wat::core::defn :user::defsurface-edits
  [ch <- (:wat::core::Vector :- [:wat::WatAST])  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::let
    [nature-opt (:user::find-kw-value ch ":nature")
     is-peer
       (:wat::core::match nature-opt 
         (:wat::core::None false)
         ((:wat::core::Some nv) (:wat::core::= (:user::kw-name nv) ":wat::kernel::Peer'")))]
    (:wat::core::if (:wat::core::not is-peer)
      (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))
      (:wat::core::let
        [name-node (:wat::core::Option/expect (:wat::core::get ch 1) "ds name")
         surface-name
           (:wat::core::if (:wat::core::= (:wat::core::ast-kind name-node) "keyword")
             (:wat::core::ast-name name-node) "")
         features-opt (:user::find-kw-value ch ":features")]
        (:wat::core::match features-opt 
          (:wat::core::None (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])))
          ((:wat::core::Some fv)
            (:wat::core::if (:wat::core::= (:wat::core::ast-kind fv) "vector")
              (:user::ops-edits (:wat::core::ast->children fv) surface-name lines)
              (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])))))))))

;; ── generic tree walk — reaches EVERY defsurface, top-level or nested (macro-embedded) ────────
(:wat::core::defn :user::node-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch)
        (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))
        (:wat::core::let
          [hname (:user::kw-name (:wat::core::first ch))
           this
             (:wat::core::if (:wat::core::= hname ":wat::core::defsurface")
               (:user::defsurface-edits ch lines)
               (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])))]
          (:wat::core::concat this (:user::seq-edits ch lines)))))
    (:wat::core::if (:wat::fix::structural? node)
      (:user::seq-edits (:wat::core::ast->children node) lines)
      (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])))))

(:wat::core::defn :user::seq-edits
  [items <- (:wat::core::Vector :- [:wat::WatAST])  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])  it <- :wat::WatAST]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
      (:wat::core::concat acc (:user::node-edits it lines)))
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))
    items))

;; ── per-file migrate ─────────────────────────────────────────────────────────
(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [lines (:wat::core::string::split src "\n")
     forms (:wat::core::ast->children (:wat::core::match (:wat::core::read-string src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None))))
     eds   (:user::seq-edits forms lines)
     rev   (:wat::core::reverse (:wat::core::sort eds))]
    (:wat::fix::fix-text-apply src rev)))

;; ── driver ───────────────────────────────────────────────────────────────────
(:wat::core::defn :user::apply-each [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::core::string::concat "[max-request-bytes] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
