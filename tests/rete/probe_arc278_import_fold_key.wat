;; DISCONFIRMING PROBE FIXTURE — vigilia Class A2.
;;
;; A KEYED accumulate fold (`acc::sum ?v`) inside a rule, with export and import as SEPARATE
;; entry points so a caller can inspect or tamper with the Export between them. The corpus had
;; no such fixture: `probe_arc278_derived_exists_acc.wat` pairs an accumulate with an export but
;; its fold is `acc::count`, which carries NO key and so never reaches `acc_var_i64`; and every
;; keyed-fold fixture (`probe_arc278_8i_accumulator_folds.wat`, the perf grid) calls the fold
;; library directly rather than through a rule's `:from`, so it builds no Accumulate NODE.
;;
;; The key is what `unpack_fold` reads straight off the wire (`export.rs`, the `:sum` arm) and
;; hands to `acc_var_i64`, whose doc calls an unbound var "a compile-time-impossible shape".
;; That proof is `build_rete_arm`'s; `import_export` does not run it.
;;
;; Want, untampered: exactly ONE SumF, on both the native and the imported path — and the
;; rule's `where` fence pins the fold's VALUE at 30 (10 + 20), so that count cannot pass on a
;; wrong sum. A live equality gate, not only a tamper target.

(:wat::core::defrecord :ifk::Group   [g <- :wat::core::i64])
(:wat::core::defrecord :ifk::Reading [g <- :wat::core::i64  v <- :wat::core::i64])
(:wat::core::defrecord :ifk::SumF    [g <- :wat::core::i64  n <- :wat::core::i64])

(:wat::rete::defrule :ifk::sum-rule
  :when [(:ifk::Group (?g <- :g))
         (?n <- (:wat::rete::acc::sum ?v) :from (:ifk::Reading (?g <- :g) (?v <- :v)))
         ;; The fence makes the COUNT see the VALUE: SumF derives only if the fold really
         ;; summed to 30, so a silently wrong sum changes the count the probe asserts.
         (:wat::rete::where (:wat::rete::core::i64::= ?n 30))]
  :then [(:ifk::SumF :g ?g :n ?n)])

(:wat::rete::defquery :ifk::q-Sum :params [] :when [(?f <- :ifk::SumF)])

(:wat::core::defn :ifk::rules [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector (:ifk::sum-rule)))

(:wat::core::defn :ifk::queries [] -> (:wat::core::PersistentVector :- [:wat::rete::Query])
  (:wat::core::PersistentVector (:ifk::q-Sum)))

(:wat::core::defn :ifk::compile [] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::compile-all (:ifk::rules) (:ifk::queries))
    ((:wat::rete::CompileOutcome::Compiled __session) __session)
    ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type)
      (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None))))

(:wat::core::defn :ifk::seed [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert s
    (:ifk::Group :g 1)
    (:ifk::Reading :g 1 :v 10)
    (:ifk::Reading :g 1 :v 20))
    ((:wat::rete::InsertOutcome::Inserted __staged) __staged)
    ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count)
      (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))

(:wat::core::defn :ifk::fired-sum [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length
    (:wat::rete::query
      (:wat::core::match (:wat::rete::fire-rules (:ifk::seed s))
        ((:wat::rete::FireOutcome::Fired __fired) __fired)
        ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds)
          (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None))
        ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still)
          (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
      (:ifk::q-Sum))))

;; THE THREE ENTRY POINTS THE PROBE NEEDS — export and import held apart, so the Export is a
;; value a caller can hold, read and tamper with between them.

(:wat::core::defn :user::fold-export [] -> :wat::rete::Export
  (:wat::rete::export (:ifk::compile)))

(:wat::core::defn :user::fold-import-and-fire [e <- :wat::rete::Export] -> :wat::core::i64
  (:ifk::fired-sum (:wat::rete::import e)))

(:wat::core::defn :user::fold-native-fire [] -> :wat::core::i64
  (:ifk::fired-sum (:ifk::compile)))

;; ── THE UNPACKED HALF — the SAME gap, reached through `Bindings::get` ─────────────────────────
;;
;; The rule above lands in `acc_var_i64`'s PACKED branch (`el.binds.len == 0`), so it can only
;; ever prove that one arm. Two more arms live behind `el.binds.len > 0`, and reaching them takes
;; two independent things, both built in below:
;;
;;   1. `el.binds.len > 0` — `pack_i64_row` refuses a record with ANY non-i64 field, so the
;;      String `:tag` denies `Tagged` a packed row and `seed`'s `skip_span` is false: elements
;;      carry a real binding span.
;;   2. `accumulate_value` instead of `fold_bucket` — the `:from` binds `?tag`, which neither the
;;      token binds nor the fold names, so `group_keys` is non-empty and the pass takes the
;;      grouped gather.
;;
;; Untampered want: TWO tag groups (a=10, b=20), the `where` fence admits only the one summing
;; to 10, so exactly ONE TagSum — a value gate again, not a liveness count.

(:wat::core::defrecord :ifk::Tagged [g <- :wat::core::i64  v <- :wat::core::i64  tag <- :wat::core::String])
(:wat::core::defrecord :ifk::TagSum [tag <- :wat::core::String  n <- :wat::core::i64])

(:wat::rete::defrule :ifk::tag-sum-rule
  :when [(:ifk::Group (?g <- :g))
         (?n <- (:wat::rete::acc::sum ?v) :from (:ifk::Tagged (?g <- :g) (?v <- :v) (?tag <- :tag)))
         (:wat::rete::where (:wat::rete::core::i64::= ?n 10))]
  :then [(:ifk::TagSum :tag ?tag :n ?n)])

(:wat::rete::defquery :ifk::q-TagSum :params [] :when [(?f <- :ifk::TagSum)])

(:wat::core::defn :ifk::tag-rules [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector (:ifk::tag-sum-rule)))

(:wat::core::defn :ifk::tag-queries [] -> (:wat::core::PersistentVector :- [:wat::rete::Query])
  (:wat::core::PersistentVector (:ifk::q-TagSum)))

(:wat::core::defn :ifk::tag-compile [] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::compile-all (:ifk::tag-rules) (:ifk::tag-queries))
    ((:wat::rete::CompileOutcome::Compiled __session) __session)
    ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type)
      (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None))))

(:wat::core::defn :ifk::tag-seed [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert s
    (:ifk::Group :g 1)
    (:ifk::Tagged :g 1 :v 10 :tag "a")
    (:ifk::Tagged :g 1 :v 20 :tag "b"))
    ((:wat::rete::InsertOutcome::Inserted __staged) __staged)
    ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count)
      (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))

(:wat::core::defn :ifk::tag-fired-sum [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length
    (:wat::rete::query
      (:wat::core::match (:wat::rete::fire-rules (:ifk::tag-seed s))
        ((:wat::rete::FireOutcome::Fired __fired) __fired)
        ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds)
          (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None))
        ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still)
          (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
      (:ifk::q-TagSum))))

(:wat::core::defn :user::tag-export [] -> :wat::rete::Export
  (:wat::rete::export (:ifk::tag-compile)))

(:wat::core::defn :user::tag-import-and-fire [e <- :wat::rete::Export] -> :wat::core::i64
  (:ifk::tag-fired-sum (:wat::rete::import e)))

(:wat::core::defn :user::tag-native-fire [] -> :wat::core::i64
  (:ifk::tag-fired-sum (:ifk::tag-compile)))

;; ── THE SLOT HALF — `fold_bucket`'s unpacked path, and why it takes THIS shape ────────────────
;;
;; `slot_i64` sits behind `fold_bucket`, which the accumulate pass reaches only when
;; `group_keys` is EMPTY. That is what makes this arm hard to tamper into, and the reason is
;; worth writing down: `group_keys = from_keys \ token_bound \ operand_keys`, and the operand
;; keys come from the FOLD. Rewriting the fold's key therefore drops the real operand OUT of
;; `operand_keys` and back INTO `group_keys` — so on any ordinary rule a tampered key diverts
;; the pass to `accumulate_value` and `slot_i64` is never called.
;;
;; The one shape that survives the tamper is a `:from` binding NOTHING the token does not
;; already bind: then `from_keys \ token_bound` is empty before the fold is consulted, and
;; `group_keys` stays empty under ANY key. Hence the deliberate three-var join below.
;;
;; The String `:tag` denies `Slotted` a packed row, so `packed_operand_field` returns None and
;; the fold takes the SLOT path rather than the packed one.
;;
;; Untampered want: bucket {v=7} sums to 7, the fence admits it, ONE SlotSum.

(:wat::core::defrecord :ifk::Label   [g <- :wat::core::i64  v <- :wat::core::i64  tag <- :wat::core::String])
(:wat::core::defrecord :ifk::Slotted [g <- :wat::core::i64  v <- :wat::core::i64  tag <- :wat::core::String])
(:wat::core::defrecord :ifk::SlotSum [g <- :wat::core::i64  n <- :wat::core::i64])

(:wat::rete::defrule :ifk::slot-sum-rule
  :when [(:ifk::Label (?g <- :g) (?v <- :v) (?tag <- :tag))
         (?n <- (:wat::rete::acc::sum ?v) :from (:ifk::Slotted (?g <- :g) (?v <- :v) (?tag <- :tag)))
         (:wat::rete::where (:wat::rete::core::i64::= ?n 7))]
  :then [(:ifk::SlotSum :g ?g :n ?n)])

(:wat::rete::defquery :ifk::q-SlotSum :params [] :when [(?f <- :ifk::SlotSum)])

(:wat::core::defn :ifk::slot-rules [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector (:ifk::slot-sum-rule)))

(:wat::core::defn :ifk::slot-queries [] -> (:wat::core::PersistentVector :- [:wat::rete::Query])
  (:wat::core::PersistentVector (:ifk::q-SlotSum)))

(:wat::core::defn :ifk::slot-compile [] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::compile-all (:ifk::slot-rules) (:ifk::slot-queries))
    ((:wat::rete::CompileOutcome::Compiled __session) __session)
    ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type)
      (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None))))

(:wat::core::defn :ifk::slot-seed [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert s
    (:ifk::Label   :g 1 :v 7 :tag "x")
    (:ifk::Slotted :g 1 :v 7 :tag "x"))
    ((:wat::rete::InsertOutcome::Inserted __staged) __staged)
    ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count)
      (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))

(:wat::core::defn :ifk::slot-fired-sum [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length
    (:wat::rete::query
      (:wat::core::match (:wat::rete::fire-rules (:ifk::slot-seed s))
        ((:wat::rete::FireOutcome::Fired __fired) __fired)
        ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds)
          (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None))
        ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still)
          (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
      (:ifk::q-SlotSum))))

(:wat::core::defn :user::slot-export [] -> :wat::rete::Export
  (:wat::rete::export (:ifk::slot-compile)))

(:wat::core::defn :user::slot-import-and-fire [e <- :wat::rete::Export] -> :wat::core::i64
  (:ifk::slot-fired-sum (:wat::rete::import e)))

(:wat::core::defn :user::slot-native-fire [] -> :wat::core::i64
  (:ifk::slot-fired-sum (:ifk::slot-compile)))
