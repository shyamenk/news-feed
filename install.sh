#!/usr/bin/env bash

# Complete Cleanup and Fresh Installation Script
# This removes all old instances and does a clean install

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}======================================${NC}"
echo -e "${BLUE}News Feed - Clean Installation${NC}"
echo -e "${BLUE}======================================${NC}"
echo

# Step 1: Remove all old binaries
echo -e "${YELLOW}Step 1: Cleaning up old installations...${NC}"

OLD_LOCATIONS=(
    "/usr/local/bin/news"
    "/usr/bin/news"
    "/bin/news"
    "$HOME/.local/bin/news"
    "$HOME/bin/news"
)

for location in "${OLD_LOCATIONS[@]}"; do
    if [ -f "$location" ]; then
        echo -e "  ${RED}Found old binary at: $location${NC}"
        if [ -w "$(dirname "$location")" ]; then
            rm -f "$location"
            echo -e "  ${GREEN}Removed: $location${NC}"
        else
            sudo rm -f "$location"
            echo -e "  ${GREEN}Removed: $location (with sudo)${NC}"
        fi
    fi
done

echo -e "${GREEN}Cleanup complete!${NC}"
echo

# Step 2: Clean build
echo -e "${YELLOW}Step 2: Clean build...${NC}"
cargo clean
echo -e "${GREEN}Build cache cleaned!${NC}"
echo

# Step 3: Build fresh binary
echo -e "${YELLOW}Step 3: Building optimized release binary...${NC}"
cargo build --release

if [ ! -f "target/release/news" ]; then
    echo -e "${RED}Error: Build failed!${NC}"
    exit 1
fi

echo -e "${GREEN}Build successful!${NC}"
echo

# Step 4: Install to /usr/local/bin
echo -e "${YELLOW}Step 4: Installing to /usr/local/bin...${NC}"

if [ ! -w "/usr/local/bin" ]; then
    sudo cp "target/release/news" "/usr/local/bin/news"
    sudo chmod +x "/usr/local/bin/news"
else
    cp "target/release/news" "/usr/local/bin/news"
    chmod +x "/usr/local/bin/news"
fi

echo -e "${GREEN}Installation complete!${NC}"
echo

# Step 5: Verify installation
echo -e "${YELLOW}Step 5: Verifying installation...${NC}"
echo

if command -v news &> /dev/null; then
    echo -e "${GREEN}✓ 'news' command is accessible${NC}"
    echo -e "  Location: $(which news)"
    echo

    # Test version
    echo -e "${YELLOW}Testing version command...${NC}"
    news --version 2>&1 || echo -e "${RED}Note: Version check failed (may need proper terminal)${NC}"
    echo

    # Test info command
    echo -e "${YELLOW}Testing info command...${NC}"
    news info 2>&1 || echo -e "${RED}Note: Info check failed (may need proper terminal)${NC}"
    echo
else
    echo -e "${RED}✗ 'news' command not found in PATH${NC}"
    echo -e "${YELLOW}Please add /usr/local/bin to your PATH:${NC}"
    echo -e "  echo 'export PATH=\"/usr/local/bin:\$PATH\"' >> ~/.bashrc"
    echo -e "  source ~/.bashrc"
    exit 1
fi

# Final summary
echo -e "${GREEN}======================================${NC}"
echo -e "${GREEN}Installation Complete!${NC}"
echo -e "${GREEN}======================================${NC}"
echo
echo -e "${BLUE}Configuration:${NC}"
echo -e "  Config:   ~/.config/news/config.toml"
echo -e "  Database: ~/.local/share/news/news_feed.db"
echo
echo -e "${BLUE}Quick Start:${NC}"
echo -e "  ${GREEN}news${NC}              - Start the RSS reader"
echo -e "  ${GREEN}news --help${NC}       - Show all options"
echo -e "  ${GREEN}news info${NC}         - Show configuration"
echo -e "  ${GREEN}news list-feeds${NC}   - List your feeds"
echo
echo -e "${YELLOW}Try running: ${GREEN}news${NC}"
echo
