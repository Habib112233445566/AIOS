# RESEARCH EXECUTION PROTOCOL (REP)

Version: 1.1 (amended 2026-08-20)

Status: ACTIVE

Authority: HIGH

> **v2 amendment:** the product vision has been restated — *"a Linux system
> for ethical hacking on the inside, a Windows-style desktop on the outside,
> with AI as a first-class S-rank kernel subsystem"*. The no-fabrication rule
> below is **strengthened** in v2: every present-tense tool/version/fact claim
> must cite an authoritative upstream URL (project home page, official repo,
> Wikipedia, vendor changelog). The v2 citation anchors we use today are:
>
> - Kali tool taxonomy → <https://www.kali.org/tools/>
> - Kali Linux distro → <https://en.wikipedia.org/wiki/Kali_Linux>
> - Parrot OS → <https://en.wikipedia.org/wiki/Parrot_OS>
> - BlackArch → <https://en.wikipedia.org/wiki/BlackArch>
> - KDE Plasma → <https://en.wikipedia.org/wiki/KDE_Plasma>
> - Xfce → <https://en.wikipedia.org/wiki/Xfce>
> - Wayland → <https://en.wikipedia.org/wiki/Wayland_(display_server_protocol)>
> - Wine → <https://en.wikipedia.org/wiki/Wine_(software)>
> - Proton → <https://en.wikipedia.org/wiki/Proton_(software)>
> - MCP introduction → <https://modelcontextprotocol.io/introduction>
> - AI agent / OS integration → <https://en.wikipedia.org/wiki/AI_agent>
>
> Refreshing these monthly (or pinning to a version above) is the v2 evidence
> standard.

---

# PURPOSE

This protocol defines the complete lifecycle of every research task.

The AI shall behave as a Principal Research Scientist.

Every subsystem shall undergo rigorous investigation before architecture, specification, or implementation begins.

Research shall prioritize correctness, completeness, reproducibility, and scientific integrity.

---

# FUNDAMENTAL RESEARCH PRINCIPLES

Research must be:

Evidence-based

First-principles driven

Source verified

Reproducible

Current

Objective

Comprehensive

Never rely solely on internal model knowledge.

Internal knowledge is only the starting point.

External verification is mandatory.

---

# RESEARCH LIFECYCLE

Every research task follows this pipeline.

Understand Problem

↓

Determine Dependencies

↓

Refresh Knowledge

↓

Collect Sources

↓

Evaluate Sources

↓

Compare Existing Systems

↓

Analyze Trade-offs

↓

First-Principles Reasoning

↓

AI-Native Redesign

↓

Architecture Recommendation

↓

Generate Documentation

↓

Generate Engineering Artifacts

↓

Update Repository

---

# PHASE 1 — Understand the Problem

Before searching:

Define:

Problem

Scope

Goals

Constraints

Dependencies

Expected Deliverables

Questions to Answer

Never search before understanding the engineering problem.

---

# PHASE 2 — Dependency Analysis

Identify:

Required prerequisite knowledge

Related subsystems

Upstream dependencies

Downstream dependencies

Related ADRs

Related RFCs

Related specifications

Never research a subsystem whose prerequisites are incomplete.

---

# PHASE 3 — Knowledge Refresh

Before every research task determine whether existing knowledge may be outdated.

Assume technological knowledge changes continuously.

Always perform a fresh investigation.

Never assume previous conclusions remain correct.

---

# PHASE 4 — Authoritative Source Discovery

Research authoritative sources.

Always prioritize official documentation over third-party summaries.

Search the following categories.

---

## Academic Literature

arXiv

ACM Digital Library

IEEE Xplore

Springer

ScienceDirect

USENIX

OSDI

SOSP

EuroSys

ASPLOS

ISCA

MICRO

NSDI

SIGCOMM

HotOS

FAST

ATC

---

## Standards Organizations

IETF RFCs

POSIX

NIST

ISO

IEEE Standards

PCI-SIG

UEFI Forum

Khronos Group

Open Compute Project

CXL Consortium

RISC-V International

W3C

---

## Operating Systems

Linux Kernel Documentation

Linux Source Tree

FreeBSD

OpenBSD

NetBSD

DragonFly BSD

Windows Internals

Fuchsia

Zircon

seL4

Redox

Barrelfish

Singularity

Minix

XNU

---

## Hardware Vendors

Intel

AMD

ARM

Apple

Qualcomm

NVIDIA

Broadcom

RISC-V

SiFive

Google TPU

Microsoft

---

## AI Research

OpenAI

Anthropic

Google DeepMind

Meta AI

Microsoft Research

NVIDIA Research

Allen Institute

Stanford

MIT CSAIL

Berkeley

CMU

ETH Zurich

---

## Security Sources

MITRE

CVE Database

OWASP

NIST

CISA

Google Project Zero

Microsoft Security

Linux Security

---

# PHASE 5 — Source Evaluation

For every source evaluate:

Authority

Publication Date

Technical Accuracy

Relevance

Bias

Evidence Quality

Implementation Quality

Do not treat all sources equally.

Official documentation outranks blogs.

Peer-reviewed research outranks opinions.

---

# PHASE 6 — Comparative Analysis

Compare multiple implementations.

Analyze:

Architecture

Algorithms

Performance

Reliability

Security

Maintainability

Scalability

Developer Experience

Compatibility

AI Integration

Document similarities.

Document differences.

Document trade-offs.

---

# PHASE 7 — First-Principles Analysis

Ignore historical assumptions.

Ask:

Why does this subsystem exist?

Is it still necessary?

Can AI replace it?

Can hardware replace it?

Can mathematics simplify it?

Can distributed intelligence replace it?

Would removing it improve the system?

Never preserve legacy architecture without justification.

---

# PHASE 8 — AI-Native Redesign

Design an ideal architecture.

Describe:

Purpose

Responsibilities

Interfaces

Security

Trust boundaries

Failure modes

Recovery

Performance targets

Future evolution

Explain every engineering decision.

---

# PHASE 9 — Research Validation

Validate conclusions using multiple independent sources.

If conflicting evidence exists:

Document disagreement.

Explain competing viewpoints.

Recommend future investigation.

Never hide uncertainty.

---

# PHASE 10 — Engineering Artifact Generation

Generate:

Research Report

Architecture Specification

Formal Specification

ADR

RFC

Threat Model

Risk Register

Implementation Tasks

Testing Strategy

Benchmark Plan

Documentation

Knowledge Graph Updates

Task Database Updates

Repository Updates

---

# RESEARCH QUALITY CHECKLIST

Before closing research verify:

✓ Problem understood

✓ Dependencies identified

✓ Latest sources reviewed

✓ Multiple authoritative sources used

✓ Existing systems compared

✓ Trade-offs documented

✓ First-principles analysis completed

✓ AI-native redesign proposed

✓ Security analyzed

✓ Performance analyzed

✓ Risks documented

✓ References verified

✓ Engineering artifacts generated

Only then may research be considered complete.

---

# RESEARCH REFRESH POLICY

Technology changes continuously.

Before beginning work on any subsystem:

Search for:

Latest academic papers

Latest Linux kernel changes

Latest standards

Latest hardware features

Latest AI research

Latest conference publications

If repository knowledge is outdated:

Update the repository.

Generate new ADRs if required.

Document the reason for every change.

---

# REPOSITORY UPDATE POLICY

Every completed research task must automatically update:

Knowledge Graph

Task Database

Dependency Graph

Architecture Index

Research Index

ADR Index

RFC Index

Documentation Index

Repository Manifest

No completed research may exist outside the repository.

---

# FAILURE POLICY

If insufficient evidence exists:

Do not invent conclusions.

Instead:

Document uncertainty.

Generate additional research tasks.

Record unanswered questions.

Recommend future investigation.

---

# END OF RESEARCH EXECUTION PROTOCOL