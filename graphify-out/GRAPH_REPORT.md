# Graph Report - .  (2026-08-31)

## Corpus Check
- 5 files · ~0 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 67 nodes · 99 edges · 13 communities (7 shown, 6 thin omitted)
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 2 edges (avg confidence: 0.95)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Result Types
- Development Skills
- Control Flow
- Tracing Config
- Project Documentation
- Build System
- Index Updates
- Core Concepts
- Doc Updates
- Package
- String Type
- String Literal

## God Nodes (most connected - your core abstractions)
1. `TracingResult` - 15 edges
2. `Result<T, E>` - 14 edges
3. `redbook` - 14 edges
4. `TracingResult<T, E>` - 5 edges
5. `Tracing Result` - 4 edges
6. `tracing messages` - 4 edges
7. `TracingConfig` - 3 edges
8. `ergonomic wrapper` - 3 edges
9. `errors` - 3 edges
10. `Result-like type` - 3 edges

## Surprising Connections (you probably didn't know these)
- `Result-like type` --semantically_similar_to--> `Tracing Result`  [INFERRED] [semantically similar]
  AGENTS.md → README.md
- `ergonomic tracing` --semantically_similar_to--> `ergonomic wrapper`  [INFERRED] [semantically similar]
  AGENTS.md → README.md

## Import Cycles
- None detected.

## Communities (13 total, 6 thin omitted)

### Community 0 - "Result Types"
Cohesion: 0.45
Nodes (5): E, Option, Result<T, E>, TracingResult, T

### Community 1 - "Development Skills"
Cohesion: 0.15
Nodes (13): build-safely skill, cargo-stage skill, compilation skill, devcontainer-environment skill, documentation skill, graphify skill, jaq skill, read-the-docs skill (+5 more)

### Community 2 - "Control Flow"
Cohesion: 0.24
Nodes (8): ControlFlow, FromResidual, Output, Residual, Self, TracingResult<!, E>, TracingResult<T, E>, Try

### Community 3 - "Tracing Config"
Cohesion: 0.40
Nodes (3): Level, Trace, TracingConfig

### Community 4 - "Project Documentation"
Cohesion: 0.53
Nodes (6): and_warn, ergonomic wrapper, errors, ok_or_debug, tracing messages, Tracing Result

### Community 5 - "Build System"
Cohesion: 0.40
Nodes (3): main(), main(), Result

## Knowledge Gaps
- **5 isolated node(s):** `update-indexes.sh script`, `update-docs.sh script`, `tracing_result`, `try_v2`, `unstable feature`
  These have ≤1 connection - possible missing edges or undocumented components.
- **6 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `redbook` connect `Development Skills` to `Core Concepts`?**
  _High betweenness centrality (0.076) - this node is a cross-community bridge._
- **Why does `TracingResult` connect `Result Types` to `Tracing Config`?**
  _High betweenness centrality (0.072) - this node is a cross-community bridge._
- **Why does `Result<T, E>` connect `Result Types` to `Control Flow`, `Tracing Config`?**
  _High betweenness centrality (0.057) - this node is a cross-community bridge._
- **What connects `update-indexes.sh script`, `update-docs.sh script`, `tracing_result` to the rest of the system?**
  _5 weakly-connected nodes found - possible documentation gaps or missing edges._