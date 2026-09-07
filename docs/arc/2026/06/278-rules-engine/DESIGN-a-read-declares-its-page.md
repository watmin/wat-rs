# DESIGN — a read declares its page

**`:max-page [field N]` + generated `<op>-all` for cursor reads.** `wat/service.wat` +
`wat/query.wat` + `wat-scripts/queue/sqs.wat`. The read side of the userland layer, mirroring
`a caller chunks to the declared limit`.

## WHY — we bounded requests and never bounded responses

```
:max-entries   bounds a REQUEST   →  a caller cannot flood the service with entries   ✓
(nothing)      bounds a RESPONSE  →  a caller CAN make the service build any reply    ⛔
```

`Queue::ReceiveResponse::Ok [envelopes <- Vector[Envelope]]` — unbounded, and `limit` is an
argument the *caller* picks. `:limit 1000000` forces the service to materialise a million
envelopes. **That is the same DoS we just closed on the write side, pointed the other way.**

★★ The codebase already knows. Three places say a later stone adds this and none was written:

```
src/types/surface.rs:395    "`:max-page-bytes` in a later stone"
src/types/surface.rs:1215   "a future stone adds `:max-page-bytes`"
src/types.rs:448            "a later stone adds `:max-page-bytes`"
```

## AND NOTHING IN THE TREE FOLLOWS A CURSOR

`Store::ScanIndexRequest` takes `cursor <- Option[String]`; `ScanIndexResponse::Success` returns
`[rows, cursor]`. Textbook pagination — and `sqs.wat:170` passes **`:cursor :wat::core::None`**.
The mechanism is declared, carried in every response, and **never used by any caller**.

★ It is not currently a bug (`take` wants one page), but it is an unexercised contract, and the
next caller that needs a second page has nothing to copy.

## ⛔ THE ONE CONTRACT DECISION — the bound is on the RESPONSE, and `limit` becomes a request

`:max-page [field N]` declares that the op returns **at most N** items in the named response
collection, whatever the caller asked for. `limit` stops being a promise and becomes a *request*
the service may serve short — which is what every paginated API in the world does.

★★ That is what makes a big read safe. Today a caller must pick a small `:limit` by hand to avoid
building a huge reply; with a page bound they can ask for everything and the service decides.

## THE TOOL — and where it is NOT emitted

Emit `<op>-all` **iff** the request carries a `cursor <- Option[String]` **and** the response's
success arm carries the paged collection **and** a `cursor <- Option[String]`.

```
Store::scan-index    cursor ✓  rows+cursor ✓   →  scan-index-all   emitted
Store::scan          cursor ✓  rows+cursor ✓   →  scan-all         emitted
Queue::receive       cursor ✗                  →  nothing          emitted
```

★★★ **`receive` must NOT get an `-all`.** A queue read is *leased, not resumable* — the envelopes
it returns go in-flight, so "follow it to the end" would drain the queue rather than page through
it. The two declarations that make pagination sound are exactly the condition for emitting the
tool, and `receive` does not have them. It gets the **bound** and no tool.

`<op>-all` returns a `:wat::stream::Stream` — the type already exists (`wat/seq.wat:76`), so a
caller folds over it exactly as one would over a paginated enumerator, and pages are pulled lazily
rather than materialised.

## THE SYMMETRY, COMPLETE

| | bound | tool | what the tool hides |
|---|---|---|---|
| **write** | `:max-entries` on the request | `<op>-all` | chunks to the limit, sums the prefix |
| **read** | `:max-page` on the response | `<op>-all` | follows the cursor, yields a `Stream` |

Both: the **surface rejects or truncates**; the **userland tool makes the bound invisible without
violating it**; the tool is emitted **iff** the declarations that make it sound are present.

## WHAT THIS DOES NOT FIX — stated because I nearly claimed it would

The measured **2.4×** at m=8 is the fanout worker's buckets (`:limit 10` over 8 destinations =
1.25 per bucket). `receive` gets a page bound here but **no tool**, and this stone does not change
any call site's `:limit`. **The 2.4× is a separate, measured item** and pretending otherwise would
be the fifth prediction I have got wrong today.

★ What this stone buys the 2.4× is *permission*: once a response is bounded, raising a caller's
`:limit` stops being a DoS risk, and that becomes a safe, measurable change instead of a dangerous
one.

## OUT OF SCOPE — REJECTED

- **`:max-page-bytes`.** The three TODOs name a *bytes* bound; this is a *count* bound, matching
  `:max-entries`. Bytes is the sibling and can follow.
- **Changing any caller's `:limit`.** The measured item, and it needs this stone first.
- **`receive-all`.** Semantically wrong — see above.
