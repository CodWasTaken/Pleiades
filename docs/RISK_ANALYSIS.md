# Pleiades Risk Analysis

## Risk Assessment Matrix

| ID | Risk | Probability | Impact | Severity | Mitigation |
|----|------|-------------|--------|----------|------------|
| R1 | Provider API changes break compatibility | Medium | High | High | Abstract provider interface; version lock; integration tests with each provider; rapid adaptation plan |
| R2 | WASM plugin runtime performance | Medium | Medium | Medium | Benchmark plugin execution; limit resource usage; provide native vs WASM options; caching |
| R3 | Terminal compatibility issues | Low | High | Medium | Extensive testing across terminals; graceful degradation; feature detection; fallback modes |
| R4 | Memory leaks in long sessions | Medium | High | High | Memory profiling CI; leak detection tests; forced compaction; session health monitoring |
| R5 | Security vulnerability in plugin system | Medium | Critical | Critical | WASM sandboxing; permission system; audit logging; security review; fuzz testing |
| R6 | API key exposure | Low | Critical | High | Secure storage guidelines; OS keyring integration; env var enforcement; secret scanning |
| R7 | Performance regressions | Medium | Medium | Medium | Benchmark CI gate; profiler integration; performance test suite; regression alerts |
| R8 | Provider rate limiting | High | Medium | Medium | Queue management; retry with backoff; provider rotation; user notification |
| R9 | Cross-platform build failures | Medium | Medium | Medium | Multi-platform CI; containerized builds; platform-specific testing; hardware CI runners |
| R10 | Community fragmentation | Low | High | Medium | Clear governance; plugin ecosystem; extension-focused architecture; strong documentation |
| R11 | Feature creep / scope bloat | High | High | High | Strict milestone discipline; feature flags; regular scope review; MVP focus per milestone |
| R12 | Documentation obsolescence | Medium | Medium | Medium | Docs-as-code; CI docs validation; ADR process; mdbook automation |
| R13 | Testing coverage gaps | Medium | High | High | Coverage gates in CI; mandatory review; test-driven development; property-based testing |
| R14 | User adoption barriers | Medium | High | Medium | Excellent onboarding; clear quickstart; compelling README; comparison docs; screenshots |
| R15 | Dependency supply chain | Medium | High | High | Cargo audit CI; dependency pinning; minimal dependency principle; regular updates; SBOM |

## Detailed Risk Analysis

### R1: Provider API Changes
**Description**: AI providers frequently update their APIs, deprecate endpoints, or change response formats, potentially breaking provider implementations.

**Impact**: Users cannot use their preferred provider until compatibility is restored.

**Mitigation**:
- Abstract provider interface isolates changes to single implementation
- Integration tests run daily against live APIs to detect breakage early
- Version pinning per-provider with clear compatibility documentation
- Community-driven provider maintenance via plugin system

### R5: Plugin Security
**Description**: Malicious or buggy plugins could read sensitive files, execute arbitrary commands, or exfiltrate data.

**Impact**: Complete system compromise; loss of user trust; legal liability.

**Mitigation**:
- WASM sandboxing with no direct filesystem or network access
- Capability-based permission system (user must grant each permission)
- Plugin manifest must declare all required permissions
- Signature verification for published plugins
- Auditable plugin source code requirement for marketplace
- Runtime resource limits (CPU, memory, execution time)

### R6: API Key Security
**Description**: API keys could be exposed through error messages, logging, crash reports, or insecure storage.

**Impact**: Financial loss; unauthorized AI usage; account compromise.

**Mitigation**:
- Env var references preferred over inline keys
- OS keyring integration for secure storage
- Keys filtered from all output, logs, and error messages
- Clear documentation on API key security best practices
- `pleiades doctor` checks for common key exposure issues

### R11: Scope Bloat
**Description**: The project's ambition could lead to trying to do everything at once, resulting in half-finished features and burnout.

**Impact**: Never reaching a stable release; abandoned project; lost community trust.

**Mitigation**:
- Strict milestone discipline — complete one before starting next
- Clear MVP definition for each milestone
- Feature flags to hide incomplete features
- Regular scope review at milestone completion
- "No" is a strategic decision, not a failure

## Dependency Analysis

### Direct Dependencies (Planned)

| Crate | Purpose | Risk Level | Alternative |
|-------|---------|------------|-------------|
| tokio | Async runtime | Low | smol, async-std |
| clap | CLI argument parsing | Low | structopt, argparse |
| ratatui | Terminal UI | Medium | cursive, termion |
| crossterm | Terminal manipulation | Low | termion (Unix-only) |
| serde | Serialization | Low | ron, bincode |
| serde_json | JSON support | Low | simd-json |
| toml | TOML support | Low | - |
| serde_yaml | YAML support | Low | - |
| reqwest | HTTP client | Low | ureq, isahc |
| syntect | Syntax highlighting | Low | - |
| wasmtime | WASM runtime | Medium | wasmer |
| rusqlite | SQLite for memory | Low | sled, redb |
| notify | File watching | Medium | - |
| tui-textarea | Full-screen multiline editing | Medium | Snapshot, resize, paste, and input-state tests |
| criterion | Benchmarking | Low | - |
| insta | Snapshot testing | Low | - |
| proptest | Property testing | Low | - |
| tarpaulin | Code coverage | Low | - |

### Dependency Management Strategy

1. **Minimal dependencies principle**: Only add a dependency when the cost of maintaining custom code exceeds the dependency risk
2. **Pin major versions**: Cargo.lock committed; dependabot for automated updates
3. **Audit regularly**: `cargo audit` in CI; security alerts enabled
4. **Prefer Rust-native**: Avoid C bindings where possible to reduce build complexity
5. **Feature gating**: Heavy dependencies behind feature flags (e.g., WASM, metrics)

## Technical Debt Prevention

| Practice | Implementation |
|----------|---------------|
| Code review | All PRs reviewed; checklist enforced |
| Linting | Clippy with aggressive settings; fail CI on warnings |
| Formatting | rustfmt enforced; formatting check in CI |
| Testing | 90%+ coverage; no PR merges without tests |
| Documentation | All public items documented; docs CI check |
| Architecture | ADRs for significant decisions; architecture review |
| Refactoring | Continuous refactoring as part of every milestone |
| Dead code | No dead code ever committed; unused code detector in CI |

## Contingency Plans

### Critical Path Failure
If a core component cannot be delivered as planned:
1. Identify fallback strategy (e.g., simpler implementation)
2. Communicate timeline impact transparently
3. Community can contribute via plugin system if appropriate
4. Consider alternative implementation approach

### Developer Attrition
If a key contributor becomes unavailable:
1. Well-documented architecture reduces bus factor
2. Modular design allows parallel work
3. Comprehensive tests serve as documentation
4. Onboarding documentation for new contributors

### Market Changes
If the competitive landscape shifts:
1. Provider-agnostic design allows rapid adaptation
2. Plugin system enables community to fill gaps
3. Open-source nature allows forking if direction diverges
4. Regular roadmap review and adjustment
