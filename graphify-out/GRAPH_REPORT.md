# Graph Report - /workspaces/tracing_result  (2026-08-30)

## Corpus Check
- 81 files · ~353,726 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 34 nodes · 41 edges · 10 communities (4 shown, 6 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Error Handling
- Control Flow
- Error Propagation
- Core Types
- Build System
- Indexing
- Try FromResidual
- Documentation Scripts
- Result Type

## God Nodes (most connected - your core abstractions)
1. `TracingResult` - 7 edges
2. `Result<T, E>` - 7 edges
3. `TracingResult<T, E>` - 5 edges
4. `main()` - 2 edges
5. `TracingResult<!, E>` - 2 edges
6. `Trace` - 2 edges
7. `update-docs.sh script` - 1 edges
8. `update-indexes.sh script` - 1 edges
9. `tracing_result` - 0 edges

## Surprising Connections (you probably didn't know these)
- `TracingResult<!, E>` --references--> `T`  [EXTRACTED]
  src/lib.rs →   _Bridges community 0 → community 2_
- `Result<T, E>` --implements--> `FromResidual`  [EXTRACTED]
  src/lib.rs →   _Bridges community 6 → community 0_
- `Result<T, E>` --implements--> `Trace`  [EXTRACTED]
  src/lib.rs → src/lib.rs  _Bridges community 0 → community 3_

## Import Cycles
- None detected.

## Communities (10 total, 6 thin omitted)

### Community 0 - "Error Handling"
Cohesion: 0.50
Nodes (6): E, S, Result<T, E>, TracingResult, String, T

### Community 2 - "Error Propagation"
Cohesion: 0.50
Nodes (3): Residual, Self, TracingResult<!, E>

### Community 6 - "Try FromResidual"
Cohesion: 0.67
Nodes (3): FromResidual, TracingResult<T, E>, Try

## Knowledge Gaps
- **3 isolated node(s):** `tracing_result`, `update-docs.sh script`, `update-indexes.sh script`
  These have ≤1 connection - possible missing edges or undocumented components.
- **6 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `TracingResult` connect `Error Handling` to `Core Types`?**
  _High betweenness centrality (0.131) - this node is a cross-community bridge._
- **Why does `Result<T, E>` connect `Error Handling` to `Core Types`, `Try FromResidual`?**
  _High betweenness centrality (0.122) - this node is a cross-community bridge._
- **Why does `TracingResult<T, E>` connect `Try FromResidual` to `Control Flow`, `Error Propagation`?**
  _High betweenness centrality (0.101) - this node is a cross-community bridge._
- **What connects `tracing_result`, `update-docs.sh script`, `update-indexes.sh script` to the rest of the system?**
  _3 weakly-connected nodes found - possible documentation gaps or missing edges._