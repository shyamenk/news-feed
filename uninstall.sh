#!/usr/bin/env bash

# News Feed Reader Uninstallation Script
# This script removes the News feed reader from /usr/local/bin

set -e

BINARY_NAME="news"
INSTALL_DIR="/usr/local/bin"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}News Feed Reader Uninstaller${NC}"
echo

if [ ! -f "${INSTALL_DIR}/${BINARY_NAME}" ]; then
    echo -e "${RED}Error: ${BINARY_NAME} is not installed in ${INSTALL_DIR}${NC}"
    exit 1
fi

# Check if we need sudo
if [ ! -w "$INSTALL_DIR" ]; then
    echo -e "${YELLOW}Removing from ${INSTALL_DIR} (requires sudo)...${NC}"
    sudo rm -f "${INSTALL_DIR}/${BINARY_NAME}"
else
    echo -e "${YELLOW}Removing from ${INSTALL_DIR}...${NC}"
    rm -f "${INSTALL_DIR}/${BINARY_NAME}"
fi

echo -e "${GREEN}Uninstallation complete!${NC}"
echo
echo "Note: Configuration and database files were NOT removed:"
echo "  Config: ~/.config/news/config.toml"
echo "  Database: ~/.local/share/news/news_feed.db"
echo
echo "To remove them manually, run:"
echo "  rm -rf ~/.config/news ~/.local/share/news"
