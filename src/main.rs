use std::env;
use std::io::stdout;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::unbounded;
use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use strata::app::App;
use strata::model::DirectoryTree;
use strata::scanner::{Scanner, ScannerMessage};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // Check for CLI flags
    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            "--benchmark-scan" | "--bench" | "-b" => {
                let target = if args.len() > 2 {
                    PathBuf::from(&args[2])
                } else {
                    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
                };
                let target = target.canonicalize().unwrap_or(target);
                return run_benchmark_scan(target);
            }
            _ => {}
        }
    }

    // Setup panic hook to restore terminal on crash
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, crossterm::event::DisableMouseCapture);
        default_hook(panic_info);
    }));

    // Target path: CLI arg or current directory
    let target_path = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let target_path = target_path
        .canonicalize()
        .unwrap_or(target_path);

    // Initialize terminal in raw mode, alternate screen, and mouse capture
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, crossterm::event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Instantiate App
    let mut app = App::new(target_path);

    // Main 60 FPS event loop
    loop {
        // Ingest streaming background scanner messages
        app.handle_scanner_messages();

        // Render frame
        terminal.draw(|f| app.render(f))?;

        if app.should_quit {
            break;
        }

        // Poll for keyboard and mouse events
        if event::poll(Duration::from_millis(20))? {
            match event::read()? {
                Event::Key(key) => app.handle_key(key),
                Event::Mouse(mouse) => app.handle_mouse(mouse),
                _ => {}
            }
        }
    }

    // Clean teardown
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, crossterm::event::DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}

fn print_help() {
    println!("Strata v0.1.0 - Terminal File Manager with Disk Analytics, Snapshots & Encryption");
    println!("Course: BCSE303P Operating Systems Lab, VIT");
    println!("Authors: Shaarav S (24BYB0118), Kevin T John (24BYB0163)");
    println!();
    println!("USAGE:");
    println!("    strata [PATH]                  Launch interactive TUI in target directory");
    println!("    strata --bench [PATH]          Run headless benchmark scan with throughput metrics");
    println!("    strata --help                  Display this help message");
    println!();
    println!("KEYBINDINGS (in TUI):");
    println!("    j / k                          Navigate up / down");
    println!("    Enter / h                      Enter directory / Ascend to parent");
    println!("    Tab                            Toggle live file preview pane");
    println!("    b                              Storage breakdown by file category");
    println!("    i                              POSIX Inode & metadata inspector");
    println!("    t                              Top 20 space hogs dashboard");
    println!("    /                              In-place live filter by name");
    println!("    s                              Cycle sorting order");
    println!("    f                              Zoxide-style frecency quick-jump");
    println!("    p                              Capture point-in-time snapshot");
    println!("    d                              Snapshot differential inspector");
    println!("    e / u                          Encrypt file/folder / Decrypt .strata");
    println!("    x                              Delete file/folder with space check");
    println!("    T                              Cycle theme (Catppuccin, Tokyo Night, Gruvbox, Nord)");
    println!("    q                              Quit");
}

fn run_benchmark_scan(target: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    let (tx, rx) = unbounded();
    let cancel = Arc::new(AtomicBool::new(false));

    println!("============================================================");
    println!("⚡ STRATA EMPIRICAL FILESYSTEM SCAN BENCHMARK");
    println!("============================================================");
    println!("Target Path: {}", target.display());

    let mut tree = DirectoryTree::new(target.clone());
    let handle = Scanner::start_scan(target, cancel, tx);

    let mut total_files = 0;
    let mut total_bytes = 0;

    while let Ok(msg) = rx.recv() {
        match msg {
            ScannerMessage::Batch(entries) => {
                for entry in entries {
                    tree.insert_entry(entry.path, entry.node_type, entry.size, entry.modified);
                }
            }
            ScannerMessage::Complete { total_files: f, total_bytes: b } => {
                total_files = f;
                total_bytes = b;
                break;
            }
            _ => {}
        }
    }

    let _ = handle.join();
    let scan_time = start.elapsed();

    let agg_start = Instant::now();
    tree.aggregate_sizes();
    let agg_time = agg_start.elapsed();
    let total_time = start.elapsed();

    let mbs = (total_bytes as f64) / (1024.0 * 1024.0);
    let scan_secs = scan_time.as_secs_f64().max(0.0001);
    let files_per_sec = (total_files as f64) / scan_secs;
    let mb_per_sec = mbs / scan_secs;

    println!("------------------------------------------------------------");
    println!("📁 Inodes Traversed:       {} items", total_files);
    println!("💾 Total Disk Allocated:   {:.2} MB ({:.3} GB)", mbs, mbs / 1024.0);
    println!("⏱️  Scan & Ingest Time:     {:.3} s ({:?})", scan_secs, scan_time);
    println!("🌳 Arena Size Aggregation: {:.3} s ({:?})", agg_time.as_secs_f64(), agg_time);
    println!("🏁 Total Wall-Clock Time:  {:.3} s ({:?})", total_time.as_secs_f64(), total_time);
    println!("🚀 Traversal Throughput:   {:.0} inodes/sec ({:.2} MB/sec)", files_per_sec, mb_per_sec);
    println!("============================================================");

    Ok(())
}
