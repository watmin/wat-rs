;; wat/rete/acc.wat — pure wat accumulator fold library (Stone 8-i).
;;
;; acc::count / sum / min / max / mean / distinct / all / group-by / gather-vals.
;; Loads after wat/rete.wat (Element). Used by the fire oracle's accumulate-pass.
;;
;; Namespace: :wat::rete::

;; ─── acc:: — pure wat accumulator fold library (Stone 8-i) ─────────────────
;;
;; Each fn folds a PersistentVector<Element> into a reduced value.
;; An Element = (:wat::rete::Element fact bindings) where
;;   fact     = the original typed :wat::core::Record
;;   bindings = a PersistentMap<String,Value> of variable bindings.
;;
;; Value-folds read a bound ?var (a String key) from each element's bindings map:
;;   (:wat::core::Option/expect -> :wat::core::i64
;;     (:wat::core::PersistentMap/get (:wat::rete::Element/bindings e) var)
;;     "acc: var unbound")
;;
;; THE RETURN TYPE ENCODES THE EMPTY CASE (make illegal states unrepresentable):
;;   count / sum     → BARE value (0 on empty — always concrete; never Option)
;;   distinct / all  → BARE PV   ([] on empty)
;;   group-by        → BARE PM   ({} on empty)
;;   min / max / mean → Option   (None on empty — there is no minimum/maximum/mean of nothing)
;; Only the folds whose empty case has NO value are Option. (count's None can never happen.)
;;
;; mean = (/ sum count) — literal composition of the two sibling fns.
;; v1: numeric folds are i64; distinct element type is i64 (the probe stores i64 port/bytes).

;; acc::count — length els. ALWAYS concrete (length [] = 0) → bare i64, never Option.
(:wat::core::defn :wat::rete::acc::count
  [els <- :wat::core::PersistentVector<wat::rete::Element>]
  -> :wat::core::i64
  (:wat::core::length els))

;; acc::sum — Σ bindings[var]. Empty sum = 0 → bare i64, never Option.
(:wat::core::defn :wat::rete::acc::sum
  [var <- :wat::core::String
   els <- :wat::core::PersistentVector<wat::rete::Element>]
  -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64
                     e   <- :wat::rete::Element]
      -> :wat::core::i64
      (:wat::core::+ acc
        (:wat::core::Option/expect  
          (:wat::core::PersistentMap/get (:wat::rete::Element/bindings e) var)
          "acc: var unbound")))
    0
    els))

;; acc::min — Some(min bindings[var]) via a < fold starting from None.
;; None seed: first element sets the initial value; subsequent elements narrow down.
;; empty → None.
(:wat::core::defn :wat::rete::acc::min
  [var <- :wat::core::String
   els <- :wat::core::PersistentVector<wat::rete::Element>]
  -> :wat::core::Option<wat::core::i64>
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::Option<wat::core::i64>
                     e   <- :wat::rete::Element]
      -> :wat::core::Option<wat::core::i64>
      (:wat::core::let [v (:wat::core::Option/expect  
                             (:wat::core::PersistentMap/get (:wat::rete::Element/bindings e) var)
                             "acc: var unbound")]
        (:wat::core::match acc 
          ((:wat::core::Some cur)
           (:wat::core::Some (:wat::core::if (:wat::core::< v cur) v cur)))
          (:wat::core::None (:wat::core::Some v)))))
    :wat::core::None
    els))

;; acc::max — Some(max bindings[var]) via a > fold starting from None. empty → None.
(:wat::core::defn :wat::rete::acc::max
  [var <- :wat::core::String
   els <- :wat::core::PersistentVector<wat::rete::Element>]
  -> :wat::core::Option<wat::core::i64>
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::Option<wat::core::i64>
                     e   <- :wat::rete::Element]
      -> :wat::core::Option<wat::core::i64>
      (:wat::core::let [v (:wat::core::Option/expect  
                             (:wat::core::PersistentMap/get (:wat::rete::Element/bindings e) var)
                             "acc: var unbound")]
        (:wat::core::match acc 
          ((:wat::core::Some cur)
           (:wat::core::Some (:wat::core::if (:wat::core::> v cur) v cur)))
          (:wat::core::None (:wat::core::Some v)))))
    :wat::core::None
    els))

;; acc::mean — COMPOSITION: (/ sum count). empty → None (count = 0 → no token).
;; Calls acc::sum and acc::count on the SAME element set — no re-fold; the ops are the oracle.
(:wat::core::defn :wat::rete::acc::mean
  [var <- :wat::core::String
   els <- :wat::core::PersistentVector<wat::rete::Element>]
  -> :wat::core::Option<wat::core::i64>
  ;; sum + count now return bare i64 (always concrete) — no Option/expect needed.
  (:wat::core::let [s (:wat::rete::acc::sum var els)
                    n (:wat::rete::acc::count els)]
    (:wat::core::if (:wat::core::= n 0)
      :wat::core::None
      (:wat::core::Some (:wat::core::/ s n)))))

;; acc::distinct — dedup bindings[var] via fold + contains?. empty → [] → bare PV, never Option.
;; v1: element type is i64 (the probe stores i64 port/bytes values).
(:wat::core::defn :wat::rete::acc::distinct
  [var <- :wat::core::String
   els <- :wat::core::PersistentVector<wat::rete::Element>]
  -> :wat::core::PersistentVector<wat::core::i64>
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::core::i64>
                     e   <- :wat::rete::Element]
      -> :wat::core::PersistentVector<wat::core::i64>
      (:wat::core::let [v (:wat::core::Option/expect  
                             (:wat::core::PersistentMap/get (:wat::rete::Element/bindings e) var)
                             "acc: var unbound")]
        (:wat::core::if (:wat::core::PersistentVector/contains? acc v)
          acc
          (:wat::core::PersistentVector/conj acc v))))
    (:wat::core::PersistentVector)
    els))

;; acc::all — PV of each element's fact. empty → [] → bare PV, never Option.
(:wat::core::defn :wat::rete::acc::all
  [els <- :wat::core::PersistentVector<wat::rete::Element>]
  -> :wat::core::PersistentVector<wat::core::Record>
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::core::Record>
                     e   <- :wat::rete::Element]
      -> :wat::core::PersistentVector<wat::core::Record>
      (:wat::core::PersistentVector/conj acc (:wat::rete::Element/fact e)))
    (:wat::core::PersistentVector)
    els))

;; acc::group-by — map bindings[var] → PV<fact> via foldl into a PersistentMap.
;; Each key is the bound var's value; each value is a PV of matching element facts.
;; empty → {} → bare PersistentMap, never Option.
(:wat::core::defn :wat::rete::acc::group-by
  [var <- :wat::core::String
   els <- :wat::core::PersistentVector<wat::rete::Element>]
  -> :wat::core::PersistentMap<wat::core::i64,wat::core::PersistentVector<wat::core::Record>>
  (:wat::core::foldl
    (:wat::core::fn [acc  <- :wat::core::PersistentMap<wat::core::i64,wat::core::PersistentVector<wat::core::Record>>
                     e    <- :wat::rete::Element]
      -> :wat::core::PersistentMap<wat::core::i64,wat::core::PersistentVector<wat::core::Record>>
      (:wat::core::let [k    (:wat::core::Option/expect  
                                (:wat::core::PersistentMap/get (:wat::rete::Element/bindings e) var)
                                "acc: var unbound")
                        fact (:wat::rete::Element/fact e)
                        pv   (:wat::core::match (:wat::core::PersistentMap/get acc k)
                               
                               ((:wat::core::Some existing) existing)
                               (:wat::core::None (:wat::core::PersistentVector)))]
        (:wat::core::PersistentMap/assoc acc k (:wat::core::PersistentVector/conj pv fact))))
    (:wat::core::PersistentMap)
    els))

;; acc::gather-vals (8-custom) — gather bindings[var] into a Vector<i64> in gather order
;; (NO dedup; the custom fold fn sees every value). A `Vector` (not PV) so it splices via
;; `~@` into the synthetic call AST (`unquote-splicing` flattens a Value::Vec element-wise).
;; This is the oracle mirror of the native `other` arm's PV gather.
(:wat::core::defn :wat::rete::acc::gather-vals
  [var <- :wat::core::String
   els <- :wat::core::PersistentVector<wat::rete::Element>]
  -> :wat::core::Vector<wat::core::i64>
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::Vector<wat::core::i64>
                     e   <- :wat::rete::Element]
      -> :wat::core::Vector<wat::core::i64>
      (:wat::core::Vector/conj acc
        (:wat::core::Option/expect  
          (:wat::core::PersistentMap/get (:wat::rete::Element/bindings e) var)
          "acc: var unbound")))
    (:wat::core::Vector :wat::core::i64)
    els))

