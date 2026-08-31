# Reproductions for stone 5 — the surface-completeness guard's reach

Two files, identical except for one field's type. Both declare `:p::Item` **outside** the
surface's `:messages` and reference it from a message.

```
direct-field-type.wat        :Ok [item  <- :p::Item]                          --check = 1  GUARD FIRES
parametric-field-type.wat    :Ok [items <- (Vector :- [:p::Item])]            --check = 0  GUARD MISSES
```

The second is `wat-queue`'s shape. `Queue::ReceiveResponse::Ok` carries
`(Vector :- [:queue::Envelope])`, so `sqs.wat` freezes clean and fails only at runtime, in a
forked child, as `unknown callee: :queue::Envelope/id`.

**After stone 5, both must fail to freeze**, with the guard's existing message naming `:p::Item`.

They live here rather than under `wat-scripts/` because `every_wat_scripts_file_loads`
type-checks that tree, and `direct-field-type.wat` is RED by design.
