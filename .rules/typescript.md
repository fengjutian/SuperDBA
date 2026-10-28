# TypeScript Rules
1. **No Explicit Any:** NEVER use `any`. Define proper interfaces/types or use `unknown` with type guards. If a type is external and unknown, define a local interface that matches the expected structure.
