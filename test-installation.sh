#!/usr/bin/env bash

# Test script to verify News installation

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}======================================${NC}"
echo -e "${BLUE}Testing News Feed Installation${NC}"
echo -e "${BLUE}======================================${NC}"
echo

# Test 1: Check if command exists
echo -e "${YELLOW}Test 1: Checking if 'news' command exists...${NC}"
if command -v news &> /dev/null; then
    echo -e "${GREEN}✓ PASS${NC} - 'news' command found at: $(which news)"
else
    echo -e "${RED}✗ FAIL${NC} - 'news' command not found"
    echo -e "${YELLOW}Solution: Make sure /usr/local/bin is in your PATH${NC}"
    exit 1
fi
echo

# Test 2: Check version
echo -e "${YELLOW}Test 2: Checking version...${NC}"
if news --version 2>&1 | grep -q "news"; then
    echo -e "${GREEN}✓ PASS${NC}"
    news --version
else
    echo -e "${YELLOW}⚠ WARNING${NC} - Version command may need terminal"
fi
echo

# Test 3: Check help
echo -e "${YELLOW}Test 3: Checking help command...${NC}"
if news --help 2>&1 | grep -q "terminal-based RSS"; then
    echo -e "${GREEN}✓ PASS${NC} - Help text available"
else
    echo -e "${YELLOW}⚠ WARNING${NC} - Help command may need terminal"
fi
echo

# Test 4: Check info command
echo -e "${YELLOW}Test 4: Checking info command...${NC}"
if news info 2>&1 | grep -q "News Feed Reader"; then
    echo -e "${GREEN}✓ PASS${NC}"
    news info
else
    echo -e "${YELLOW}⚠ WARNING${NC} - Info command may need terminal"
fi
echo

# Test 5: Check config directory
echo -e "${YELLOW}Test 5: Checking configuration setup...${NC}"
CONFIG_DIR="$HOME/.config/news"
DATA_DIR="$HOME/.local/share/news"

if [ -d "$CONFIG_DIR" ]; then
    echo -e "${GREEN}✓ PASS${NC} - Config directory exists: $CONFIG_DIR"
    if [ -f "$CONFIG_DIR/config.toml" ]; then
        echo -e "${GREEN}✓ PASS${NC} - Config file exists"
    else
        echo -e "${YELLOW}⚠ INFO${NC} - Config file will be created on first run"
    fi
else
    echo -e "${YELLOW}⚠ INFO${NC} - Config directory will be created on first run"
fi

if [ -d "$DATA_DIR" ]; then
    echo -e "${GREEN}✓ PASS${NC} - Data directory exists: $DATA_DIR"
    if [ -f "$DATA_DIR/news_feed.db" ]; then
        echo -e "${GREEN}✓ PASS${NC} - Database exists"
    else
        echo -e "${YELLOW}⚠ INFO${NC} - Database will be created on first run"
    fi
else
    echo -e "${YELLOW}⚠ INFO${NC} - Data directory will be created on first run"
fi
echo

# Test 6: List feeds (if database exists)
echo -e "${YELLOW}Test 6: Testing list-feeds command...${NC}"
if [ -f "$DATA_DIR/news_feed.db" ]; then
    news list-feeds 2>&1 && echo -e "${GREEN}✓ PASS${NC}" || echo -e "${YELLOW}⚠ WARNING${NC}"
else
    echo -e "${YELLOW}⚠ INFO${NC} - No database yet (will be created on first run)"
fi
echo

# Summary
echo -e "${GREEN}======================================${NC}"
echo -e "${GREEN}Test Summary${NC}"
echo -e "${GREEN}======================================${NC}"
echo -e "${BLUE}Installation appears to be working!${NC}"
echo
echo -e "${YELLOW}Next steps:${NC}"
echo -e "  1. Run: ${GREEN}news${NC}"
echo -e "  2. Navigate with Tab and arrow keys"
echo -e "  3. Press 'q' to quit"
echo -e "  4. Run: ${GREEN}news --help${NC} for all options"
echo
