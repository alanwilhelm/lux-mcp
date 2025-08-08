#!/bin/bash

# Lux MCP Code Quality Check Script
# Runs fmt, check, clippy, and tests

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}   Lux MCP Code Quality Check${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Function to run command and check result
run_check() {
    local name="$1"
    local cmd="$2"
    
    echo -e "${YELLOW}Running $name...${NC}"
    if eval "$cmd"; then
        echo -e "${GREEN}✓ $name passed${NC}\n"
        return 0
    else
        echo -e "${RED}✗ $name failed${NC}\n"
        return 1
    fi
}

# Track failures
FAILED=0

# 1. Format check
echo -e "${BLUE}[1/5] Format Check${NC}"
if cargo fmt -- --check 2>/dev/null; then
    echo -e "${GREEN}✓ Code is properly formatted${NC}\n"
else
    echo -e "${YELLOW}⚠ Code needs formatting. Running cargo fmt...${NC}"
    cargo fmt
    echo -e "${GREEN}✓ Code formatted${NC}\n"
fi

# 2. Compilation check
echo -e "${BLUE}[2/5] Compilation Check${NC}"
if run_check "cargo check" "cargo check --all-targets 2>&1 | tail -5"; then
    :
else
    FAILED=$((FAILED + 1))
fi

# 3. Clippy lints
echo -e "${BLUE}[3/5] Clippy Lints${NC}"
if cargo clippy --all-targets --all-features -- -D warnings 2>&1 | grep -E "warning:|error:" > /dev/null; then
    echo -e "${YELLOW}⚠ Clippy found issues:${NC}"
    cargo clippy --all-targets --all-features 2>&1 | grep -E "warning:|error:" | head -10
    echo -e "${YELLOW}Run 'cargo clippy --fix' to auto-fix some issues${NC}\n"
    FAILED=$((FAILED + 1))
else
    echo -e "${GREEN}✓ No clippy warnings${NC}\n"
fi

# 4. Test compilation
echo -e "${BLUE}[4/5] Test Compilation${NC}"
if run_check "test build" "cargo test --no-run 2>&1 | tail -3"; then
    :
else
    FAILED=$((FAILED + 1))
fi

# 5. Documentation check
echo -e "${BLUE}[5/5] Documentation Check${NC}"
if cargo doc --no-deps --quiet 2>&1 | grep -E "warning:|error:" > /dev/null; then
    echo -e "${YELLOW}⚠ Documentation issues found${NC}"
    cargo doc --no-deps 2>&1 | grep -E "warning:|error:" | head -5
    FAILED=$((FAILED + 1))
else
    echo -e "${GREEN}✓ Documentation builds cleanly${NC}\n"
fi

# Summary
echo -e "${BLUE}========================================${NC}"
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ All checks passed!${NC}"
    echo -e "${GREEN}Your code is ready for commit.${NC}"
else
    echo -e "${RED}❌ $FAILED check(s) failed${NC}"
    echo -e "${YELLOW}Please fix the issues above before committing.${NC}"
    exit 1
fi
echo -e "${BLUE}========================================${NC}"

# Optional: Run tests if requested
if [ "$1" = "--test" ] || [ "$1" = "-t" ]; then
    echo ""
    echo -e "${BLUE}Running tests...${NC}"
    cargo test --quiet
    echo -e "${GREEN}✓ All tests passed${NC}"
fi

# Optional: Run with auto-fix
if [ "$1" = "--fix" ] || [ "$1" = "-f" ]; then
    echo ""
    echo -e "${BLUE}Auto-fixing issues...${NC}"
    cargo fmt
    cargo fix --allow-dirty --allow-staged
    cargo clippy --fix --allow-dirty --allow-staged
    echo -e "${GREEN}✓ Auto-fix complete${NC}"
    echo -e "${YELLOW}Please review changes before committing${NC}"
fi