# Installation Guide for News Feed Reader

## Quick Start (Clean Installation)

### Step 1: Clean Install Everything
Run this script to remove any old installations and do a fresh install:

```bash
cd /home/shyamenk/Desktop/Experiment/news-feed
./cleanup-and-install.sh
```

This will:
- Remove any old `news` binaries from all common locations
- Clean the build cache
- Build a fresh optimized release binary
- Install to `/usr/local/bin/news`
- Verify the installation

### Step 2: Test the Installation
```bash
./test-installation.sh
```

This will verify that everything is working correctly.

### Step 3: Start Using It!
```bash
news
```

---

## If You Get "command not found"

If after installation you get `news: command not found`, it means `/usr/local/bin` is not in your PATH.

### Fix 1: Add to PATH (Permanent)
Add this to your shell config file (`~/.bashrc` or `~/.zshrc`):

```bash
echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

For Zsh users:
```bash
echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

### Fix 2: Use Full Path (Temporary)
```bash
/usr/local/bin/news
```

### Fix 3: Install to a Different Location
Install to `~/.local/bin` which is usually in PATH:

```bash
cp target/release/news-feed ~/.local/bin/news
chmod +x ~/.local/bin/news
```

---

## If You Get Permission Errors

If you get permission denied errors:

```bash
sudo chmod +x /usr/local/bin/news
```

Or reinstall:
```bash
sudo rm /usr/local/bin/news
sudo cp target/release/news-feed /usr/local/bin/news
sudo chmod +x /usr/local/bin/news
```

---

## Verify Installation

After installation, verify everything works:

```bash
# Check if news is in your PATH
which news

# Test the version
news --version

# Check configuration info
news info

# Show help
news --help
```

---

## First Run

When you run `news` for the first time:

```bash
news
```

It will automatically:
1. Create `~/.config/news/config.toml`
2. Create `~/.local/share/news/news_feed.db`
3. Add some default RSS feeds
4. Start the TUI application

**Controls:**
- `Tab` - Switch between tabs
- `j/k` or `↑/↓` - Navigate
- `Enter` - Select/Open
- `q` - Quit

---

## Troubleshooting

### Issue: Binary won't run
**Solution:** Make sure it's executable
```bash
chmod +x /usr/local/bin/news
```

### Issue: "command not found" even after installation
**Solution:** Reload your shell or check PATH
```bash
# Reload shell
source ~/.bashrc  # or source ~/.zshrc

# Check if /usr/local/bin is in PATH
echo $PATH | grep /usr/local/bin
```

### Issue: Permission denied
**Solution:** Install with sudo
```bash
sudo cp target/release/news-feed /usr/local/bin/news
sudo chmod +x /usr/local/bin/news
```

### Issue: Want to reinstall fresh
**Solution:** Use the cleanup script
```bash
./cleanup-and-install.sh
```

---

## Manual Installation Steps

If the scripts don't work, here's the manual process:

### 1. Remove old installations
```bash
sudo rm -f /usr/local/bin/news /usr/bin/news /bin/news
rm -f ~/.local/bin/news ~/bin/news
```

### 2. Clean and build
```bash
cd /home/shyamenk/Desktop/Experiment/news-feed
cargo clean
cargo build --release
```

### 3. Install
```bash
sudo cp target/release/news /usr/local/bin/news
sudo chmod +x /usr/local/bin/news
```

### 4. Test
```bash
news --version
news info
news
```

---

## Uninstallation

To remove News Feed Reader:

### Option 1: Use the uninstall script
```bash
./uninstall.sh
```

### Option 2: Manual removal
```bash
# Remove binary
sudo rm /usr/local/bin/news

# (Optional) Remove config and data
rm -rf ~/.config/news
rm -rf ~/.local/share/news
```

---

## Next Steps

After successful installation:

1. **Add your feeds**: Edit `~/.config/news/config.toml`
2. **Explore**: Run `news` and press Tab to see different tabs
3. **Get help**: Run `news --help` for all commands
4. **Read docs**: Check `README.md` for full documentation

Enjoy your terminal-based RSS reader! 🚀
