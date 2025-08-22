#!/bin/bash

# Test script for Nix package functionality

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🧪 Testing Nix package for conventionnal-commits${NC}"
echo

# Step 1: Test Nix build
echo -e "${YELLOW}📦 Testing Nix build...${NC}"
if nix-build default.nix; then
    echo -e "${GREEN}✅ Nix build successful${NC}"
else
    echo -e "${RED}❌ Nix build failed${NC}"
    exit 1
fi

echo -e "${YELLOW}📦 Testing Nix build with Flakes...${NC}"
if nix build; then
    echo -e "${GREEN}✅ Nix Flakes build successful${NC}"
else
    echo -e "${RED}❌ Nix Flakes build failed${NC}"
    echo -e "${YELLOW}Note: Flakes might not be enabled on this system${NC}"
fi

# Step 2: Test installation
echo -e "${YELLOW}📥 Testing installation...${NC}"
if nix-env -f default.nix -i; then
    echo -e "${GREEN}✅ Installation successful${NC}"
else
    echo -e "${RED}❌ Installation failed${NC}"
    exit 1
fi

# Step 3: Test binary functionality
echo -e "${YELLOW}🚀 Testing binary functionality...${NC}"

echo -e "${BLUE}Testing --version flag...${NC}"
if commit-check --version; then
    echo -e "${GREEN}✅ Version test passed${NC}"
else
    echo -e "${RED}❌ Version test failed${NC}"
    exit 1
fi

echo -e "${BLUE}Testing valid commit message...${NC}"
if commit-check --message "feat: add new feature"; then
    echo -e "${GREEN}✅ Valid commit test passed${NC}"
else
    echo -e "${RED}❌ Valid commit test failed${NC}"
    exit 1
fi

echo -e "${BLUE}Testing invalid commit message...${NC}"
if commit-check --message "invalid commit"; then
    echo -e "${RED}❌ Invalid commit test failed (should have failed)${NC}"
    exit 1
else
    echo -e "${GREEN}✅ Invalid commit test passed (correctly rejected)${NC}"
fi

echo -e "${BLUE}Testing examples command...${NC}"
if commit-check examples > /dev/null; then
    echo -e "${GREEN}✅ Examples command test passed${NC}"
else
    echo -e "${RED}❌ Examples command test failed${NC}"
    exit 1
fi

echo -e "${BLUE}Testing types command...${NC}"
if commit-check types > /dev/null; then
    echo -e "${GREEN}✅ Types command test passed${NC}"
else
    echo -e "${RED}❌ Types command test failed${NC}"
    exit 1
fi

echo -e "${BLUE}Testing verbose output...${NC}"
if commit-check --message "feat(auth): add OAuth" --verbose | grep -q "Type: feat"; then
    echo -e "${GREEN}✅ Verbose output test passed${NC}"
else
    echo -e "${RED}❌ Verbose output test failed${NC}"
    exit 1
fi

# Step 4: Test with nix-shell
echo -e "${YELLOW}🐚 Testing with nix-shell...${NC}"

echo -e "${BLUE}Testing nix-shell execution...${NC}"
if nix-shell -p conventionnal-commits --run "commit-check --message 'test: nix-shell test'"; then
    echo -e "${GREEN}✅ nix-shell test passed${NC}"
else
    echo -e "${RED}❌ nix-shell test failed${NC}"
    exit 1
fi

# Step 5: Test Flakes app (if Flakes are available)
echo -e "${YELLOW}🎯 Testing Nix Flakes app...${NC}"
if command -v nix >/dev/null 2>&1 && nix --version | grep -q "flakes"; then
    if nix run . -- --message "feat: test flakes app"; then
        echo -e "${GREEN}✅ Nix Flakes app test passed${NC}"
    else
        echo -e "${RED}❌ Nix Flakes app test failed${NC}"
        exit 1
    fi
else
    echo -e "${YELLOW}⚠️ Nix Flakes not available, skipping app test${NC}"
fi

# Step 6: Clean up
echo -e "${YELLOW}🧹 Cleaning up...${NC}"
nix-env -e conventionnal-commits || true

# Step 7: Performance test (basic)
echo -e "${YELLOW}⚡ Testing performance...${NC}"
echo -e "${BLUE}Testing multiple commits processing...${NC}"

start_time=$(date +%s.%N)
for i in {1..100}; do
    nix-shell -p conventionnal-commits --run "commit-check --message 'feat: test commit $i'" > /dev/null
done
end_time=$(date +%s.%N)

runtime=$(echo "$end_time - $start_time" | bc)
echo -e "${GREEN}✅ Processed 100 commits in ${runtime} seconds${NC}"

echo
echo -e "${GREEN}🎉 All Nix package tests passed successfully!${NC}"
echo -e "${BLUE}The conventionnal-commits Nix package is ready for nixpkgs submission.${NC}"

echo
echo -e "${YELLOW}Next steps:${NC}"
echo "1. Update the hashes in default.nix with the correct values"
echo "2. Create a release tag on GitHub (v0.1.0)"  
echo "3. Update the GitHub URL in default.nix"
echo "4. Follow the nixpkgs submission guide"
echo "5. Test on different platforms if possible"
