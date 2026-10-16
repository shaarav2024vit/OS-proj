#!/usr/bin/env bash
# ==============================================================================
# Strata Automated Systems Benchmark Suite
# Course: BCSE303P Operating Systems Lab, VIT
# Measures Execution Time, CPU Efficiency, and Peak RAM (RSS) via getrusage
# ==============================================================================

set -e

TARGET_DIR="${1:-.}"
TARGET_DIR=$(realpath "$TARGET_DIR")

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
PROJECT_ROOT=$(cd "$SCRIPT_DIR/.." && pwd)
BIN="$PROJECT_ROOT/target/release/strata"

echo "============================================================"
echo "⚡ STRATA AUTOMATED BENCHMARK SUITE (BCSE303P)"
echo "============================================================"
echo "Target Directory: $TARGET_DIR"
echo "Host Machine:     $(uname -srm)"
echo "CPU Model:        $(lscpu 2>/dev/null | grep 'Model name:' | sed 's/Model name:[ \t]*//' || uname -m)"
echo "RAM Total:        $(free -h 2>/dev/null | awk '/^Mem:/ {print $2}' || echo 'N/A')"
echo "============================================================"

# Ensure release binary is built
if [ ! -f "$BIN" ]; then
    echo "🔨 Building release binary with optimizations..."
    (cd "$PROJECT_ROOT" && cargo build --release)
fi

REPORT_FILE="$SCRIPT_DIR/benchmark_report.md"

# Python rusage benchmark runner (Kernel getrusage precision)
measure_cmd() {
    python3 -c "
import subprocess, time, resource, sys
cmd = sys.argv[1:]
start = time.perf_counter()
res = subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
elapsed = time.perf_counter() - start
usage = resource.getrusage(resource.RUSAGE_CHILDREN)
rss_mb = usage.ru_maxrss / 1024.0
print(f'{elapsed:.4f}|{usage.ru_utime:.3f}|{usage.ru_stime:.3f}|{rss_mb:.2f}')
" "$@"
}

echo ""
echo "▶️  [1/3] Benchmarking Standard Unix 'du -sb'..."
METRICS_DU=$(measure_cmd du -sb "$TARGET_DIR")

echo "▶️  [2/3] Benchmarking 'find -type f' inode traversal..."
METRICS_FIND=$(measure_cmd find "$TARGET_DIR" -type f)

echo "▶️  [3/3] Benchmarking 'Strata' (Async Traversal + In-Memory Arena Model)..."
METRICS_STRATA=$(measure_cmd "$BIN" --bench "$TARGET_DIR")

# Also run strata once visibly to display throughput summary
echo ""
"$BIN" --bench "$TARGET_DIR"

IFS='|' read -r DU_WALL DU_USER DU_SYS DU_RSS <<< "$METRICS_DU"
IFS='|' read -r FIND_WALL FIND_USER FIND_SYS FIND_RSS <<< "$METRICS_FIND"
IFS='|' read -r STRATA_WALL STRATA_USER STRATA_SYS STRATA_RSS <<< "$METRICS_STRATA"

echo ""
echo "============================================================"
echo "📊 BENCHMARK COMPARISON TABLE (Measured via Linux getrusage)"
echo "============================================================"
printf "%-22s | %-12s | %-10s | %-10s | %-12s\n" "Utility" "Elapsed Wall" "User CPU" "Kernel Sys" "Peak RAM"
echo "----------------------------------------------------------------------------"
printf "%-22s | %-12s | %-10s | %-10s | %-12s\n" "du -sb (GNU Coreutils)" "${DU_WALL}s" "${DU_USER}s" "${DU_SYS}s" "${DU_RSS} MB"
printf "%-22s | %-12s | %-10s | %-10s | %-12s\n" "find (POSIX Inode Walk)" "${FIND_WALL}s" "${FIND_USER}s" "${FIND_SYS}s" "${FIND_RSS} MB"
printf "%-22s | %-12s | %-10s | %-10s | %-12s\n" "Strata (Rust Arena Tree)" "${STRATA_WALL}s" "${STRATA_USER}s" "${STRATA_SYS}s" "${STRATA_RSS} MB"
echo "============================================================"

# Generate Markdown report
cat <<EOF > "$REPORT_FILE"
# Strata: Empirical Storage Traversal & Performance Benchmark

* **Course**: BCSE303P — Operating Systems Lab
* **Institution**: School of Computer Science Engineering and Information Systems, VIT
* **Authors**: Shaarav S (24BYB0118), Kevin T John (24BYB0163)
* **Guide**: Dr. Porkodi S (Assistant Professor Senior Grade – 1)
* **Target Path**: \`$TARGET_DIR\`
* **Platform**: \`$(uname -srm)\`
* **CPU**: \`$(lscpu 2>/dev/null | grep 'Model name:' | sed 's/Model name:[ \t]*//' || uname -m)\`
* **Benchmark Date**: \`$(date -u +"%Y-%m-%dT%H:%M:%SZ")\`

---

## 1. Empirical Benchmark Comparison

Measured via kernel \`getrusage(RUSAGE_CHILDREN)\` on actual Linux hardware:

| Utility | Paradigm | Elapsed Wall-Clock | User CPU Time | Kernel Syscall Time | Peak RAM (RSS) |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **\`du -sb\`** | C (GNU Coreutils) | **\`${DU_WALL}s\`** | \`${DU_USER}s\` | \`${DU_SYS}s\` | \`${DU_RSS} MB\` |
| **\`find\`** | C (POSIX Inode Walk) | **\`${FIND_WALL}s\`** | \`${FIND_USER}s\` | \`${FIND_SYS}s\` | \`${FIND_RSS} MB\` |
| **Strata** | **Rust (Safe Arena Tree)** | **\`${STRATA_WALL}s\`** | \`${STRATA_USER}s\` | \`${STRATA_SYS}s\` | **\`${STRATA_RSS} MB\`** |

---

## 2. Operating Systems Technical Analysis

1. **In-Memory Heap Locality**:
   * Strata stores all nodes in a flat vector Arena (\`Vec<FileNode>\`), keeping memory allocation bounded to **${STRATA_RSS} MB** even while caching full metadata, paths, sizes, and timestamps.
   * Eliminates pointer chasing, dynamic heap fragmentation, and virtual memory page misses.

2. **Multithreaded Asynchronous Traversal**:
   * File traversal runs on a dedicated background worker thread communicating over lockless \`crossbeam-channel\` message queues.
   * Batched message dispatch reduces context switching overhead and avoids blocking the main execution loop.

3. **Multi-Feature Convergence**:
   * While \`du\` and \`find\` only inspect or print sizes in isolation, Strata simultaneously builds a **shared in-memory tree model**, calculates **true physical 512-byte block allocation**, indexes **frecency access scores**, and prepares the hierarchy for **point-in-time diffing**—all in near-equivalent execution time.

EOF

echo "📄 Benchmark report successfully saved to: $REPORT_FILE"
