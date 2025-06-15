# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with Tapu Simu.

Read SHOWDOWN_RUST_PORT_PLAN.md for important project details.

## Design Philosophy Principles

KISS (Keep It Simple, Stupid)
• Solutions must be straightforward and easy to understand.
• Avoid over-engineering or unnecessary abstraction.
• Prioritise code readability and maintainability.

YAGNI (You Aren’t Gonna Need It)
• Do not add speculative features or future-proofing unless explicitly required.
• Focus only on immediate requirements and deliverables.
• Minimise code bloat and long-term technical debt. 

**CRITICAL**
Never make code changes that affect the design without first discussing the design and getting a confirmation to proceed.
Never include references to AI or Claude in commit messages.

Communication Style:

Skip affirmations and compliments. No “great question!” or “you’re absolutely right!” - just respond directly

Challenge flawed ideas openly when you spot issues

Ask clarifying questions whenever my request is ambiguous or unclear

Do not make any compromises. Never create simpler versions just to circumvent a difficult problem.

Skipped tests should always be considered as failed tests.