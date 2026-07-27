# stdio-service — a stateful service whose wire IS stdin/stdout

```
./target/release/wat wat-scripts/demos/stdio-service/stdio-service.wat \
  < wat-scripts/demos/stdio-service/session.edn
```

```
#repl.Reply/Value [5]
#repl.Reply/Value [12]
#repl.Reply/Value [12]
#repl.Reply/Value [10]
#repl.Reply/Bye [10]
```

Interactively, one frame per line:

```
./target/release/wat wat-scripts/demos/stdio-service/stdio-service.wat
#repl.Cmd/Bump [5]
#repl.Cmd/Show []
#repl.Cmd/Quit []
```

## The shape

```
:user::main
   └─ TCO →  :repl::serve state
               read ONE frame  →  dispatch  →  reply  →  TCO → :repl::serve state'
```

`main` does one thing: hand control to the loop with the initial state. The
program's entire behaviour is the frame processor.

Every non-terminal arm ends in a **tail call carrying the next state**. TCO turns
that into a jump, so the loop is flat however long the conversation runs — and the
state is a parameter, so there is nothing mutable anywhere.

## Why one frame at a time

Each frame is one **MTU on the wire** (512 KiB — `DEFAULT_MAX_FRAME_BYTES`). The
loop is how a bounded reader crosses an unbounded stream without growing a stack.
You never hold the conversation in memory; you hold one frame and the state it
folds into.

## Exhaustive matching is the wall

`:repl::Cmd` is an enum, so the match names **every** variant — a `_` arm on an
enum is illegal here (109's `NOTE-full-enum-match-mandatory-no-wildcard-arm`).
Adding a command later breaks the *build*, rather than falling through at runtime
to a peer who sent something you forgot to handle.

The type of the frame comes from the arms. A frame that is not a `Cmd` fails as a
**located decode error**, not as a value that quietly means something else.

## Relationship to `defservice`

This is the loop `defservice` generates for you, written out.

| | owns the channel | dispatch | state | use when |
|---|---|---|---|---|
| `defservice` | the substrate | derived from a surface | threaded for you | something **dials** you |
| this demo | you (fd 0/1 after handover) | you write it | you thread it | you are a program at the end of a **pipe** |

If another service or a bracket worker dials you, **use `defservice`** —
hand-rolled IPC is precisely what it exists to replace.

This file is the other case, and the substrate draws the line itself: a root
program has no owner-link, and asking for one says so —

> *"no self-peer — `(:wat::program::self-peer)` is only valid inside a spawned
> process service; root has no owner-link"*

So a root program that wants a conversation writes this loop. After handover,
fd 0/1 are simply yours.

## Sibling

`../stream-protocol/` shows the **bounded** case — marked sections, acked frames,
then handover — which is the shape the substrate used to deliver *this* program.
This one is the unbounded, stateful case that comes after.

## The wire form

Variants serialize positionally: `#ns.Type/Variant [a b]`, not a map. A zero-field
variant is `#repl.Cmd/Show []`. The fastest way to learn any wire form is to have a
program `println` the value and read what comes out.
