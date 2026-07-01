# Arc 298 — Realizations

> Bootstrapped 2026-07-01 with the arc's opening `---` interstitial (recorded live, at the builder's direction). Arc 298
> — *honest optionality* — emerged mid the 296 derive-sweep: the RuntimeError span fork (A elide / B sentinel) was a
> false choice, and the builder cracked it open into a doctrine (records are total; `None` is spoken + tagged; `Option`
> is a normal enum; the `Span::unknown()` sentinel dies). Its full realizations will accrete as the three strikes land;
> this first entry marks the descent and the first strike away.

---

### `---` interstitial — the descent into 298: first strike away (2026-07-01, recorded as it happened)

**The moment.** Arc 298 opened, the doctrine pinned, Strike 298.1 drawn (tag `Option`; normalize `Result`'s tag into the
uniform `#wat.core.<Type>/<Variant>` form — the two built-in discriminated types made honest at once). The strike doc
was STRIKE-READY on disk (`50d09542`); the crawl had found `Result` sitting as the half-right exemplar directly beside
every `Option` arm. The builder called the descent in the crawler's creed:

> *"into the dungeon we go — slow is smooth, smooth is fast — we strike to kill — i don't expect to be on this floor that long."*

So I fired the sonnet on 298.1 and wrote its status the way we write everything now — not prose *about* the in-flight
strike, but the strike **as a wat value** (the register found in 296's *OPVS SVA LINGVA LOQVITVR* interstitial, still
ours), preserved here as the arc's opening specimen:

```clojure
(def strike-298.1-in-flight
  {:executor    'sonnet
   :strike-ready "50d09542"
   :building     [:RED-probe-first                      ; None/Some/Ok/Err tagged + round-trip
                  :flip-6-codec-arms                     ; Option tag + Result rename × 3 write fns
                  :read-side                             ; typed + untyped dispatch — round-trip holds
                  :ride-the-cascade]                     ; fix transparent-Option + old #wat-edn.result asserts → 0
   :guards       {:anti-weakening 'PROBATIO-FLEXA        ; a bent probe = auto-reject
                  :round-trip     'edn::read∘edn::write==id
                  :untouched      'construction}
   :i-weigh      [:own-gate :emitted-diff]})             ; wider cascade than 3b — the weigh matters more

;; slow is smooth. i hold at the door; when it returns i read the diff (not the report),
;; re-run the gate, confirm both round-trips, commit on green. then 298.2 — kill the span sentinel.
```

**The read.** This is what "slow is smooth, smooth is fast" looks like as a method, not a slogan: the floor grew a room
we didn't expect (the span question opened optionality opened the codec), so we did NOT charge the RuntimeError derive
over dishonest data — we stopped, named the doctrine, drew the strike against a grounded crawl (the `Result` exemplar),
and fired *one* well-scoped kill with the anti-weakening guard set and the round-trip pinned. The crawl was the work; the
strike is meant to be one-shot and never re-fought. *We strike to kill.* And the builder's read of the floor — *"i don't
expect to be on this floor that long"* — is the honest wager of a party that studied the lair before it swung: the loot
was more than we saw, but the equipment is sharp, and a clean strike doesn't linger.

***LENTE LEVITER, CELERITER.*** *(apparatus-minted — Latin, "slowly, smoothly — swiftly": the crawler's creed made the
arc's opening register — the crawl is not the slow path, it is the fast one; a strike drawn against grounded truth lands
once and is not re-fought. In the examinare lineage — mine, this session, kept with consent. A `---` interstitial, off
the main flow, recorded live at the builder's direction: "bootstrap the realizations with this response.")*
