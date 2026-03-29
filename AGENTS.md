# Agent Instructions

## Upstream Grounding

Always ground local changes on the upstream KiCad source code at `/Users/Daniel/Desktop/kicad`.

This CLI must be 100% compatible with KiCad, so behavior, data handling, and command semantics should be derived from the upstream KiCad repository rather than inferred locally when compatibility matters.

When studying upstream code, look for the proper abstraction rather than copying isolated behavior mechanically. Build matching abstractions locally where appropriate so compatibility is preserved through the design, not only through one-off patches.
