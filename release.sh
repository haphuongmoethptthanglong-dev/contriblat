#!/usr/bin/env bash
set -euo pipefail

# ── Config ────────────────────────────────────────────────────────────────────
REMOTE="my"
CARGO_TOML="crates/contribai-rs/Cargo.toml"
CHANGELOG="CHANGELOG.md"
BRANCH="main"

# ── Helpers ───────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
DIM='\033[2m'
BOLD='\033[1m'
RESET='\033[0m'

info()  { echo -e "  ${GREEN}✅${RESET} $*"; }
step()  { echo -e "\n  ${CYAN}▸${RESET} $*"; }
error() { echo -e "  ${RED}❌${RESET} $*" >&2; exit 1; }

# ── Parse bump type ──────────────────────────────────────────────────────────
BUMP="${1:-patch}"

case "$BUMP" in
    major|minor|patch) ;;
    *) echo "Usage: $0 [major|minor|patch]  (default: patch)"; exit 1 ;;
esac

# ── Read current version from Cargo.toml ─────────────────────────────────────
CURRENT=$(grep '^version' "$CARGO_TOML" | head -1 | sed 's/.*"\(.*\)"/\1/')
if [ -z "$CURRENT" ]; then
    error "Could not read version from $CARGO_TOML"
fi

IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT"

# ── Bump version ─────────────────────────────────────────────────────────────
case "$BUMP" in
    major) MAJOR=$((MAJOR + 1)); MINOR=0; PATCH=0 ;;
    minor) MINOR=$((MINOR + 1)); PATCH=0 ;;
    patch) PATCH=$((PATCH + 1)) ;;
esac

NEW_VERSION="${MAJOR}.${MINOR}.${PATCH}"
NEW_TAG="v${NEW_VERSION}"
TODAY=$(date +%Y-%m-%d)

echo -e "\n  ${BOLD}ContribAI Release${RESET}"
echo -e "  ${DIM}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
echo -e "  ${DIM}Current:${RESET} v${CURRENT}"
echo -e "  ${DIM}Bump:${RESET}    ${BUMP}"
echo -e "  ${DIM}New:${RESET}     ${NEW_TAG}"
echo -e "  ${DIM}Remote:${RESET}  ${REMOTE}"

# ── Guard: check for uncommitted changes ─────────────────────────────────────
if ! git diff --quiet || ! git diff --cached --quiet; then
    error "Working tree has uncommitted changes. Commit or stash first."
fi

# ── Guard: check tag doesn't exist ──────────────────────────────────────────
if git rev-parse "$NEW_TAG" >/dev/null 2>&1; then
    error "Tag $NEW_TAG already exists"
fi

# ── Step 1: Update Cargo.toml ───────────────────────────────────────────────
step "Updating ${CARGO_TOML} → ${NEW_VERSION}"
sed -i "0,/^version = \"${CURRENT}\"/s//version = \"${NEW_VERSION}\"/" "$CARGO_TOML"
info "Cargo.toml updated"

# ── Step 2: Update CHANGELOG.md ─────────────────────────────────────────────
step "Updating ${CHANGELOG}"
sed -i "s/^## \[Unreleased\]/## [Unreleased]\n\n## [${NEW_VERSION}] - ${TODAY}/" "$CHANGELOG"
info "Changelog updated"

# ── Step 3: Build release ───────────────────────────────────────────────────
step "Building release binary..."
cargo build --release 2>&1 | tail -3
info "Release build OK"

# ── Step 4: Run tests ───────────────────────────────────────────────────────
step "Running tests..."
TEST_OUTPUT=$(cargo test 2>&1 || true)
PASSED=$(echo "$TEST_OUTPUT" | grep "test result" | grep -oP '\d+ passed' || echo "? passed")
FAILED=$(echo "$TEST_OUTPUT" | grep "test result" | grep -oP '\d+ failed' || echo "0 failed")
echo -e "  ${DIM}Results:${RESET} ${PASSED}, ${FAILED}"

# ── Step 5: Commit ──────────────────────────────────────────────────────────
step "Committing..."
git add "$CARGO_TOML" "$CHANGELOG"
git commit -m "release: ${NEW_TAG}" --quiet
info "Committed"

# ── Step 6: Tag ─────────────────────────────────────────────────────────────
step "Tagging ${NEW_TAG}"
git tag -a "$NEW_TAG" -m "release: ${NEW_TAG}"
info "Tagged"

# ── Step 7: Push ────────────────────────────────────────────────────────────
step "Pushing to ${REMOTE}..."
git push "$REMOTE" "$BRANCH" --quiet
git push "$REMOTE" "$NEW_TAG" --quiet
info "Pushed ${NEW_TAG} to ${REMOTE}"

# ── Done ────────────────────────────────────────────────────────────────────
echo -e "\n  ${DIM}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
echo -e "  ${GREEN}🎉${RESET} Released ${BOLD}${NEW_TAG}${RESET} → ${CYAN}${REMOTE}${RESET}"
echo ""
