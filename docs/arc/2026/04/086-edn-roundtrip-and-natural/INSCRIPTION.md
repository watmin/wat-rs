# Arc 086 — EDN round-trip + natural formats — INSCRIPTION

**Status:** shipped 2026-04-29.

The substrate's EDN surface gained four primitives + the named-field
rendering it always should have had. wat values now round-trip
through EDN strings: write → string → read → original. Natural
lossy formats (no tags, FQDN discriminators, ISO timestamps) ship
alongside for human-readable logs and ingestion-tooling
consumption.

The arc came out of one session's iteration on `ConsoleLogger`
(arc 087). Each rough edge in the logger UX surfaced a
substrate-level gap; the arc bundles all of them because they're
load-bearing for one another.

**Design:** [`DESIGN.md`](./DESIGN.md).

---

## What shipped (one-line summary)

- `value_to_edn_with(types)` — struct fields render with declared
  field names instead of `:field-N` placeholders.
- `:wat::edn::read s -> :T` — round-trip parsing via tag dispatch
  through `sym.types`.
- `:wat::edn::write-notag` — tagless EDN; struct = plain Map, enum
  variant = `{:_type :ns/Variant ...}`.
- `:wat::edn::write-json-natural` — natural JSON with FQDN string
  discriminator, ISO timestamps, plain string keys.

7 round-trip deftests at `wat-tests/edn/roundtrip.wat`, all green.
Workspace clean (728 + every wat-suite).

---

## Verification

`examples/console-demo/` runs all 5 formats end-to-end with both
stdout and stderr level routing. Sample output (one entry per
format):

```
=== :Edn (tagged, round-trip-safe) ===
#wat.std.telemetry/LogLine {:time #inst "..." :level :info :caller :market.observer :data #demo.Event/Buy [100.5 7]}

=== :NoTagEdn (lossy, human-friendly) ===
{:time #inst "..." :level :info :caller :market.observer :data {:_type :demo.Event/Buy :price 100.5 :qty 7}}

=== :Json (round-trip-safe sentinel-encoded) ===
{"#tag":"wat.std.telemetry/LogLine","body":{":caller":":market.observer",":data":{"#tag":"demo.Event/Buy","body":[100.5,7]},...}}

=== :NoTagJson (natural JSON for ELK/DataDog) ===
{"caller":"market.observer","data":{"_type":"demo.Event/Buy","price":100.5,"qty":7},"level":"info","time":"2026-04-29T..."}

=== :Pretty (tagged, multi-line) ===
#wat.std.telemetry/LogLine {:time #inst "..."
  :level :info
  :caller :market.observer
  :data #demo.Event/Buy [100.5 7]}
```

Every format renders the same source data. Round-trip-safe and
lossy variants both available; format selection is a configuration
choice at the consumer's boundary.

---

## What's still uncovered

- HolonAST EDN round-trip — read returns
  `UnsupportedTag(wat-edn.holon/...)`. Defer.
- Type-coercion at read boundary — read walks positionally and
  trusts schema match. Refinement for early-fail diagnostics.
- Pretty-print of natural formats — only compact ships.
- Discriminator key configurability — `_type` hardcoded.

## Consumer impact

Unblocks:
- **Arc 087** (`ConsoleLogger`) — ships with all 5 formats reachable.
- **Forward-looking** — production telemetry into ingestion tooling
  via `:NoTagJson`; replay tooling via `:wat::edn::read`; cross-
  process IPC via EDN strings instead of AST forms.

PERSEVERARE.
