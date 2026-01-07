# Contributing to Shimmy

Thank you for your interest in Shimmy! í¶€

## Open Source, Not Open Contribution

Shimmy is **open source** but **not open contribution**.

- The code is freely available under the MIT license
- You can fork, modify, use, and learn from it without restriction
- **Pull requests are not accepted by default**
- All architectural, roadmap, and merge decisions are made by the project maintainer

This model keeps the project coherent, maintains clear ownership, and ensures consistent quality. It's the same approach used by SQLite and many infrastructure projects.

## How to Contribute

If you believe you can contribute meaningfully to Shimmy:

1. **Email the maintainer first**: [michaelallenkuykendall@gmail.com](mailto:michaelallenkuykendall@gmail.com)
2. Describe your background and proposed contribution
3. If there is alignment, a scoped collaboration may be discussed privately
4. Only after discussion will PRs be considered

**Unsolicited PRs will be closed without merge.** This isn't personal â€” it's how this project operates.

## What We Welcome (via email first)

- Bug reports with detailed reproduction steps (Issues are fine)
- Security vulnerability reports (please email directly)
- Documentation improvements (discuss first)
- Platform-specific bug fixes (discuss first)

## What We Handle Internally

- New features and architectural changes
- API design decisions
- Dependency updates
- Performance optimizations
- Cross-platform compatibility work

## Bug Reports

Bug reports via GitHub Issues are welcome! Please include:
- Platform (Windows/macOS/Linux) and version
- Rust version and Shimmy version
- Minimal reproduction case
- Expected vs actual behavior
- Backend details if relevant (llama.cpp version, model info)

## Code Style (for reference)

If a contribution is discussed and approved:
- Rust 2021 edition with `cargo fmt` and `cargo clippy -- -D warnings`
- Comprehensive error handling using proper Result types
- All public APIs must have documentation with examples
- Tests for new functionality

## Shimmy Philosophy

Any accepted work must align with:
- **Lightweight**: ~5MB binary target
- **Zero-config**: No setup, just works
- **OpenAI API compatibility**: Drop-in replacement
- **Invisible infrastructure**: Minimal surface area
- **Free forever**: Core features remain free, premium features licensed separately

## Why This Model?

Building reliable LLM infrastructure requires tight architectural control. This ensures:
- Consistent API design and backwards compatibility
- No ownership disputes or governance overhead
- Quality control without committee delays
- Clear direction for the project's future

The code is open. The governance is centralized. This is intentional.

## Recognition

Helpful bug reports and community members are acknowledged in release notes.
If email collaboration leads to merged work, attribution will be given appropriately.

---

**Maintainer**: Michael A. Kuykendall
