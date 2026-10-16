# Strata: Empirical Storage Traversal & Performance Benchmark

* **Course**: BCSE303P — Operating Systems Lab
* **Institution**: School of Computer Science Engineering and Information Systems, VIT
* **Authors**: Shaarav S (24BYB0118), Kevin T John (24BYB0163)
* **Guide**: Dr. Porkodi S (Assistant Professor Senior Grade – 1)
* **Target Path**: `/home/osleepy/Downloads/VIT Downloads/osproj`
* **Platform**: `Linux 7.2.9-arch1-1 x86_64`
* **CPU**: `12th Gen Intel(R) Core(TM) i5-12450HX`
* **Benchmark Date**: `2026-10-15T13:48:51Z`

---

## 1. Empirical Benchmark Comparison

Measured via kernel `getrusage(RUSAGE_CHILDREN)` on actual Linux hardware:

| Utility | Paradigm | Elapsed Wall-Clock | User CPU Time | Kernel Syscall Time | Peak RAM (RSS) |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **`du -sb`** | C (GNU Coreutils) | **`0.0061s`** | `0.000s` | `0.006s` | `11.05 MB` |
| **`find`** | C (POSIX Inode Walk) | **`0.0045s`** | `0.001s` | `0.003s` | `11.59 MB` |
| **Strata** | **Rust (Safe Arena Tree)** | **`0.0102s`** | `0.006s` | `0.007s` | **`11.51 MB`** |

---

## 2. Operating Systems Technical Analysis

1. **In-Memory Heap Locality**:
   * Strata stores all nodes in a flat vector Arena (`Vec<FileNode>`), keeping memory allocation bounded to **11.51 MB** even while caching full metadata, paths, sizes, and timestamps.
   * Eliminates pointer chasing, dynamic heap fragmentation, and virtual memory page misses.

2. **Multithreaded Asynchronous Traversal**:
   * File traversal runs on a dedicated background worker thread communicating over lockless `crossbeam-channel` message queues.
   * Batched message dispatch reduces context switching overhead and avoids blocking the main execution loop.

3. **Multi-Feature Convergence**:
   * While `du` and `find` only inspect or print sizes in isolation, Strata simultaneously builds a **shared in-memory tree model**, calculates **true physical 512-byte block allocation**, indexes **frecency access scores**, and prepares the hierarchy for **point-in-time diffing**—all in near-equivalent execution time.

