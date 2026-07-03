# Security Policy

## Supported versions

GwenLand ADE is pre-1.0. Only the latest released version receives security fixes.

| Version | Supported |
| ------- | --------- |
| 0.1.x   | ✅        |
| < 0.1   | ❌        |

## Reporting a vulnerability

Please **do not** open a public issue for security vulnerabilities.

Report privately via [GitHub Security Advisories](https://docs.github.com/en/code-security/security-advisories/guidance-on-reporting-and-writing-information-about-vulnerabilities/privately-reporting-a-security-vulnerability)
for this repository ("Report a vulnerability" under the **Security** tab). This
keeps the report confidential until a fix is available.

When reporting, please include:

- A description of the vulnerability and its impact
- Steps to reproduce (proof of concept if possible)
- Affected version(s) and platform
- Any suggested remediation

## Response

We aim to acknowledge reports within a few business days, provide an initial
assessment, and keep you updated as we work toward a fix. Once resolved, we will
coordinate a disclosure timeline with you and credit you if you wish.

## Scope

This policy covers the GwenLand ADE application (`ade-engine`, `ade-ui`) in this
repository. Vulnerabilities in third-party dependencies should be reported
upstream; if a dependency issue affects GwenLand ADE, let us know so we can bump
or mitigate.

## Local data

GwenLand ADE stores memory files under `~/.gwenland/ade/memory/`. These files may
contain project context; treat them as you would any local workspace data.
