# stream-protocol — the legible twin of the substrate's own boot

```
./target/release/wat wat-scripts/demos/stream-protocol/stream-protocol.wat \
  < wat-scripts/demos/stream-protocol/session.edn
```

```
#proto.Ack/Got [0]
#proto.Ack/Got [1]
#proto.Ack/SectionAck [2]
#proto.Ack/Got [0]
#proto.Ack/Got [1]
#proto.Ack/SectionAck [2]
31
11
```

## Why it exists

By the time your `:user::main` runs, the substrate has **already spoken a framed
protocol to your program**: N substrate frames, a marker, N program frames, a
marker, then handover — from that moment stdin and stdout are yours.

That boot is written in Rust, so a student who writes wat cannot read the teacher.
This demo is the same shape, in wat, on the same wire, using the same two verbs
the substrate used: `readln` takes one frame, `println` writes one.

## What you inherit — guarantees, not a style

- **Every frame is one typed EDN value.** A malformed frame is a *located* decode
  error, not a substring that quietly failed to match.
- **Every frame is acked.** "The peer received and accepted this" becomes a fact
  you hold rather than a hope. A bare write tells you bytes left; it does not tell
  you they were understood.
- **A section that can vary in length ends with a marker.** Growing it later is
  then not a wire break — which matters once a parent and child may not be the
  same binary (`170/TIERS.md`: *one protocol; four transports*).
- **Reassembly is concatenation.** A frame boundary may fall mid-form or
  mid-token; the reader never parses content. Meaning is decided once, after the
  marker.

## The trap — this is NOT how to build a service

The loop is a serve loop by hand, and that similarity is the point *and* the
hazard:

| | owns the channel | dispatch | use it when |
|---|---|---|---|
| `defservice` | the substrate | generated from a surface | something dials you |
| this demo | **you** (fd 0/1 after handover) | you write it | you are a program at the end of a pipe |

If you are building something other services talk to, **use `defservice`**.
Hand-rolled IPC is exactly what it exists to replace, and this corpus deleted its
old hand-rolled-service reference for teaching the wrong lesson.

Copy the *shape* — framed values, acked, marked sections. Do not copy this into a
service.

## The session file

`session.edn` is the driving side of the conversation. Variants serialize
positionally, so a two-field variant is `#ns.Type/Variant [a b]` — not a map. The
fastest way to learn any wire form is to have a program `println` the value and
read what comes out.
