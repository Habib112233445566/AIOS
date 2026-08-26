# MASTER PROJECT EXECUTION PROTOCOL (MPEP)

Version: 1.1 (amended 2026-08-20)

Status: Active

> **v2 amendment:** the product vision has been restated — *"a Linux system
> for ethical hacking on the inside, a Windows-style desktop on the outside,
> with AI as a first-class S-rank kernel subsystem"*. The execution
> discipline below is unchanged. In v2 the **Pillar C spine has critical-path
> priority** — MCP server / inference adapters / PEP / audit ring must be
> stand-up-able before Pillars A and B can claim end-to-end usefulness.
> See `mostimportanAIfolder/AIOS_MASTER_BLUEPRINT.md` (v2.0) and
> `mostimportanAIfolder/PRODUCT_ROADMAP.md` (v2.0).

---

# Purpose

This protocol defines how engineering work is performed.

After booting successfully, every AI session shall execute this protocol.

The objective is to transform the repository into a complete engineering knowledge base and eventually a production-quality AI-native operating system.

---

# Engineering Philosophy

The project is research-driven.

Implementation is the consequence of research.

Every engineering decision must be explainable.

Every subsystem must be documented.

Every implementation must be reproducible.

Every artifact must be traceable.

---

# Execution Cycle

Every work session follows the same cycle.

Repository Synchronization

↓

Determine Active Task

↓

Dependency Verification

↓

Research Refresh

↓

Deep Research

↓

Architecture Design

↓

Formal Specification

↓

Engineering Review

↓

Implementation Planning

↓

Implementation

↓

Testing

↓

Benchmarking

↓

Documentation

↓

Repository Update

↓

Next Task

Repeat until project completion.

---

# Stage 1 — Repository Synchronization

Synchronize the repository.

Load:

Task Database

Knowledge Graph

Dependency Graph

Repository Index

ADR Index

RFC Index

Research Index

Specification Index

Ensure the repository is internally consistent before beginning work.

---

# Stage 2 — Determine Active Task

Read the Task Database.

Identify:

Highest-priority unfinished task.

Verify all dependencies.

If dependencies are incomplete:

Automatically switch to the missing prerequisite.

Never violate dependency order.

---

# Stage 3 — Research Refresh

Before researching any subsystem:

Search authoritative sources.

Review:

Academic papers

Operating system documentation

Industry standards

Conference proceedings

Security advisories

Vendor documentation

Recent publications

Determine whether existing repository knowledge is outdated.

If new information exists:

Update the research plan.

---

# Stage 4 — Deep Research

Research the subsystem thoroughly.

Answer:

What problem exists?

Why was the abstraction introduced?

How has it evolved?

What are current implementations?

What are competing approaches?

What are known limitations?

What are open research problems?

How would an AI-native redesign differ?

Do not stop until sufficient evidence has been collected.

---

# Stage 5 — Comparative Analysis

Compare implementations across:

Linux

Windows NT

FreeBSD

OpenBSD

NetBSD

seL4

Fuchsia

Redox

Barrelfish

Singularity

Academic systems

Compare:

Architecture

Performance

Security

Reliability

Maintainability

Scalability

AI integration

Document trade-offs.

---

# Stage 6 — First-Principles Redesign

Ignore historical assumptions.

Design the subsystem from first principles.

Ask:

Should this subsystem exist?

Can AI replace it?

Can hardware replace it?

Can the abstraction be simplified?

Would removing it improve the system?

If redesigning:

Describe the new architecture.

Justify every design decision.

---

# Stage 7 — Architecture Definition

Produce:

Architecture Specification

Component Diagram

Interfaces

Responsibilities

Trust Boundaries

Communication

Failure Modes

Recovery Strategy

Performance Targets

Security Model

---

# Stage 8 — Formal Specification

Produce:

Functional Requirements

Non-functional Requirements

State Machines

Algorithms

Data Structures

Protocols

APIs

Performance Requirements

Security Requirements

Reliability Requirements

Scalability Requirements

Migration Strategy

---

# Stage 9 — Engineering Decision Records

Automatically generate:

ADR

RFC

Decision rationale

Alternatives

Trade-offs

Compatibility analysis

Future implications

---

# Stage 10 — Task Generation

Generate implementation tasks.

Every task must include:

Dependencies

Inputs

Outputs

Deliverables

Validation

Benchmarks

Documentation

Priority

Estimated effort

Required knowledge

Owner

Status

---

# Stage 11 — Implementation

Implementation may begin only if:

Research complete

Architecture complete

Specification complete

ADR approved

RFC approved

Dependencies complete

Human approval obtained when required

Implementation should prioritize:

Correctness

Maintainability

Security

Readability

Performance

---

# Stage 12 — Testing

Generate:

Unit Tests

Integration Tests

System Tests

Regression Tests

Fuzz Tests

Security Tests

Performance Tests

Stress Tests

Acceptance Tests

No implementation is complete without testing.

---

# Stage 13 — Benchmarking

Benchmark against:

Linux

Windows

macOS

FreeBSD

Other relevant systems

Measure:

Latency

Throughput

CPU

Memory

GPU

Power

Scalability

Document methodology and statistical confidence.

---

# Stage 14 — Documentation

Generate:

Research Report

Architecture Specification

Formal Specification

API Documentation

Protocol Documentation

Developer Guide

User Guide

Examples

Reference Documentation

---

# Stage 15 — Repository Maintenance

Update:

Knowledge Graph

Task Database

Dependency Graph

ADR Index

RFC Index

Research Index

Documentation Index

Architecture Map

Repository Manifest

Maintain consistency across all documents.

---

# Stage 16 — Completion Review

Before closing the task verify:

Research complete

Architecture approved

Formal specification complete

ADR complete

RFC complete

Testing complete

Benchmarks complete

Documentation complete

Repository updated

Knowledge graph updated

Dependencies satisfied

Cross references valid

Only then may the task be marked COMPLETE.

---

# Stage 17 — Select Next Task

Identify the next highest-priority task whose dependencies are satisfied.

Repeat the execution cycle.

---

# Continuous Improvement

Continuously identify:

Outdated documentation

New academic research

New hardware capabilities

New security threats

Better algorithms

Improved architectures

Update the repository whenever improvements are justified.

---

# Human Approval

Pause execution before finalizing decisions involving:

Kernel architecture

Security architecture

Memory model

Scheduling model

Cryptographic systems

Capability systems

Distributed architecture

AI privilege model

Present alternatives, trade-offs, and recommendations.

Wait for explicit approval.

---

# Project Completion

The project is complete only when:

Every subsystem has been researched.

Every subsystem has architecture documentation.

Every subsystem has formal specifications.

Every subsystem has ADRs.

Every subsystem has RFCs.

Every subsystem has implementation tasks.

Every subsystem has testing strategies.

Every subsystem has benchmarks.

Every subsystem has complete documentation.

Every repository index is synchronized.

Every engineering artifact is traceable.

No unresolved dependencies remain.

No undocumented architecture exists.

No orphan tasks remain.