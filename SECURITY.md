# Security policy

## Reporting a vulnerability

aiproof is a security-adjacent tool (it detects credential leaks and
prompt-injection patterns) so I take responsible disclosure seriously.

If you find a security vulnerability in aiproof itself — a bug that lets
a crafted prompt escape analysis, causes RCE, leaks data, or bypasses
autofix safety guarantees — please **do not open a public issue.**

Instead, email: **kristianb43r@gmail.com**

Include:
- A minimal reproducer (prompt file / config / command invocation)
- The version of aiproof (`aiproof --version`)
- Platform (macOS / Linux / Windows + arch)
- Expected vs. actual behavior
- Impact (what can an attacker do?)

I'll acknowledge within **72 hours** and aim to ship a fix within
**14 days** for high-severity issues. I'll credit you in the changelog
unless you prefer anonymity.

## Supported versions

| Version | Supported |
|---|---|
| 0.1.x | ✅ |
| < 0.1.0 | ❌ |

## Scope

**In scope:**
- Crashes, panics, hangs on crafted input
- Rule bypasses that cause real security findings (AIP006, AIP007, AIP008) to be missed
- Autofix bugs that corrupt files or write outside intended paths
- Supply-chain risks in the build pipeline

**Out of scope:**
- False positives (file a regular issue; this is quality, not security)
- Rule coverage gaps on novel patterns we haven't written a rule for yet
- Issues in dependencies of aiproof (report to the upstream project)
- Findings aiproof produces about *your* code — those aren't aiproof
  bugs; they're findings to fix in your project

## Disclosure timeline for findings aiproof reports

When aiproof reports a real credential leak or jailbreak vector in a
third-party open-source project (like the AutoGPT / haystack findings in
the v0.1.0 release), we follow **90-day coordinated disclosure**:

1. File a private security advisory / email to the maintainers with the
   specific finding, line reference, and severity assessment.
2. Wait up to 90 days for a fix.
3. After 90 days (or sooner if the project ships a fix), publish the
   finding in the changelog and any launch materials.

If you find such an issue yourself via aiproof, you're free to file
issues however you like — but please give maintainers time to fix before
going public with active credentials.
