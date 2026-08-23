;; probe-arc278-surface-registers-service-reads.wat
;;
;; ⛔ SUPERSEDED 2026-08-05 — THE STONE THIS GATED NO LONGER EXISTS. Kept for its
;; substrate question, NOT for its verdict; nothing downstream is waiting on it.
;;
;; It was STEP 0 of DESIGN-STONE-the-surface-speaks-at-expand-time.md (#74), back when
;; #74 proposed an expand-time registry channel so `defservice` could READ the response
;; type a `defsurface` had declared. The builder then ruled the name into LAW
;; (`<Op>Response`, enforced at registration in `synthesize_surface_protocol`), which
;; collapsed the stone: there is nothing left to read, because the concatenation is
;; correct by construction. The runtime RESPONSE-TYPE constant this file's header used
;; to defend is DELETED, and so is the EDN-decode machinery it fed.
;;
;; What survives is the QUESTION below — can a defsurface's expand-time write be seen by
;; a later defservice's expansion? — which is a real property of `MacroRegistry` that no
;; other file asks, and which a future stone may want. It is unanswered, and answering it
;; is not owed to anything.
;;
;; ── THE ONE CLAIM UNDER TEST ─────────────────────────────────────────────────
;; #74 wants a defsurface to write an op->response-type map onto `MacroRegistry`
;; during ITS expansion, and a later defservice to read it during ITS expansion.
;; Most of that is already established and does NOT need re-proving here:
;;
;;   - top-level sequential registration is a DOCUMENTED guarantee, and
;;     `tests/services/probe_arc294_expand_order.wat` is its regression test
;;     (arc 294 item 9a extended it to do/let bodies);
;;   - a defsurface IS destructured at expand time — `hoist_surface_messages`
;;     (src/macros/expand.rs:212-233) walks its `:messages` and hoists each child
;;     through `hoist_top_level_form`, which REGISTERS companion defmacros;
;;   - a defservice has its `:satisfies` target in hand as a token in its own form.
;;
;; What is NOT established is the join: does a defservice's expansion read the SAME
;; registry instance the SURFACE's hoist wrote to? `probe_arc294_expand_order.wat`
;; proves a defservice sees ITS OWN minted companions — not a sibling form's.
;;
;; ⚠ THE SPECIFIC HAZARD, and the reason this cannot be settled by reading:
;;   `src/macros/expand.rs:396` — `let mut scratch = registry.clone();`
;; If any part of surface expansion runs against a CLONE, the write is dropped on the
;; floor and #74 is dead no matter how sound the rest of the design is.
;;
;; ── THE INSTRUMENT ───────────────────────────────────────────────────────────
;; Use the channel that already exists rather than instrumenting a new one. A
;; `defrecord` inside a surface's `:messages` mints a kwargs companion defmacro when
;; the surface is hoisted. So: declare a record ONLY in the surface's `:messages`,
;; then have the SERVICE's handler construct it by its KWARGS spelling.
;;
;; If the handler body expands, the companion registered during the SURFACE's
;; expansion was visible during the SERVICE's expansion — same registry instance,
;; across a top-level form boundary. That IS #74's channel, proven without adding a
;; single line of the thing being proposed.
;;
;; GREEN  -> the join holds; #74's Step 0 passes; draw the brief.
;; RED    -> read the failure. `UnknownFunction`/unresolved on the kwargs spelling
;;           means the write did not survive to the reader. #74 DIES; say so and
;;           leave the constant in place.
;;
;; ⛔ Construct via the KWARGS name (bare `Tally`), never the positional prime
;;    (`Tally'`). The prime is the raw constructor and would resolve through the type
;;    registry at freeze — it would pass whether or not the expand-time channel works,
;;    which is a vacuous gate. The kwargs macro is the ONLY spelling that must have
;;    been registered during expansion for this body to expand at all.

(:wat::core::defsurface :probe::Chan :nature :wat::kernel::Peer
  :messages
  ;; `Tally` is declared HERE and nowhere else. Its kwargs companion can only exist
  ;; if the surface's hoist registered it.
  [(:wat::core::defrecord :probe::Chan::Tally [n <- :wat::core::i64])
   (:wat::core::defrecord :probe::Chan::CountRequest [])
   (:wat::core::defenum :probe::Chan::CountResponse :wat::enum::Pure
     :Ok               [tally <- :probe::Chan::Tally]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String
                        got <- :wat::core::String])]
  :features
  [(count [self <- :probe::Chan  req <- :probe::Chan::CountRequest]
     -> :probe::Chan::CountResponse :max-request-bytes 4096)])

(:wat::service::defservice :probe::chan-svc
  :satisfies :probe::Chan
  :durable   [seen <- :wat::core::i64]
  :ephemeral []
  :impls
  ;; ★ THE ASSERTION IS THIS BODY EXPANDING AT ALL. `(:probe::Chan::Tally :n …)` is the
  ;; KWARGS spelling, which exists only as a defmacro minted while the SURFACE above was
  ;; hoisted. A defservice is a separate top-level form, so reaching it here means the
  ;; write crossed the boundary on the same registry instance — past `scratch.clone()`.
  [(count [s ctx req]
     (:wat::service::Outcome::Reply s
       (:probe::Chan::CountResponse::Ok
         (:probe::Chan::Tally :n (:probe::chan-svc::State/seen s)))))])
