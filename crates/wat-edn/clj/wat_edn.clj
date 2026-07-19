(ns wat-edn
  "wat-edn — install wat's EDN tag vocabulary into Clojure, with real tools.

  The proof of \"wat IS EDN\": an INDEPENDENT reference implementation — Clojure's
  `clojure.edn` — reads every tag wat emits, and reconstructs the value ones as
  proper Clojure data a program can dispatch on. If Clojure reads and handles it,
  wat's output is canonical EDN, validated by the reader the whole Lisp world
  trusts.

  The bridge between the worlds. Vended externally later; for now a local proof.

  Two vocabularies:
    - the CLOSED value vocabulary (Option, Result, Span, Pos) → named readers that
      build real records + the tools to handle them (some?/none?/unwrap, ok?/err?).
    - the OPEN, growing error/diagnostic vocabulary (#wat.check/… #wat.macro/…
      #wat.runtime/… #wat.kernel/… #wat.load/… …) → a structural default that
      builds a WatTagged record preserving the tag. You cannot enumerate a set
      that keeps growing; you handle every #wat.ns/Type by construction. This is
      the general form of what #holon started (arc 294.b).
    - #uuid and #inst are built into clojure.edn."
  (:refer-clojure :exclude [some some? val])
  (:require [clojure.edn :as edn]))

;; ─── Option — the tool Clojure needs to handle wat's Option ─────────────────
;; wat (arc 278 A.0): #wat.core.Option/Some [v] | #wat.core.Option/None []
;; Every variant is vector-bodied; `nil` is the unit value only.
(defrecord Some [value])
(defrecord None [])
(def none (->None))
(defn some  "construct Some" [v] (->Some v))
(defn some? "is it Some?"    [x] (instance? Some x))
(defn none? "is it None?"    [x] (instance? None x))
(defn option? [x] (or (some? x) (none? x)))
(defn unwrap
  "Some→value; None→throw, or the supplied default."
  ([x]         (if (some? x) (:value x) (throw (ex-info "unwrap on None" {:x x}))))
  ([x default] (if (some? x) (:value x) default)))

;; ─── Result — the tool Clojure needs to handle wat's Result ─────────────────
;; wat (arc 278 A.0): #wat.core.Result/Ok [v] | #wat.core.Result/Err [e]
(defrecord Ok  [value])
(defrecord Err [error])
(defn ok  "construct Ok"  [v] (->Ok v))
(defn err "construct Err" [e] (->Err e))
(defn ok?  "is it Ok?"  [x] (instance? Ok x))
(defn err? "is it Err?" [x] (instance? Err x))
(defn result? [x] (or (ok? x) (err? x)))
(defn unwrap-ok
  "Ok→value; Err→throw, or the supplied default."
  ([x]         (if (ok? x) (:value x) (throw (ex-info "unwrap-ok on Err" {:x x}))))
  ([x default] (if (ok? x) (:value x) default)))

;; ─── Span / Pos — wat's source-location value records ───────────────────────
;; wat: #wat.core/Span {:file :line :col :end}   #wat.core/Pos {:line :col}
(defrecord Pos  [line col])
(defrecord Span [file line col end])

;; ─── the OPEN error/diagnostic vocabulary ───────────────────────────────────
;; #wat.check/CheckErrors, #wat.macro/ProgramBodyEvalFailed, #wat.runtime/UnboundSymbol,
;; #wat.kernel/ProcessPanics, … — a growing set. Each is map-shaped; we keep its
;; fields directly addressable (:message :location :causes …) and stamp :wat/tag so
;; the value knows what it is. A non-map tagged value (rare) becomes a WatTagged.
(defrecord WatTagged [tag value])

;; ─── the reader map — the tags, installed ───────────────────────────────────
(def readers
  "Named readers for the closed value vocabulary. Pass to clojure.edn/read-string
  as `:readers`. The open error vocabulary is caught by `default-reader` below."
  ;; Arc 278 A.0 — variant bodies are field-vectors; unwrap the single field.
  {'wat.core/Span         map->Span
   'wat.core/Pos          map->Pos
   'wat.core.Option/Some  (fn [body] (->Some (first body)))
   'wat.core.Option/None  (fn [_] none)
   'wat.core.Result/Ok    (fn [body] (->Ok (first body)))
   'wat.core.Result/Err   (fn [body] (->Err (first body)))})

(defn default-reader
  "Any #wat.ns/Type not in `readers`. Map-shaped (the error/diagnostic vocabulary)
  → the map with :wat/tag stamped, fields directly addressable. Non-map → WatTagged.
  Covers the whole open vocabulary by construction (the #holon generalization)."
  [tag value]
  (if (map? value)
    (assoc value :wat/tag (str tag))
    (->WatTagged (str tag) value)))

(defn read-wat
  "Read a wat EDN string through Clojure's canonical EDN reader. Value tags become
  real records (Some/None, Ok/Err, Span, Pos) with tools to handle them; every
  other #wat tag becomes a WatTagged record; #uuid/#inst are built in. Throws iff
  the input is not well-formed EDN — a clean return IS the proof."
  [s]
  (edn/read-string {:readers readers :default default-reader} s))
