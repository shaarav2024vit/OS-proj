# Strata: BCSE303P Project Review - 1 Presentation & Viva Guide

**Course**: BCSE303P — Operating Systems Lab  
**Institution**: School of Computer Science Engineering and Information Systems, Vellore Institute of Technology  
**Project Title**: Strata: A Terminal-Based File Manager with Integrated Disk Analytics, Snapshots, and Encryption  
**Authors**: Shaarav S (24BYB0118), Kevin T John (24BYB0163)  
**Faculty Guide**: Dr. Porkodi S (Assistant Professor Senior Grade – 1)

---

## 📑 Quick Navigation
1. [Slide-by-Slide Presentation Deck](#part-1-slide-by-slide-presentation-deck)
2. [3-Minute Live Demo Script](#part-2-3-minute-live-demonstration-script)
3. [Top 12 Tough Viva Questions & Model Answers](#part-3-top-12-tough-viva-questions--model-answers)

---

## PART 1: Slide-by-Slide Presentation Deck

### Slide 1: Title & Administrative Details
* **Header**: Strata: A Terminal-Based File Manager with Integrated Disk Analytics, Snapshots, and Encryption
* **Subtitle**: BCSE303P — Operating Systems Lab Project Review - 1
* **Details**:
  * Shaarav S — `24BYB0118`
  * Kevin T John — `24BYB0163`
  * Under the Guidance of: **Dr. Porkodi S**, Assistant Professor Senior Grade – 1
  * School of Computer Science Engineering and Information Systems (SCOPE), VIT
* **Speaker Notes**:
  > *"Respected guide Dr. Porkodi mam and evaluators, good morning. Today, Kevin and I are presenting Review-1 for our Operating Systems Lab project, titled 'Strata: A Terminal-Based File Manager with Integrated Disk Analytics, Snapshots, and Encryption'."*

---

### Slide 2: Problem Statement & Motivation
* **Title**: Motivation: The Fragmented Command-Line Storage Workflow
* **Key Points**:
  * **Tool Siloing**: Developers juggle disconnected single-purpose utilities:
    * Disk analysis $\to$ `ncdu`, `gdu`
    * Directory traversal $\to$ `zoxide`, `cd`, `fzf`
    * Change audit $\to$ `diff -r`, custom shell scripts
    * File security $\to$ `gpg`, `age`, `openssl`
  * **Performance Bottleneck**: Every utility performs duplicate, uncoordinated filesystem traversal passes across identical directory trees.
  * **Context Switching Friction**: No shared in-memory state; paths must be manually copied, re-typed, or piped across utilities.
  * **Black-Box Storage**: Standard file managers show item names but fail to communicate true physical block allocation or historical disk growth.
* **Speaker Notes**:
  > *"Developers waste significant time switching between isolated utilities with differing syntax. Every time you run ncdu to find a large directory and then switch to gpg to encrypt it, the OS performs duplicate inode traversals. Strata eliminates this friction by unifying storage analysis, navigation, historical diffs, and security around a single shared directory tree."*

---

### Slide 3: Project Objectives & Scope
* **Title**: Objectives & Boundary Scope for Review-1
* **Objectives**:
  1. **Unified Memory Model**: Design an Arena-based directory tree loaded once by an asynchronous background scanner.
  2. **Non-Blocking Disk Analytics**: Deliver responsive 60 FPS terminal interaction while scanning tens of gigabytes.
  3. **Predictive Frecency Navigation**: Implement a time-decay access scoring engine (`zoxide` model) with fuzzy matching.
  4. **Point-in-Time Snapshot Diff Engine**: Track storage drift by reporting `ADDED`, `REMOVED`, `GROWN`, and `SHRUNK` paths with size deltas.
  5. **Inline Cryptographic Vault**: Implement zero-dependency file and directory encryption using authenticated AES-256-GCM and Argon2id.
* **Scope**:
  * Single-user, local-disk environment; keyboard-first terminal workflow.
  * Excludes cloud sync, distributed filesystems, or GUI wrappers to focus strictly on OS-level systems programming.
* **Speaker Notes**:
  > *"Our primary objective is to prove that one single high-speed scan can power disk visualization, intelligent navigation, historical change tracking, and authenticated encryption without repeated disk I/O."*

---

### Slide 4: System Architecture & Layered Design
* **Title**: Layered Modular Architecture
* **Visual Diagram**:
  ```
                        ┌─────────────────────────────────────────────────┐
                        │      TUI View Layer (Ratatui / Crossterm)       │
                        │  Explorer | Usage Bars | Inode | Diff | Vault   │
                        └────────────────────────┬────────────────────────┘
                                                 │ Reads State
                                                 ▼
  ┌──────────────────────┐      ┌─────────────────────────────────────────┐
  │   Scanner Engine     ├─────►│        Shared In-Memory Tree            │
  │  (walkdir / mpsc)    │ Pubs │  (Flat Arena Model: Inodes, Block Sizes)│
  └──────────────────────┘      └────────┬────────────┬─────────────┬─────┘
                                         │            │             │
                                         ▼            ▼             ▼
                                   ┌───────────┐┌───────────┐ ┌───────────┐
                                   │ Snapshots ││  Nav DB   │ │  Crypto   │
                                   │  & Diff   ││ (Frecency)│ │ (AES-GCM) │
                                   └───────────┘└───────────┘ └───────────┘
  ```
* **Core Principles**:
  * Strict separation of concerns: The TUI owns zero data; it only renders state.
  * Inter-Process / Thread Communication: The scanner streams batched node entries over crossbeam lock-free channels.
* **Speaker Notes**:
  > *"This architectural diagram represents our design. Five decoupled engines sit beneath the terminal view layer. The background scanner streams updates into the shared Arena tree, which subsequently feeds our snapshot comparator, frecency jump store, and AES-GCM crypto engine."*

---

### Slide 5: Operating System Concepts Implemented
* **Title**: Operating System Principles in Practice (BCSE303P)
* **Table of OS Concepts**:

| OS Concept | How It Is Implemented in Strata |
| :--- | :--- |
| **Concurrency & Synchronization** | Multi-threaded worker thread communicating via lock-free `crossbeam-channel` message queues; atomic cancellation tokens (`AtomicBool`). |
| **Inode Metadata & Storage** | Physical allocated size (`st_blocks * 512`) vs apparent length (`st_size`); Inode/Device ID tracking (`st_dev`, `st_ino`) to detect symlink loops. |
| **Memory Locality & Heap Management** | Flat vector-backed index Arena tree (`Vec<FileNode>`) eliminating pointer-chasing and cache misses (sub-35MB footprint for 500k files). |
| **Atomic File I/O Transactions** | Staging writes to `.tmp` files, flushing to disk buffers via `fsync` (`sync_all()`), and atomic renaming (`renameat`) to guarantee data integrity. |
| **POSIX Signals & Terminal Raw Mode** | Handling `SIGWINCH` resize events and restoring `termios` raw mode through custom panic unwind hooks. |
| **Cryptographic Memory Sanitization** | Automatic zeroization of master passwords and symmetric keys in volatile RAM using `zeroize`. |

* **Speaker Notes**:
  > *"Strata is specifically designed to demonstrate the core tenets of our operating systems syllabus: asynchronous concurrency, low-level POSIX inode metadata inspection, cache-friendly heap memory management, and atomic disk transactions."*

---

### Slide 6: Key Modules & Algorithmic Highlights
* **Title**: Engineering Specifications & Algorithms
* **Module Highlights**:
  * **Tree Size Aggregation**: Topological bottom-up accumulation ensuring $O(N)$ size calculation from leaf nodes to root.
  * **Frecency Scoring Model**:
    $$\text{Score}(d) = \text{VisitCount} \times W(\Delta t)$$
    Where $W(\Delta t)$ decays geometrically: 4.0 (< 1 hr), 2.0 (< 1 day), 1.0 (< 1 week), 0.5 (older).
  * **$O(N + M)$ Dual-Snapshot Diff**: Linear hash-join algorithm identifying state transitions (`ADDED`, `REMOVED`, `GROWN`, `SHRUNK`).
  * **Authenticated Encryption**: AES-256-GCM authenticated cipher with 28-byte unique random salt and nonce, paired with Argon2id memory-hard key derivation.
* **Speaker Notes**:
  > *"For navigation, we implement a time-decay frecency model inspired by zoxide. For change detection, our diff engine matches trees in linear time O(N + M). For security, we use NIST-standard AES-256-GCM with Argon2id."*

---

### Slide 7: UI/UX & The Linux-Rice Philosophy
* **Title**: User Experience: Polished Linux Rice Terminal
* **Aesthetic Highlights**:
  * **Catppuccin Mocha Palette**: Eliminates harsh generic colors in favor of soothing, high-contrast dark slate (`#1E1E2E`) and pastel accents.
  * **Nerd Font Glyphs**: Over 20 file-extension mappings (Rust ``, Python ``, C ``, Git ``).
  * **Miller Column Split**: Live file preview pane (`Tab`) with code line numbers and directory breakdowns.
  * **Proportional Gradient Bars**: Dynamic sub-character unicode bars (`[████████░░░░░░░░]`) shifting from Teal $\to$ Yellow $\to$ Peach $\to$ Red.
  * **Powerline Statusline**: Mode badges (`NORMAL`, `SCANNING`, `QUICK-JUMP`, `INODE-STAT`) and storage gauges.
* **Speaker Notes**:
  > *"Rather than looking like an unpolished school script or generic CLI, Strata adheres to modern Linux ricing design standards: Catppuccin Mocha palette, live file preview panes, and sub-character gradient bars."*

---

### Slide 8: Live Demonstration & Review Milestones
* **Title**: Current Progress & Review-1 Deliverables
* **Completed Milestones**:
  * ✅ High-performance multithreaded directory scanner and shared Arena model.
  * ✅ Ratatui terminal interface with usage bars, sorting, and drill-down.
  * ✅ POSIX Inode & Filesystem Metadata Inspector (`i` key).
  * ✅ Top 20 Space Hogs Dashboard (`t` key) and live filtering (`/`).
  * ✅ Zoxide-style frecency store and fuzzy jump modal (`f`).
  * ✅ Point-in-time snapshot generator and differential inspector (`p`, `d`).
  * ✅ Storage Breakdown by File Category (`b` key).
  * ✅ Automated Systems Benchmark Suite (`benches/benchmark.sh`) measuring rusage memory & throughput.
  * ✅ Unit and integration test suite (100% pass rate).
* **Next Steps for Review-2**:
  * Multi-selection tagging (`Space` key) for bulk operations.
  * Shell "CD on exit" integration.
* **Speaker Notes**:
  > *"All Review-1 deliverables are fully operational, tested, and empirically benchmarked. Strata clocks over 260,000 inodes per second with lower peak memory footprint than GNU du. We are now excited to demonstrate the live application."*

---

## PART 2: 3-Minute Live Demonstration Script

Follow this exact sequence during your live presentation with Dr. Porkodi S:

### Setup Before You Start
Open your terminal in full screen:
```bash
cd "/home/osleepy/Downloads/VIT Downloads/osproj"
./target/release/strata ~
```

---

### Step 1: Launch & Non-Blocking Background Scanner (0:00 – 0:35)
* **What to do**:
  * Launch `./target/release/strata ~`
  * Point out the **animated Braille spinner** in the bottom statusline: `⠋ SCANNING`.
  * While the scan is running, immediately press `j` and `k` to move the cursor, and `s` to change sorting modes.
* **What to say**:
  > *"Notice that as soon as Strata launches on my home directory, the UI is instantly interactive. The animated Braille spinner indicates that filesystem traversal is happening asynchronously on a background thread. There is zero UI freeze or frame drop because we use lock-free crossbeam message queues between threads."*

---

### Step 2: Inode & POSIX Metadata Inspector (`i` key) (0:35 – 1:10)
* **What to do**:
  * Highlight any file or directory.
  * Press **`i`**.
  * Point out:
    1. **Inode Number** and **Device ID**.
    2. **POSIX Permissions** in symbolic (`-rwxr-xr-x`) and octal (`0755`).
    3. **Physical Disk Allocated** (`st_blocks * 512`) vs **Logical Size** (`st_size`).
    4. Access, Modification, and Change timestamps.
  * Press **`Esc`** to exit.
* **What to say**:
  > *"Here, pressing 'i' opens our POSIX Inode Inspector. This directly demonstrates OS filesystem internals. You can see the exact Inode number, hardlink count, and device ID. Most importantly, notice the comparison between logical file length and physical allocated blocks. If a file is sparse or has unallocated holes, Strata detects the discrepancy by querying st_blocks multiplied by 512, which mirrors how the OS kernel allocates disk blocks."*

---

### Step 3: Miller-Column Live Preview (`Tab`), Category Breakdown (`b`) & Space Hogs (`t`) (1:10 – 1:45)
* **What to do**:
  * Press **`Tab`** to toggle the right-hand preview pane.
  * Move up and down across files to show code preview with line numbers (`001 │ ...`).
  * Press **`b`** to open the **Storage Breakdown by File Category** modal. Show the percentage bars for Build Artifacts, Source Code, Archives, Media, etc. Hit **`Esc`**.
  * Press **`t`** to open the **Top 20 Space Hogs** dashboard.
  * Highlight an item in the top hogs list and press **`Enter`** to jump straight to its parent folder.
* **What to say**:
  > *"Pressing 'Tab' toggles our Miller-column preview pane, which parses text files on the fly without loading entire gigabyte files into memory. Pressing 'b' pulls up an interactive category breakdown showing what proportion of disk usage belongs to build artifacts, source code, or media. Furthermore, when analyzing bloat, pressing 't' pulls up the Top 20 Space Hogs across the entire scanned hierarchy. Hitting Enter navigates straight to that directory."*

---

### Step 4: Frecency Navigation (`f`) & In-Place Filter (`/`) (1:45 – 2:15)
* **What to do**:
  * Press **`/`** and type a few letters of a visible folder to demonstrate instant filtering. Hit **`Esc`**.
  * Press **`f`** to open the Frecency Quick-Jump modal.
  * Type a fuzzy query (e.g. `doc` or `src`), showing highlighted character matches in Mauve.
  * Hit **`Enter`** to jump.
* **What to say**:
  > *"Strata tracks user navigation patterns using a time-decay frecency algorithm inspired by zoxide. Pressing 'f' brings up our fuzzy jump menu. As I type, the fuzzy matcher scores recent and frequent directories, allowing instantaneous transitions across deep paths."*

---

### Step 5: Historical Snapshot & Differential Engine (`p`, `d`) (2:15 – 2:45)
* **What to do**:
  * Press **`p`** to capture a snapshot (notice status bar: `Snapshot saved: snap_...`).
  * Press **`d`** to open the **Snapshot Diff Inspector**.
  * Show the colored badges: `[+] ADD`, `[-] DEL`, `[▲] GROW`, `[▼] SHRK`.
  * Highlight `Ctrl+S` export capability to markdown.
* **What to say**:
  > *"Pressing 'p' serializes an in-memory snapshot of the live directory tree to disk. Pressing 'd' invokes our differential comparator. If files were added, deleted, or grew in size between snapshots, our O(N + M) diff engine categorizes them with exact size deltas. Pressing Ctrl+S exports this directly into a GitHub Markdown report for audit trails."*

---

### Step 6: Inline Cryptographic Vault (`e`, `u`) (2:45 – 3:15)
* **What to do**:
  * Highlight a sample test file or folder.
  * Press **`e`**.
  * Show the vault modal with the **passphrase entropy meter** (`Weak` $\to$ `Strong`).
  * Type a password, hit `Tab`, type confirmation, and press `Enter`.
  * Show the item immediately transform into a `.strata` file with a peach padlock icon `󰌆`.
  * Notice the right preview pane displays the authenticated AES-256-GCM container info!
  * Highlight the `.strata` file, press **`u`**, enter password, and watch it restore cleanly.
* **What to say**:
  > *"Finally, we eliminate external tools for security. Pressing 'e' invokes our inline vault. We use Argon2id to derive a 256-bit key from the user passphrase, generate a cryptographically random salt and 12-byte nonce, and encrypt using AES-256-GCM. The ciphertext is written atomically to a temporary file, flushed, and renamed. If it's a directory, it is packaged as an in-memory TAR archive before encryption. Pressing 'u' verifies the Poly1305 authentication tag and restores the original plaintext."*

---

## PART 3: Top 12 Tough Viva Questions & Model Answers

### Q1: Why did you write Strata in Rust instead of C or Python?
* **Model Answer**:
  > *"We chose Rust because it provides C-level bare-metal performance and fine-grained memory control without the risk of segmentation faults, buffer overflows, or data races. Its ownership model and compile-time concurrency guarantees allow us to run background scanning threads safely alongside the TUI event loop with zero lock contention. Furthermore, Rust compiles to a standalone, zero-dependency 1.9MB binary."*

### Q2: How does Strata calculate true disk usage vs apparent file size?
* **Model Answer**:
  > *"Standard utilities often report `st_size` (the logical byte count). However, in Unix filesystems (like ext4 or Btrfs), disk space is allocated in blocks. Strata queries `std::os::unix::fs::MetadataExt::blocks()`, which returns the number of 512-byte filesystem blocks physically assigned to the inode. This ensures that sparse files with holes or compressed filesystem clusters reflect their genuine physical footprint on disk, matching `ncdu`."*

### Q3: How do you prevent infinite loops when symlinks point back to parent directories?
* **Model Answer**:
  > *"Every file and directory in a POSIX system is uniquely identified by the tuple `(st_dev, st_ino)` — the device ID and the inode number. In `src/scanner/walker.rs`, the background scanner maintains a thread-safe `HashSet<(u64, u64)>` of visited inodes. Before descending into any path, Strata checks this set; if the inode has already been visited, traversal skips it, completely preventing cyclical symlink recursion."*

### Q4: Why use an Arena-based tree instead of a standard pointer tree (`Rc<RefCell<Node>>`)?
* **Model Answer**:
  > *"In Rust, building cyclic or bidirectional trees with `Rc<RefCell<Node>>` causes severe memory fragmentation, cache misses, and reference cycle leaks. In `src/model/tree.rs`, we designed an index-based flat Arena tree (`Vec<FileNode>`). Each node is identified by a simple `usize` NodeId. This guarantees that all nodes reside contiguously in virtual memory, yielding high CPU cache locality and reducing our RAM footprint to under 35 MB for 500,000 files."*

### Q5: How do you achieve non-blocking scanning while keeping the TUI responsive at 60 FPS?
* **Model Answer**:
  > *"The scanning engine runs on a dedicated background thread spawned with `std::thread::spawn`. It communicates with the main thread via an asynchronous, lock-free `crossbeam-channel`. The scanner batches entries into chunks of 512 files and emits throttled progress heartbeats every 50ms. The main TUI event loop polls this receiver with `try_recv()` at 60 FPS (16ms ticks), so the UI renders smoothly without waiting for disk I/O."*

### Q6: What is 'frecency' and how is it mathematically calculated in your navigation module?
* **Model Answer**:
  > *"Frecency is a hybrid metric combining frequency (how often a directory is visited) and recency (how recently it was accessed), popularized by Mozilla and zoxide. In `src/frecency/store.rs`, we calculate: $\text{Score} = \text{Count} \times W(\Delta t)$. The weight $W$ decays based on elapsed time: 4.0 if visited within the last hour, 2.0 within the last day, 1.0 within the week, and 0.5 otherwise. This ensures frequently visited paths stay near the top while obsolete directories naturally decay."*

### Q7: What encryption algorithm do you use, and why AES-GCM instead of AES-CBC?
* **Model Answer**:
  > *"We use AES-256-GCM (Galois/Counter Mode). AES-CBC only provides confidentiality and is vulnerable to padding oracle attacks unless combined with an external HMAC. AES-GCM is an Authenticated Encryption with Associated Data (AEAD) cipher. It computes a 16-byte Poly1305 authentication tag simultaneously with encryption. If an attacker modifies even a single bit of the encrypted file on disk, decryption fails immediately before any corrupted data is unpacked."*

### Q8: How do you derive the symmetric key from the user's password?
* **Model Answer**:
  > *"We use Argon2id (RFC 9106), the winner of the Password Hashing Competition. Simple hashes like MD5 or SHA-256 are vulnerable to GPU and ASIC brute-forcing. Argon2id is memory-hard, requiring 19 MB of memory and multiple iterations per derivation, which renders GPU parallel dictionary attacks infeasible. We also generate a cryptographically secure 16-byte random salt per encryption operation using `rand::thread_rng()`."*

### Q9: How do you ensure data safety during in-place encryption so power loss doesn't destroy the file?
* **Model Answer**:
  > *"We follow the ACID write-ahead principle. We write the ciphertext to a temporary file (`<name>.strata.tmp`). Once writing completes, we invoke `sync_all()` (the `fsync` syscall) to ensure all kernel write buffers are physically flushed to persistent storage. Only then do we execute an atomic rename (`std::fs::rename`) over the target path. If power is lost mid-encryption, the original plaintext file remains intact and untruncated."*

### Q10: How does your snapshot diff engine work in $O(N + M)$ time?
* **Model Answer**:
  > *"When comparing two snapshots containing $N$ and $M$ items, we construct hash maps keyed by relative path. In linear time, we evaluate all entries: paths present in the new snapshot but missing in the old are tagged `ADDED`; paths missing in the new are tagged `REMOVED`; and matching paths are compared by size delta to categorize them as `GROWN`, `SHRUNK`, or `UNCHANGED`. This avoids recursive nested searches."*

### Q11: What is the purpose of the `zeroize` crate in your crypto engine?
* **Model Answer**:
  > *"Standard variables in memory are not cleared when dropped; they remain in the OS process heap until overwritten, which makes them vulnerable to memory dump extraction or cold-boot attacks. The `zeroize` crate ensures that master password buffers and derived 256-bit keys implement `ZeroizeOnDrop`, writing volatile zeros over the memory locations as soon as they leave scope."*

### Q12: How do you handle terminal resizing and crash recovery?
* **Model Answer**:
  > *"When running in raw mode, an unhandled panic leaves the terminal in a broken state where enter keys and echoes fail. In `src/main.rs`, we register a custom `std::panic::set_hook`. If a panic occurs anywhere in the program, the hook immediately invokes `disable_raw_mode()` and `LeaveAlternateScreen` before printing the error trace. For resizing, Ratatui automatically listens for POSIX `SIGWINCH` signals via crossterm and recalculates viewport layouts on every frame."*
