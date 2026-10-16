# Strata 󰉋

> **A Terminal-Based File Manager with Integrated Disk Analytics, Snapshots, and Inline Encryption**  
> *Developed for BCSE303P — Operating Systems Lab, Vellore Institute of Technology*  
> *Authors: Shaarav S (24BYB0118), Kevin T John (24BYB0163)*  
> *Guide: Dr. Porkodi S (Assistant Professor Senior Grade – 1)*

---

## 󰏘 Design & Aesthetic: Polished Linux Rice

Strata is styled after the modern Linux ricing aesthetic (inspired by Catppuccin Mocha, Tokyo Night, `yazi`, and `lualine`):

* **Curated Palette**: Deep dark base (`#1E1E2E`) with Catppuccin pastel accents (Mauve `#CBA6F7`, Sapphire `#74C7EC`, Teal `#94E2D5`, Peach `#FAB387`, and Maroon `#F38BA8`).
* **Nerd Font Extension Icons**: Every file extension (`.rs`, `.py`, `.c`, `.md`, `.json`, `.tar`, `.png`, etc.) maps to its authentic colorful Nerd Font icon.
* **Miller Column Live Preview Pane (`Tab`)**: Split view showing line-numbered file contents for code, directory summaries for folders, and security envelope details for encrypted `.strata` archives.
* **Animated Braille Scanner Spinner**: Smooth rotating spinner (`⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏`) during background disk scans.
* **Smooth Usage Bars**: Dynamic proportional sub-character unicode gradients (`[████████░░░░░░░░]`) that change color smoothly based on disk consumption (Teal $\to$ Yellow $\to$ Peach $\to$ Red).
* **Live Theme Switcher (`T` key)**: Cycle between 4 iconic Linux ricing palettes without restarting — ☕ Catppuccin Mocha, 🌃 Tokyo Night, 🌲 Gruvbox Dark, and ❄️ Nord. Every UI element re-renders instantly.
* **Powerline Statusline**: Clean status pills displaying operational modes (`NORMAL`, `SCANNING`, `QUICK-JUMP`, `SNAPSHOT-DIFF`, `VAULT-CRYPT`, `INODE-STAT`, `TOP-HOGS`, `LIVE-FILTER`), active directory path, storage gauge, and contextual hotkeys.

---

## 🚀 Key Features

1. **Integrated Disk Analytics (`ncdu` style)**:
   * Recursive, non-blocking background scanner using `walkdir` and lockless channels (`crossbeam-channel`).
   * Inode cycle prevention (`st_dev` / `st_ino`) and true physical block allocation calculation (`st_blocks * 512`).
   * In-memory flat Arena tree model for $O(1)$ node lookups and bottom-up size aggregation.

2. **Storage Breakdown by File Category (`b` key)**:
   * Categorizes the entire project storage into major asset classes:
     * 󰏗 **Build Artifacts & Cache** (`target/`, `.o`, `node_modules`, `.so`)
     *  **Source Code & Scripts** (`.rs`, `.py`, `.c`, `.js`, `.go`, `.sh`)
     *  **Archives & Vaults** (`.strata`, `.zip`, `.tar.gz`)
     *  **Data & Configurations** (`.json`, `.yaml`, `.toml`, `.db`)
     * 󰋩 **Media, Audio & Video** (`.png`, `.mp4`, `.svg`)
     *  **Documents & Notes** (`.pdf`, `.md`, `.txt`)
   * Displays proportional usage bars, percentage distribution, and total file counts per category.

3. **POSIX Inode & Metadata Inspector (`i` key)**:
   * Real-time inspection of OS filesystem properties: Inode number, Device ID, symbolic & octal permissions (`rwxr-xr-x` / `0755`), hard links, UID/GID, optimum block size, physical vs logical size ratio (sparse files), and access/modify/change timestamps (`atime`, `mtime`, `ctime`).

4. **Top 20 Space Hogs Dashboard (`t` key)**:
   * Scans the entire project tree and surfaces the 20 largest files regardless of nesting depth, with instant jump navigation (`Enter`).

5. **In-Place Live Search / Filter (`/` key)**:
   * Filter current directory entries instantly by name or extension as you type.

6. **Frecency-Based Directory Navigation (`zoxide` style)**:
   * Tracks directory visit count and calculates time-decay frecency scores.
   * Instant fuzzy-matched quick jump (`f` or `Ctrl+P`) with highlighted matching keystrokes.
   * Persistent store in `~/.local/share/strata/frecency.json`.

7. **Point-in-Time Snapshots, Diff Engine & Markdown Export (`p`, `d`, `Ctrl+S`)**:
   * Capture instant snapshots of any directory tree (`p`).
   * Bidirectional differential inspector (`d`) categorizing changes into `[+] ADD`, `[-] DEL`, `[▲] GROW`, and `[▼] SHRK` with size deltas.
   * Export comprehensive analysis directly to GitHub Flavored Markdown (`Ctrl+S`).

8. **Inline Cryptographic Vault (AES-256-GCM + Argon2id)**:
   * In-place file and whole-directory encryption (`e`) and decryption (`u`).
   * Password confirmation check and entropy strength meter.
   * Directory packaging via in-memory TAR streams.
   * Atomic file writes (`.strata.tmp` $\to$ `.strata`) and memory scrubbing (`zeroize`).

9. **Space Reclamation (`x` key)**:
   * Delete large bloat directories with a confirmation dialogue calculating exact bytes to be freed.

---

## ⌨️ Complete Keybindings

| Keybinding | Action |
| :--- | :--- |
| `j` / `↓` | Move cursor down |
| `k` / `↑` | Move cursor up |
| `Enter` / `l` / `→` | Enter directory / drill down |
| `Backspace` / `h` / `←` | Ascend to parent directory |
| `Tab` | Toggle Live File Preview Pane (Yazi-style split) |
| `b` | Open Storage Breakdown by File Category |
| `i` | Open POSIX Inode & Filesystem Metadata Inspector |
| `t` | Open Top 20 Space Hogs Dashboard |
| `/` | In-place live file filtering |
| `s` | Cycle sort order (Size ▼, Size ▲, Name ▲, Name ▼, Items ▼, Time ▼) |
| `f` / `Ctrl+P` | Open Frecency Quick-Jump modal (fuzzy search) |
| `p` | Capture point-in-time snapshot of current directory |
| `d` | Open Snapshot Diff Inspector |
| `Ctrl+S` (in Diff) | Export differential report to `strata_diff_report.md` |
| `e` | Encrypt selected file or folder (with password confirmation) |
| `u` | Decrypt selected `.strata` encrypted archive |
| `x` / `Delete` | Space reclamation: confirm and delete selected item |
| `r` | Trigger background rescan of current directory |
| `T` | Cycle theme: Catppuccin Mocha → Tokyo Night → Gruvbox Dark → Nord |
| `q` / `Esc` | Quit Strata |

---

## 🛠️ Build & Run

```bash
# Optimized release binary
cargo build --release

# Run in current directory
./target/release/strata

# Or inspect any target directory
./target/release/strata ~

# Headless benchmark scan
./target/release/strata --bench /usr

# Run complete automated systems benchmark suite
./benches/benchmark.sh ~
```
