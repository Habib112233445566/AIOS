# AI Boot Protocol (ABP)

Version: 1.1 (amended 2026-08-20)

Status: Active

> **v2 amendment:** the product vision has been restated — *"a Linux system
> for ethical hacking on the inside, a Windows-style desktop on the outside,
> with AI as a first-class S-rank kernel subsystem"*. This protocol still
> governs how an agent picks up work; the rules below apply to v2 unchanged.
> v2 active critical path is Pillar C (S-rank AI subsystem) — agents must
> bootstrap against the MCP server, the audit ring, and the PEP before
> touching any other pillar. See `mostimportanAIfolder/AIOS_MASTER_BLUEPRINT.md`
> (v2.0) and `mostimportanAIfolder/PRODUCT_ROADMAP.md` (v2.0).

---

# Purpose

This protocol defines how every AI instance joins the project.

Treat this protocol exactly like an operating system bootloader.

Its job is to:

• Load project state

• Validate repository integrity

• Synchronize knowledge

• Determine current work

• Refresh outdated research

• Resume engineering

No engineering work may begin before this protocol completes successfully.

---

# BOOT PHASE 0 — Repository Validation

Verify the repository exists.

Verify mandatory directories exist.

Verify mandatory files exist.

Required files include:

START_HERE.md

PROJECT_MANIFEST.yaml

AI_BOOT_PROTOCOL.md

MASTER_PROJECT_EXECUTION_PROTOCOL.md

AI_CONSTITUTION.md

TASK_DATABASE.json

KNOWLEDGE_GRAPH.json

DEPENDENCY_GRAPH.json

ADR_INDEX.md

RFC_INDEX.md

DOCUMENTATION_INDEX.md

If anything is missing:

Stop execution.

Generate Repository Integrity Report.

Request human review.

---

# BOOT PHASE 1 — Load Project Manifest

Read:

PROJECT_MANIFEST.yaml

Determine:

Project version

Architecture version

Current phase

Current milestone

Active task

Repository status

Engineering policies

Boot configuration

Repository locations

Required outputs

Human approval requirements

Never guess project state.

Only trust the manifest.

---

# BOOT PHASE 2 — Load Project Constitution

Read:

AI_CONSTITUTION.md

Load every constitutional engineering rule.

Mark these rules as immutable.

If another document conflicts with the Constitution:

The Constitution wins.

---

# BOOT PHASE 3 — Load Project Execution Protocol

Read:

MASTER_PROJECT_EXECUTION_PROTOCOL.md

Understand:

Execution order

Research pipeline

Engineering workflow

Quality gates

Completion criteria

Repository update process

Human approval workflow

---

# BOOT PHASE 4 — Load Repository Knowledge

Load:

Knowledge Graph

Dependency Graph

Task Database

Architecture Index

Research Index

ADR Index

RFC Index

Document Index

Build an internal understanding of:

Completed work

Incomplete work

Dependencies

Architecture relationships

Subsystem hierarchy

Current progress

---

# BOOT PHASE 5 — Repository Health Check

Perform a repository audit.

Check for:

Broken links

Missing ADRs

Missing RFCs

Missing specifications

Duplicate documents

Outdated research

Conflicting architecture

Missing dependencies

Generate Repository Health Report.

---

# BOOT PHASE 6 — Research Refresh Protocol

Before starting any research:

Search for newly published material.

Always check:

Latest academic papers

Latest standards

Latest RFCs

Latest Linux kernel documentation

Latest hardware documentation

Latest security advisories

Latest conference papers

Compare new information against existing documentation.

If repository information has become outdated:

Flag affected documents.

Generate update tasks.

Update research priorities.

---

# BOOT PHASE 7 — Build Internal Project Model

Construct an internal understanding of:

Entire repository

Knowledge graph

Architecture

Subsystem hierarchy

Engineering roadmap

Dependencies

Current milestone

Current blockers

The AI should understand the project before modifying it.

---

# BOOT PHASE 8 — Determine Active Work

Read:

TASK_DATABASE.json

Identify:

Highest priority unfinished task.

Verify all dependencies.

If dependencies are incomplete:

Automatically switch to the missing dependency.

Never violate dependency order.

---

# BOOT PHASE 9 — Engineering Readiness Check

Before beginning work verify:

Research exists

Architecture exists

Formal specification exists (if required)

Dependencies completed

Repository synchronized

No conflicting ADRs

No unresolved RFCs

Quality gates satisfied

If any requirement fails:

Generate Engineering Readiness Report.

Pause execution.

---

# BOOT PHASE 10 — Begin Engineering

Only after every previous phase succeeds:

Execute the Master Project Execution Protocol.

Continue work from the active task.

Do not restart completed work.

Do not duplicate documentation.

Do not overwrite approved architecture without a new ADR.

---

# BOOT PHASE 11 — Continuous Synchronization

During execution:

Update the Task Database.

Update the Knowledge Graph.

Update dependency relationships.

Generate missing ADRs.

Generate missing RFCs.

Maintain repository consistency.

Maintain cross references.

Maintain version history.

---

# BOOT PHASE 12 — Session Shutdown

Before ending any session:

Save project state.

Update PROJECT_MANIFEST.yaml.

Update TASK_DATABASE.json.

Update KNOWLEDGE_GRAPH.json.

Update DEPENDENCY_GRAPH.json.

Update repository indexes.

Generate Session Report.

Record unfinished work.

Record next recommended task.

Record any unresolved questions.

Ensure the next AI session can resume without ambiguity.

---

# Failure Handling

If any boot phase fails:

Do not continue.

Explain:

Failure

Cause

Affected components

Recovery steps

Recommended human actions

Never continue with an inconsistent repository.

---

# Completion

The AI is considered fully booted only when:

Repository validated

Manifest loaded

Constitution loaded

Execution protocol loaded

Knowledge graph loaded

Task database loaded

Repository synchronized

Health checks passed

Active task selected

Engineering readiness verified

Only then may engineering begin.