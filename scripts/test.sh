#!/bin/bash

# Comprehensive test script for conventional-commits

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🧪 Running comprehensive tests for conventional-commits${NC}"
echo

# Step 1: Build the project first
echo -e "${YELLOW}📦 Building project...${NC}"
cargo build --release

echo -e "${YELLOW}📦 Building debug version...${NC}"
cargo build

# Step 2: Run library tests
echo -e "${YELLOW}🧪 Running library tests...${NC}"
cargo test --lib --verbose

# Step 3: Run integration tests (but build binaries first)
echo -e "${YELLOW}🧪 Running integration tests...${NC}"
cargo test --test '' --verbose 2>/dev/null || true
cargo test --verbose

# Step 4: Test CLI manually
echo -e "${YELLOW}🚀 Testing CLI functionality manually...${NC}"

echo -e "${BLUE}Testing valid commit...${NC}"
if cargo run --bin commit-check -- --message "feat: add new feature" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Valid commit test passed${NC}"
else
    echo -e "${RED}❌ Valid commit test failed${NC}"
    exit 1
fi

echo -e "${BLUE}Testing invalid commit...${NC}"
if cargo run --bin commit-check -- --message "invalid commit" > /dev/null 2>&1; then
    echo -e "${RED}❌ Invalid commit test failed (should have failed)${NC}"
    exit 1
else
    echo -e "${GREEN}✅ Invalid commit test passed (correctly rejected)${NC}"
fi

echo -e "${BLUE}Testing examples command...${NC}"
if cargo run --bin commit-check -- examples > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Examples command test passed${NC}"
else
    echo -e "${RED}❌ Examples command test failed${NC}"
    exit 1
fi

echo -e "${BLUE}Testing types command...${NC}"
if cargo run --bin commit-check -- types > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Types command test passed${NC}"
else
    echo -e "${RED}❌ Types command test failed${NC}"
    exit 1
fi

echo -e "${BLUE}Testing verbose output...${NC}"
if cargo run --bin commit-check -- --message "feat(auth): add OAuth" --verbose | grep -q "Type: feat"; then
    echo -e "${GREEN}✅ Verbose output test passed${NC}"
else
    echo -e "${RED}❌ Verbose output test failed${NC}"
    exit 1
fi

echo -e "${BLUE}Testing stdin input...${NC}"
if echo "fix: resolve bug" | cargo run --bin commit-check > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Stdin input test passed${NC}"
else
    echo -e "${RED}❌ Stdin input test failed${NC}"
    exit 1
fi

# Step 5: Run clippy and fmt checks
echo -e "${YELLOW}📋 Running clippy...${NC}"
cargo clippy -- -D warnings

echo -e "${YELLOW}🎨 Checking formatting...${NC}"
cargo fmt -- --check

echo
echo -e "${GREEN}🎉 All tests passed successfully!${NC}"
echo -e "${BLUE}The conventional-commits tool is ready to use.${NC}"
