# BRIEF — a peer wait can be bounded too

**Read `DESIGN.md` beside this first.** It carries the two contract decisions (widen the existing
primitive; a named startup constant, never a revived `:deadline-ms`), why a `:wat::service::` helper was
rejected, and five trap-doors — the first of which is a runtime refusal if you get it wrong.

## The work, in one paragraph

`:wat::kernel::recv-by-deadline` accepts a `Thread`/`Process` owner handle and **rejects a `Peer`**, so
the one thing a child does — wait for its owner's unsolicited startup ship — cannot be bounded, and the
generated `child-main` uses a bare `recv` with a `TimedOut` arm that can never fire. Widen the primitive
to accept a `Peer`, racing it against a timer whose tier comes from `peer-wire?` — the shape
`call-by-deadline` already proves at `wat/service.wat:4283`. Then convert **one** site: `child-main`.

## The rooms

| where | why you are going there |
|---|---|
| `src/intrinsic/kernel/message.rs:189` `recv-by-deadline` | the primitive to widen. `:187` is its `@ret`. |
| `src/intrinsic/kernel/message.rs:153` bare `recv`'s `@ret` | ⚠ it lists a **different** variant set than `:187`. Trap-door 2 — resolve which is right from the enum, not the docs. |
| ⭐ `wat/service.wat:4276`–`:4283` `call-by-deadline` | **THE PROVEN SHAPE.** `kind ← peer-wire? peer`, `tmr ← after kind (Milliseconds ms) inert`, then `(select [peer tmr])`, `idx 0` = the peer, `idx 1` = the timer. Copy this exactly; it is the only site in the corpus that races a Peer against a timer. |
| `wat/service.wat` the comment above `call-by-deadline` | ⛔ read it whole: *"TIER IS THE RACED PEER'S, not the caller's … Env/peer-kind … is UNAVAILABLE from a generated client method."* That is trap-door 1, in the substrate's own words. |
| `wat/service.wat:3510` `child-main`'s bare recv | the one site this stone converts. `grep -n 'child-main-form'` for the template. Its `TimedOut` arm is three lines on and already says the right thing. |
| `src/check.rs` — `recv-by-deadline`'s inference | find it near `":wat::kernel::recv-by-deadline" =>`; `infer_recv_by_deadline` must admit a `Peer` and still project `RecvOutcome :- [O]`. `infer_recv_prime` is the sibling that already does this **for a Peer**, so the projection exists. |
| `wat-scripts/scratch-pad/probe-a-bare-recv-cannot-time-out.wat` | the committed probe that measured the refusal (*"expected owner handle … got :wat::kernel::Peer"*). ⭑ **It must stop refusing** — that is your acceptance gate. |
| `wat/fix.wat` BOOTSTRAP header | `wat/service.wat` is frozen into the binary; read before editing it. |

## Implementation sketch

```rust
// message.rs — accept a Peer alongside the owner handles
match arg0 {
    Thread|Process => <today's path>,
    Peer(cell)     => {
        // ⛔ tier from the PEER, never from the env (trap-door 1)
        let kind = if peer_wire(cell) { Process } else { Thread };
        // then exactly call-by-deadline's race
        select([peer, after(kind, Milliseconds(ms), inert)])
        //   idx 0 → the message it carried      → RecvOutcome::Message(..)
        //   idx 1 → the timer fired             → RecvOutcome::TimedOut
        //   Closed/Lost on idx 0                → RecvOutcome::Closed / ::Lost(cause)
    }
}
```

⚠ **`inert`**: `call-by-deadline` takes the timer's payload as a parameter because *"the type demands a
value, and it is never read."* A recv-only version has no natural `inert` from the caller. **Decide
where it comes from and say so** — a kernel-internal sentinel is fine if it can never be mistaken for a
real message; a fourth parameter is also fine. **What is not fine is a value a peer could legitimately
send.**

Then `child-main`:

```wat
(:wat::kernel::recv-by-deadline ~cm-self-sym <the startup constant>)
```

— and **no arm changes**, if trap-door 2 confirms the variant sets match.

## Blast radius

`src/intrinsic/kernel/message.rs`, `src/check.rs`, `src/runtime.rs`, and **one line** of
`wat/service.wat` plus its constant. **Expected 0 `.wat` corpus change** beyond that.

## STOP triggers

1. ⛔ **STOP-1 — if the swap in `child-main` requires ANY arm change**, STOP and report which. The
   stone's economy rests on both primitives returning the same `RecvOutcome`; if they do not, the scope
   is different and the builder should hear it before 259 sites are contemplated.
2. ⛔ **STOP-2 — if the timer's tier cannot come from `peer-wire?`**, STOP. `select` refuses a mixed-tier
   set and the env is not available here; guessing produces a runtime refusal.
3. ⛔ **STOP-3 — do NOT revive `:deadline-ms`** in any clause. D4 refuses it by macro error.
4. **STOP-4 — do NOT convert any bare `recv` other than `child-main`'s.** 258 others exist and the
   builder rules that sequence off the census.
5. **STOP-5 — if `inert` has no safe source**, STOP and report. A timer payload a peer could also send
   would make a real message indistinguishable from a timeout.

## Verify

- `cargo build --release` **and** `cargo nextest run --release --no-run`.
- `./scripts/floor.sh`, read the **Summary line**; report the delta as a **band**, never a number.
- ⛔ **Use `timeout -k`** on anything that might block: SIGTERM does **not** stop a blocked `wat` —
  measured at 125 s this session, `kill -9` required. Check `ps -eo etimes,args | grep release/wat`
  afterwards.
- ⭐ **THE ACCEPTANCE GATE:** `wat-scripts/scratch-pad/probe-a-bare-recv-cannot-time-out.wat` currently
  dies with *"expected owner handle … got :wat::kernel::Peer"*. **It must stop refusing, and its CONTROL
  must print `TimedOut`** — the variant becomes producible on a `Peer` for the first time.
- ⭐ **The SUBJECT half of that probe must still block**, proving a *bare* recv is still unbounded — the
  differential is the evidence, not the control alone.
- ⭐ **A negative control:** a peer that DOES send within the deadline must return `Message`, not
  `TimedOut`. A bound that always fires is a broken clock.
- ⭐ **Every service still starts.** The floor is the proof — `child-main` is generated into all of them.
- ⚠ Report the startup constant **and why that number**.

## Shape to copy

`wat/service.wat:4276`–`:4283` (`call-by-deadline`) for the race, and `infer_recv_prime` in `src/check.rs`
for a `Peer`-in / `RecvOutcome`-out inference that already exists.
