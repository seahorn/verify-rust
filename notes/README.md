# notes

Working notes for things that are true about this project but not visible in the
code: environment constraints, why a pin is a pin, and what breaks when you move
one of them.

These are written to explain the *reasoning*, so that when you hit a variant of a
problem described here you can diagnose it yourself rather than pattern-match on a
command that no longer applies.

- [building-on-ubuntu-24.04.md](building-on-ubuntu-24.04.md) — getting the LLVM 14
  stack to build on a distro that has moved on. Read this if `cmake` fails before
  a single source file compiles.
- [llvm18-migration.md](llvm18-migration.md) — moving from SeaHorn `dev14` to
  `dev18`, why the Rust toolchain has to move with it, and the current state of
  that work.
