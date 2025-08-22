#!/bin/bash

# Setup script for git commit-msg hook using conventional-commits

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Setting up Conventional Commits validation hook...${NC}"

# Check if we're in a git repository
if [ ! -d ".git" ]; then
    echo -e "${RED}Error: Not in a git repository${NC}"
    exit 1
fi

# Check if commit-check is installed
if ! command -v commit-check &> /dev/null; then
    echo -e "${YELLOW}commit-check not found. Installing from crates.io...${NC}"
    cargo install conventional-commits
fi

# Create hooks directory if it doesn't exist
mkdir -p .git/hooks

# Create the commit-msg hook
cat > .git/hooks/commit-msg << 'EOF'
#!/bin/sh
# Conventional Commits validation hook
# This hook validates commit messages using the conventional-commits tool

# Read the commit message
commit_message=$(cat "$1")

# Validate the commit message
if ! echo "$commit_message" | commit-check; then
    echo ""
    echo "❌ Commit message does not follow Conventional Commits format!"
    echo ""
    echo "Examples of valid commit messages:"
    echo "  feat: add new user registration"
    echo "  fix: resolve login timeout issue"
    echo "  docs: update API documentation"
    echo "  feat(auth): implement OAuth2 flow"
    echo "  feat!: breaking change to user API"
    echo ""
    echo "For more examples, run: commit-check examples"
    echo "For commit types info, run: commit-check types"
    echo ""
    exit 1
fi

echo "✅ Commit message is valid!"
EOF

# Make the hook executable
chmod +x .git/hooks/commit-msg

echo -e "${GREEN}✅ Git commit-msg hook installed successfully!${NC}"
echo ""
echo "The hook will now validate all commit messages according to Conventional Commits."
echo "To test it, try making a commit with an invalid message format."
echo ""
echo "To remove the hook, delete: .git/hooks/commit-msg"
