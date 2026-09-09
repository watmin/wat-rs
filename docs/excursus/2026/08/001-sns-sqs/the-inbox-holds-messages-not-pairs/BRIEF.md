# BRIEF — the inbox holds messages, not pairs

## The work, in one paragraph

`Topic::publish` currently expands each batch into `msgs × nsubs` bodies, tags every one with its
subscriber index, and pushes the pairs through the inbox — where the worker parses the index back out,
buckets by it, strips it, and delivers. Move the expansion into the worker: publish sends **messages**, the
worker fans them to `sub-addrs`. The subscriber index stops existing as a payload field, the `rem`/`need`
top-up is deleted, and the publisher's admission stops depending on `nsubs`.

## Read in order

1. **`DESIGN.md` beside this file** — the round-trip table, the four things this deletes, and the three
   settled rulings. It is authoritative over anything below.
2. **`wat-scripts/topic/sns-fanout.wat:88-118`** — `Topic::publish`. `:93` is the `msgs × nsubs` fold
   emitting `"{i}|{msg}|{t0b}"`; `:113` is the `send-all` into the inbox.
3. **`wat-scripts/topic/sns-fanout.wat:118-150`** — `floor = pairs / nsubs`, `rem`, and the `need`
   top-up. **All of this goes.** With messages, the accepted count *is* the message count.
4. **`wat-scripts/topic/sns-fanout.wat:476-580`** — the worker. `:476` receive, `:503` `split body "|"`,
   ~`:540` bucket by `i`, `:548` re-frame dropping `i`, `:560` `Queue/send` to sub `i`, **`:579` ack the
   inbox**. The expansion moves *here*; the ack ordering stays exactly as it is.
5. **`wat-scripts/topic/sns-fanout.wat:63-66`** — the topic's `:durable`. `nsubs` and `inbox-addr` live
   here; `sub-addrs` is durable-safe by the same rule (an `Address` is EDN-expressible).

## Implementation sketch

```wat
;; publish — no expansion, no index tag
bodies (:wat::core::foldl … (:wat::core::conj acc
          (:wat::core::format "{m}|{t0b}" :m msg :t0b t0b)) … msgs)
;; send-all to the inbox, then reply with the accepted MESSAGE count:
(:demo::Topic::PublishResponse::Accepted accepted)   ;; no floor/rem arithmetic

;; worker — expansion happens here, per received message, per subscriber
;; for each msg in received, for i in 0..nsubs:
;;     (:queue::Queue/send (nth ss i) (SendRequest :queue "q{i}"
;;        :bodies [(format "{m}|{t3b}")] :now-ns …))
;; then, unchanged and AFTER the sends:
(:queue::Queue/ack inb (:queue::Queue::AckRequest :queue "inbox" :ids ack-ids))
```

## Blast radius

`wat-scripts/topic/sns-fanout.wat`, plus `wat-scripts/fanout/circuit.wat` **only** if the report line needs
it. **No `wat/`. No `src/`. No `mem.wat`. No store changes.**

Expect more edit sites than this sketch names — every stone in this campaign has. Report the real count.

## STOP triggers

**STOP-1** — if `distinct ≠ n×m` or `dup ≠ 0` at **any** size, **STOP**. This changes who performs the
fanout; a subscriber missing a message is the failure that matters and `distinct` is the instrument that
detects it. Report the numbers and the per-tier line.

**STOP-2** — do **not** change the ack ordering. The worker sends to the sub queues and *then* acks the
inbox (`:560` before `:579`). That is what makes a mid-expansion death safe: the entry expires, is
re-processed, and duplicates are absorbed by `seen`. **Reversing it loses messages silently.**

**STOP-3** — do **not** change `:cap 64` on the inbox, and do not remove `cap` from the Queue record. Row 6
predicts inbox `refused` drops from 1209–1233 to **0** on its own; changing the cap would destroy that
measurement. If refusals do **not** drop, that is a finding — report it.

**STOP-4** — if the worker cannot reach the subscriber addresses without new plumbing, **STOP and name what
is missing.** It already holds `sub-addrs` and already loops `0..nsubs`, so a gap here is a finding about
the service shape.

**STOP-5** — if the `rem`/`need` top-up cannot be deleted because something else depends on it, **STOP and
report what.** Leaving it as unreachable code that reads as safety is the exact defect class this campaign
has been removing.

**STOP-6** — do **not** raise `:max-entries [msgs 10]`. A batch limit is contract.

**STOP-7** — on any red floor arm: capture it whole, name the exact arm, **do not re-run it.**

## What "done" looks like

`distinct = n×m` and `dup = 0` at n=1000, 2000 and 4000 with `vis-ms=1000` — that is the gate. Inbox
`refused` reported against its 1209–1233 baseline. `Accepted c` demonstrably in messages. The `rem`/`need`
top-up and the `"{i}|"` tag both **absent from the file**. Phases within their known bands (drain 4000/1000
was 4.44–4.58, fill linear, setup ~12.4 s). `every_wat_scripts_file_loads` PASS. Floor Summary read after
the sweep: 5237 / 22 skipped / 0 FAIL / 0 TIMEOUT.

Run your own pre-change baseline on this box, minutes before the change, rather than relying only on the
figures quoted here.

**Write the SCORE to `SCORE.md` beside this file** in the shape of the neighbouring efforts' SCOREs. **Do
not commit.**

State plainly whether the inbox stopped refusing, and report the edit count you actually touched.
