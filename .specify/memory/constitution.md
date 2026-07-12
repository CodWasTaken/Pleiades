<!--
  SYNC IMPACT REPORT

  Version change: N/A (initial ratification) → 1.0.0
  Modified principles: All new — none previously defined
  Added sections:
    - I. Code Quality
    - II. Testing Standards
    - III. User Experience Consistency
    - IV. Performance Requirements
    - V. Discord Platform Principles
    - VI. Agent Safety Principles (Section 2)
    - VII. Development Workflow & Governance (Section 3)
    - Governance, Amendment & Compliance rules
  Removed sections: N/A
  Templates requiring updates: None — all templates are generic and reference no specific principle names
  Follow-up TODOs: None — all placeholders resolved
-->

# Codder AI Agent Constitution

## Core Principles

### I. Code Quality
Production-grade maintainability MUST be the default for every contribution. The codebase MUST enforce a clear modular architecture with strict separation between the Discord interface, agent reasoning, tools, memory, and execution systems. Unnecessary complexity MUST be avoided in favor of readable and understandable code. Every major feature MUST include documentation covering its purpose, usage, and integration points.

Rationale: Codder operates autonomously — unclear or tightly-coupled code creates latent failure modes that are expensive to diagnose in production.

### II. Testing Standards
All important systems MUST have automated tests. Agent behavior MUST be testable with mocked tools and isolated environments. Tool execution MUST have validation tests that verify correct behavior under expected inputs and error conditions. Database and memory systems MUST have reliability tests that demonstrate data integrity and recovery behavior. Regression tests MUST exist for every existing Codder feature to prevent regressions during development.

Rationale: Autonomous agents have no human-in-the-loop to catch mistakes during operation — tests serve as the primary safety net.

### III. User Experience Consistency
Discord MUST feel like a professional AI assistant interface. Every response MUST be clear, structured, and easy to follow. Long outputs MUST be intelligently split across Discord messages. Agent state MUST always be visible to users. Users MUST understand what Codder is doing, current task progress, which tools are being used, any errors or limitations encountered, and completion status.

Rationale: Transparency builds trust. An opaque agent erodes user confidence and makes debugging interactions impossible.

### IV. Performance Requirements
Token usage MUST be efficient to minimize API costs. Context management MUST prevent unnecessary expansion that would increase costs or exceed model limits. Memory MUST be selective — not everything needs to be stored. Long-running tasks MUST NOT block Discord responsiveness; they MUST yield control to handle new interactions. Background jobs MUST be handled safely with proper error recovery. Discord API limitations including rate limits and payload size MUST be respected at all times.

Rationale: Codder operates under real-world constraints — cost awareness and responsiveness are non-negotiable for a production agent.

### V. Discord Platform Principles
Discord message limits MUST be respected. Important information MUST NEVER be silently truncated. Embeds, threads, pagination, files, and follow-up messages MUST be used when appropriate to present information clearly. Presence/status SHOULD communicate useful agent state without becoming noisy. Rate limits MUST be handled with proper backoff and retry logic.

Rationale: The Discord platform defines the user's interaction surface — violating its constraints produces a broken experience.

## Agent Safety Principles

### VI. Tool Permissions & Execution Safety
Tools MUST require appropriate permissions before execution. Dangerous actions including file deletion, data mutation, and external API writes MUST require explicit user confirmation. The agent MUST explain its planned actions before executing sensitive operations. Every tool call MUST be logged with sufficient context for audit and debugging.

Rationale: Autonomous tool execution carries real risk. Safety constraints prevent accidental damage and enable accountability.

## Development Workflow & Governance

### VII. Amendment & Compliance
Amendments to this constitution require documented rationale, stakeholder approval, and a migration plan for affected systems. The constitution uses semantic versioning as defined in the version line below. Every spec, plan, and task MUST reference the relevant constitution principles. Code review MUST verify principle adherence. Violations MUST be flagged and resolved before merge.

Rationale: The constitution is the project's binding governance document — changes must be deliberate and traceable.

## Governance

This constitution supersedes all other development practices within the Codder project. Amendments require documented rationale, stakeholder approval, and — for material changes — a migration plan for affected systems.

**Versioning Policy**:
- MAJOR: Backward-incompatible governance changes or principle removals/redefinitions.
- MINOR: New principles or materially expanded guidance.
- PATCH: Clarifications, wording refinements, and non-semantic fixes.

**Compliance Review**: Every spec, plan, and task MUST reference the relevant constitution principles. Code review MUST verify principle adherence. Violations MUST be flagged and resolved before merge.

**Version**: 1.0.0 | **Ratified**: 2026-07-12 | **Last Amended**: 2026-07-12
