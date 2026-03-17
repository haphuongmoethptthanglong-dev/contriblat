# Changelog

All notable changes to ContribAI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-03-18

### Added
- **Issue Solver**: Classify GitHub issues by labels/keywords, filter by solvability, LLM-powered solving
- **Framework Strategies**: Auto-detect Django, Flask, FastAPI, React/Next.js, Express with tailored analysis
- **Quality Scorer**: 7-check quality gate before PR submission (change size, commit format, debug code, placeholders)
- **CLI**: New `solve` command for issue-driven contributions
- Tests: 169 total tests covering all modules

## [0.2.0] - 2026-03-18

### Added
- **Retry Utilities**: `async_retry` decorator with exponential backoff + jitter
- **LRU Cache**: Response caching for GitHub API and LLM calls
- **Test Suite**: 128 tests across all modules (config, models, memory, LLM, GitHub, discovery, analyzer, generator, PR manager, CLI)
- Integration tests for pipeline dry run and analyze-only mode

## [0.1.0] - 2026-03-17

### Added
- **Core Pipeline**: Full discover → analyze → generate → PR workflow
- **Multi-LLM Support**: Gemini (primary), OpenAI, Anthropic, Ollama providers
- **GitHub Integration**: Async API client with rate limiting, repo discovery
- **Code Analysis**: Security, code quality, documentation, and UI/UX analyzers
- **Contribution Generator**: LLM-powered code generation with self-review
- **PR Manager**: Automated fork → branch → commit → PR workflow
- **Memory System**: SQLite-backed persistent tracking of repos and PRs
- **Rich CLI**: Commands: `run`, `target`, `analyze`, `status`, `stats`, `config`
- **Team Infrastructure**: 8 agent roles, 10 workflows, CI/CD pipelines
- **GitHub Templates**: PR template, bug/feature/security issue templates
- **DevOps**: Dockerfile, Makefile, GitHub Actions CI/CD
- **Documentation**: README, CONTRIBUTING, CHANGELOG, SECURITY
