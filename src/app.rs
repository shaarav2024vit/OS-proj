use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use crossbeam_channel::{unbounded, Receiver, Sender};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Frame;

use crate::crypto::CryptoEngine;
use crate::frecency::FrecencyStore;
use crate::model::{get_volume_info, DirectoryTree, NodeId, SortOrder, VolumeInfo};
use crate::scanner::{Scanner, ScannerMessage};
use crate::snapshot::{DiffEntry, Snapshot, SnapshotManager};
use crate::ui::components::*;
use crate::ui::theme::{Theme, ThemeName};

pub enum ModalState {
    None,
    Jump {
        query: String,
        results: Vec<(PathBuf, i64, Vec<usize>)>,
        selected_index: usize,
    },
    Diff {
        snapshots: Vec<Snapshot>,
        active_snap_idx: usize,
        diffs: Vec<DiffEntry>,
        selected_diff_idx: usize,
    },
    Crypto {
        action: CryptoAction,
        target_path: PathBuf,
        password: String,
        confirm: String,
        is_confirm_field: bool,
        status_msg: Option<String>,
    },
    Inode {
        target_path: PathBuf,
    },
    TopHogs {
        hogs: Vec<SpaceHogEntry>,
        selected_index: usize,
    },
    Delete {
        target_path: PathBuf,
        is_dir: bool,
        reclaimed_bytes: u64,
    },
    Breakdown {
        categories: Vec<CategoryStat>,
        total_bytes: u64,
        selected_index: usize,
    },
    Filter,
}

pub struct App {
    pub tree: DirectoryTree,
    pub current_dir_id: NodeId,
    pub selected_index: usize,
    pub sort_order: SortOrder,
    pub frecency_store: FrecencyStore,
    pub theme: Theme,
    pub theme_name: ThemeName,

    pub is_scanning: bool,
    pub status_message: String,
    pub scanner_rx: Receiver<ScannerMessage>,
    pub scanner_tx: Sender<ScannerMessage>,
    pub scan_cancel: Arc<AtomicBool>,

    pub preview_enabled: bool,
    pub filter_query: String,
    pub frame_tick: usize,
    pub volume_info: Option<VolumeInfo>,
    pub last_terminal_size: std::cell::Cell<(u16, u16)>,

    pub modal: ModalState,
    pub should_quit: bool,
}

impl App {
    pub fn new(start_path: PathBuf) -> Self {
        let (tx, rx) = unbounded();
        let cancel = Arc::new(AtomicBool::new(false));

        let tree = DirectoryTree::new(start_path.clone());
        let mut frecency = FrecencyStore::load_or_default();
        frecency.record_visit(&start_path);
        let volume_info = get_volume_info(&start_path);
        let theme_name = ThemeName::CatppuccinMocha;

        let mut app = Self {
            tree,
            current_dir_id: 0,
            selected_index: 0,
            sort_order: SortOrder::SizeDesc,
            frecency_store: frecency,
            theme: Theme::from_name(theme_name),
            theme_name,

            is_scanning: false,
            status_message: "Ready".to_string(),
            scanner_rx: rx,
            scanner_tx: tx,
            scan_cancel: cancel,

            preview_enabled: true,
            filter_query: String::new(),
            frame_tick: 0,
            volume_info,
            last_terminal_size: std::cell::Cell::new((0, 0)),

            modal: ModalState::None,
            should_quit: false,
        };

        app.trigger_rescan(start_path);
        app
    }

    pub fn trigger_rescan(&mut self, path: PathBuf) {
        self.scan_cancel.store(true, Ordering::Relaxed);
        let new_cancel = Arc::new(AtomicBool::new(false));
        self.scan_cancel = new_cancel.clone();

        self.volume_info = get_volume_info(&path);
        self.tree = DirectoryTree::new(path.clone());
        self.current_dir_id = 0;
        self.selected_index = 0;
        self.is_scanning = true;
        self.status_message = format!("Scanning {}...", path.display());

        Scanner::start_scan(path, new_cancel, self.scanner_tx.clone());
    }

    pub fn handle_scanner_messages(&mut self) {
        self.frame_tick = self.frame_tick.wrapping_add(1);

        while let Ok(msg) = self.scanner_rx.try_recv() {
            match msg {
                ScannerMessage::Batch(entries) => {
                    for entry in entries {
                        self.tree.insert_entry(
                            entry.path,
                            entry.node_type,
                            entry.size,
                            entry.modified,
                        );
                    }
                    self.tree.aggregate_sizes();
                }
                ScannerMessage::Progress { files, current_path, .. } => {
                    self.status_message = format!(
                        "Scanning: {} items ({})",
                        files,
                        current_path.file_name().and_then(|n| n.to_str()).unwrap_or("")
                    );
                }
                ScannerMessage::Complete { total_files, .. } => {
                    self.tree.aggregate_sizes();
                    self.is_scanning = false;
                    self.status_message = format!(
                        "Scan complete: {} items indexed",
                        total_files
                    );
                }
                ScannerMessage::Error(_) => {}
            }
        }
    }

    pub fn current_children(&self) -> Vec<NodeId> {
        let all_sorted = self.tree.sorted_children(self.current_dir_id, self.sort_order);
        if self.filter_query.is_empty() {
            all_sorted
        } else {
            let q = self.filter_query.to_lowercase();
            all_sorted
                .into_iter()
                .filter(|&id| {
                    if let Some(node) = self.tree.get_node(id) {
                        node.name.to_lowercase().contains(&q)
                    } else {
                        false
                    }
                })
                .collect()
        }
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::ScrollDown => {
                let children = self.current_children();
                if !children.is_empty() {
                    self.selected_index = (self.selected_index + 1) % children.len();
                }
            }
            MouseEventKind::ScrollUp => {
                let children = self.current_children();
                if !children.is_empty() {
                    self.selected_index = if self.selected_index == 0 {
                        children.len() - 1
                    } else {
                        self.selected_index - 1
                    };
                }
            }
            MouseEventKind::Down(MouseButton::Left) => {
                self.handle_mouse_click(mouse.column, mouse.row);
            }
            _ => {}
        }
    }

    fn handle_mouse_click(&mut self, col: u16, row: u16) {
        let (term_width, term_height) = self.last_terminal_size.get();
        if term_width == 0 || term_height == 0 {
            return;
        }

        let table_width = if self.preview_enabled && term_width >= 80 {
            term_width * 58 / 100
        } else {
            term_width
        };

        // Click inside Explorer table area (row 1 is volume bar, row 2 is table start)
        if col < table_width && row >= 1 && row < term_height.saturating_sub(2) {
            let table_start_y = 1;
            // Rows start 3 lines down from table border (top border + header + margin)
            if row >= table_start_y + 3 {
                let clicked_idx = (row - (table_start_y + 3)) as usize;
                let children = self.current_children();
                if clicked_idx < children.len() {
                    if clicked_idx == self.selected_index {
                        // Double-click on already-selected row -> Drill down!
                        if let Some(&child_id) = children.get(clicked_idx) {
                            if let Some(node) = self.tree.get_node(child_id) {
                                if node.is_dir() {
                                    let path = node.path.clone();
                                    self.current_dir_id = child_id;
                                    self.selected_index = 0;
                                    self.filter_query.clear();
                                    self.frecency_store.record_visit(&path);
                                    self.volume_info = get_volume_info(&path);
                                }
                            }
                        }
                    } else {
                        self.selected_index = clicked_idx;
                    }
                }
            } else if row == table_start_y {
                // Clicked on table title/header border: Ascend to parent directory!
                if let Some(current_node) = self.tree.get_node(self.current_dir_id) {
                    if let Some(parent_id) = current_node.parent {
                        self.current_dir_id = parent_id;
                        self.selected_index = 0;
                        self.filter_query.clear();
                    }
                }
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match &mut self.modal {
            ModalState::None => self.handle_normal_key(key),
            ModalState::Jump { .. } => self.handle_jump_key(key),
            ModalState::Diff { .. } => self.handle_diff_key(key),
            ModalState::Crypto { .. } => self.handle_crypto_key(key),
            ModalState::Inode { .. } => self.handle_inode_key(key),
            ModalState::TopHogs { .. } => self.handle_top_hogs_key(key),
            ModalState::Delete { .. } => self.handle_delete_key(key),
            ModalState::Breakdown { .. } => self.handle_breakdown_key(key),
            ModalState::Filter => self.handle_filter_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) {
        let children = self.current_children();
        let child_count = children.len();

        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => self.should_quit = true,

            // Toggle live file preview pane
            KeyCode::Tab => {
                self.preview_enabled = !self.preview_enabled;
            }

            // Dynamic live theme switcher — cycle 4 iconic Linux rice palettes
            KeyCode::Char('T') => {
                self.theme_name = self.theme_name.next();
                self.theme = Theme::from_name(self.theme_name);
                self.status_message = format!("Theme: {}", self.theme_name.label());
            }

            // File Category Storage Breakdown
            KeyCode::Char('b') => {
                let (categories, total_bytes) = calculate_category_stats(&self.tree);
                self.modal = ModalState::Breakdown {
                    categories,
                    total_bytes,
                    selected_index: 0,
                };
            }

            // In-place live search/filter
            KeyCode::Char('/') => {
                self.modal = ModalState::Filter;
            }

            // Inode metadata inspector
            KeyCode::Char('i') => {
                if let Some(&child_id) = children.get(self.selected_index) {
                    if let Some(node) = self.tree.get_node(child_id) {
                        let path = node.path.clone();
                        self.modal = ModalState::Inode { target_path: path };
                    }
                } else if let Some(current) = self.tree.get_node(self.current_dir_id) {
                    let path = current.path.clone();
                    self.modal = ModalState::Inode { target_path: path };
                }
            }

            // Top 20 space hogs dashboard
            KeyCode::Char('t') => {
                let hogs = collect_space_hogs(&self.tree, 20);
                self.modal = ModalState::TopHogs {
                    hogs,
                    selected_index: 0,
                };
            }

            // Space reclamation delete modal
            KeyCode::Char('x') | KeyCode::Delete => {
                if let Some(&child_id) = children.get(self.selected_index) {
                    if let Some(node) = self.tree.get_node(child_id) {
                        self.modal = ModalState::Delete {
                            target_path: node.path.clone(),
                            is_dir: node.is_dir(),
                            reclaimed_bytes: node.disk_size,
                        };
                    }
                }
            }

            // Navigation
            KeyCode::Char('j') | KeyCode::Down => {
                if child_count > 0 {
                    self.selected_index = (self.selected_index + 1) % child_count;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if child_count > 0 {
                    self.selected_index = if self.selected_index == 0 {
                        child_count - 1
                    } else {
                        self.selected_index - 1
                    };
                }
            }
            KeyCode::Char('l') | KeyCode::Enter | KeyCode::Right => {
                if let Some(&child_id) = children.get(self.selected_index) {
                    if let Some(child_node) = self.tree.get_node(child_id) {
                        if child_node.is_dir() {
                            let path = child_node.path.clone();
                            self.current_dir_id = child_id;
                            self.selected_index = 0;
                            self.filter_query.clear();
                            self.frecency_store.record_visit(&path);
                            self.volume_info = get_volume_info(&path);
                        }
                    }
                }
            }
            KeyCode::Char('h') | KeyCode::Backspace | KeyCode::Left => {
                if let Some(current_node) = self.tree.get_node(self.current_dir_id) {
                    if let Some(parent_id) = current_node.parent {
                        self.current_dir_id = parent_id;
                        self.selected_index = 0;
                        self.filter_query.clear();
                    }
                }
            }

            // Sorting cycle
            KeyCode::Char('s') => {
                self.sort_order = match self.sort_order {
                    SortOrder::SizeDesc => SortOrder::SizeAsc,
                    SortOrder::SizeAsc => SortOrder::NameAsc,
                    SortOrder::NameAsc => SortOrder::NameDesc,
                    SortOrder::NameDesc => SortOrder::CountDesc,
                    SortOrder::CountDesc => SortOrder::ModifiedDesc,
                    SortOrder::ModifiedDesc => SortOrder::SizeDesc,
                };
            }

            // Quick-jump modal
            KeyCode::Char('f') | KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let initial_query = "".to_string();
                let results = self.frecency_store.query(&initial_query);
                self.modal = ModalState::Jump {
                    query: initial_query,
                    results,
                    selected_index: 0,
                };
            }

            // Point-in-time Snapshot
            KeyCode::Char('p') => {
                if let Some(curr) = self.tree.get_node(self.current_dir_id) {
                    let path = curr.path.clone();
                    match SnapshotManager::create_snapshot(&self.tree, &path) {
                        Ok(snap) => {
                            self.status_message = format!("Snapshot saved: {}", snap.id);
                        }
                        Err(err) => {
                            self.status_message = format!("Snapshot failed: {}", err);
                        }
                    }
                }
            }

            // Snapshot Diff modal
            KeyCode::Char('d') => {
                let snapshots = SnapshotManager::list_snapshots();
                if snapshots.is_empty() {
                    self.status_message = "No snapshots found. Press [p] to create one!".into();
                } else {
                    let curr_path = self.tree.get_node(self.current_dir_id).map(|n| n.path.clone()).unwrap_or_default();
                    let live_snap = SnapshotManager::create_snapshot(&self.tree, &curr_path).ok();
                    let diffs = if let Some(live) = live_snap {
                        SnapshotManager::diff(&snapshots[0], &live)
                    } else {
                        Vec::new()
                    };

                    self.modal = ModalState::Diff {
                        snapshots,
                        active_snap_idx: 0,
                        diffs,
                        selected_diff_idx: 0,
                    };
                }
            }

            // Inline Encryption (with confirmation)
            KeyCode::Char('e') => {
                if let Some(&child_id) = children.get(self.selected_index) {
                    if let Some(node) = self.tree.get_node(child_id) {
                        let path = node.path.clone();
                        self.modal = ModalState::Crypto {
                            action: CryptoAction::Encrypt,
                            target_path: path,
                            password: String::new(),
                            confirm: String::new(),
                            is_confirm_field: false,
                            status_msg: None,
                        };
                    }
                }
            }

            // Inline Decryption
            KeyCode::Char('u') => {
                if let Some(&child_id) = children.get(self.selected_index) {
                    if let Some(node) = self.tree.get_node(child_id) {
                        let path = node.path.clone();
                        if path.to_string_lossy().ends_with(".strata") {
                            self.modal = ModalState::Crypto {
                                action: CryptoAction::Decrypt,
                                target_path: path,
                                password: String::new(),
                                confirm: String::new(),
                                is_confirm_field: false,
                                status_msg: None,
                            };
                        } else {
                            self.status_message = "Select a .strata file to decrypt".into();
                        }
                    }
                }
            }

            // Rescan
            KeyCode::Char('r') => {
                if let Some(curr) = self.tree.get_node(self.current_dir_id) {
                    let path = curr.path.clone();
                    self.trigger_rescan(path);
                }
            }

            // Clear filter if Esc
            KeyCode::Esc => {
                self.filter_query.clear();
            }

            _ => {}
        }
    }

    fn handle_jump_key(&mut self, key: KeyEvent) {
        if let ModalState::Jump { query, results, selected_index } = &mut self.modal {
            match key.code {
                KeyCode::Esc => self.modal = ModalState::None,
                KeyCode::Char(c) => {
                    query.push(c);
                    *results = self.frecency_store.query(query);
                    *selected_index = 0;
                }
                KeyCode::Backspace => {
                    query.pop();
                    *results = self.frecency_store.query(query);
                    *selected_index = 0;
                }
                KeyCode::Down | KeyCode::Tab => {
                    if !results.is_empty() {
                        *selected_index = (*selected_index + 1) % results.len();
                    }
                }
                KeyCode::Up => {
                    if !results.is_empty() {
                        *selected_index = if *selected_index == 0 {
                            results.len() - 1
                        } else {
                            *selected_index - 1
                        };
                    }
                }
                KeyCode::Enter => {
                    if let Some((target_path, _, _)) = results.get(*selected_index).cloned() {
                        self.modal = ModalState::None;
                        self.frecency_store.record_visit(&target_path);
                        self.trigger_rescan(target_path);
                    }
                }
                _ => {}
            }
        }
    }

    fn handle_diff_key(&mut self, key: KeyEvent) {
        if let ModalState::Diff { snapshots, active_snap_idx, diffs, selected_diff_idx } = &mut self.modal {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => self.modal = ModalState::None,
                KeyCode::Down | KeyCode::Char('j') => {
                    if !diffs.is_empty() {
                        *selected_diff_idx = (*selected_diff_idx + 1) % diffs.len();
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if !diffs.is_empty() {
                        *selected_diff_idx = if *selected_diff_idx == 0 {
                            diffs.len() - 1
                        } else {
                            *selected_diff_idx - 1
                        };
                    }
                }
                KeyCode::Tab => {
                    if !snapshots.is_empty() {
                        *active_snap_idx = (*active_snap_idx + 1) % snapshots.len();
                        let curr_path = self.tree.get_node(self.current_dir_id).map(|n| n.path.clone()).unwrap_or_default();
                        if let Ok(live) = SnapshotManager::create_snapshot(&self.tree, &curr_path) {
                            *diffs = SnapshotManager::diff(&snapshots[*active_snap_idx], &live);
                            *selected_diff_idx = 0;
                        }
                    }
                }
                // Export report to Markdown
                KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if let Some(snap) = snapshots.get(*active_snap_idx) {
                        let curr_path = self.tree.get_node(self.current_dir_id).map(|n| n.path.clone()).unwrap_or_default();
                        if let Ok(live) = SnapshotManager::create_snapshot(&self.tree, &curr_path) {
                            let report_path = PathBuf::from("strata_diff_report.md");
                            if let Ok(p) = SnapshotManager::export_markdown_report(snap, &live, diffs, &report_path) {
                                self.status_message = format!("Exported report to {}", p.display());
                                self.modal = ModalState::None;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn handle_crypto_key(&mut self, key: KeyEvent) {
        let mut close_modal = false;
        let mut do_action = false;

        if let ModalState::Crypto { action, password, confirm, is_confirm_field, status_msg, .. } = &mut self.modal {
            match key.code {
                KeyCode::Esc => close_modal = true,
                KeyCode::Tab => {
                    if *action == CryptoAction::Encrypt {
                        *is_confirm_field = !*is_confirm_field;
                    }
                }
                KeyCode::Char(c) => {
                    if *is_confirm_field {
                        confirm.push(c);
                    } else {
                        password.push(c);
                    }
                }
                KeyCode::Backspace => {
                    if *is_confirm_field {
                        confirm.pop();
                    } else {
                        password.pop();
                    }
                }
                KeyCode::Enter => {
                    if password.is_empty() {
                        *status_msg = Some("Passphrase cannot be empty!".into());
                    } else if *action == CryptoAction::Encrypt && password != confirm {
                        *status_msg = Some("Passphrases do not match!".into());
                    } else {
                        do_action = true;
                    }
                }
                _ => {}
            }
        }

        if close_modal {
            self.modal = ModalState::None;
        } else if do_action {
            if let ModalState::Crypto { action, target_path, password, status_msg, .. } = &mut self.modal {
                let res = match action {
                    CryptoAction::Encrypt => CryptoEngine::encrypt_path(target_path, password),
                    CryptoAction::Decrypt => CryptoEngine::decrypt_path(target_path, password),
                };

                match res {
                    Ok(dest) => {
                        self.status_message = format!("Success: {}", dest.display());
                        self.modal = ModalState::None;
                        if let Some(curr) = self.tree.get_node(self.current_dir_id) {
                            let p = curr.path.clone();
                            self.trigger_rescan(p);
                        }
                    }
                    Err(err) => {
                        *status_msg = Some(err);
                    }
                }
            }
        }
    }

    fn handle_inode_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('i') => {
                self.modal = ModalState::None;
            }
            _ => {}
        }
    }

    fn handle_top_hogs_key(&mut self, key: KeyEvent) {
        if let ModalState::TopHogs { hogs, selected_index } = &mut self.modal {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('t') => {
                    self.modal = ModalState::None;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if !hogs.is_empty() {
                        *selected_index = (*selected_index + 1) % hogs.len();
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if !hogs.is_empty() {
                        *selected_index = if *selected_index == 0 {
                            hogs.len() - 1
                        } else {
                            *selected_index - 1
                        };
                    }
                }
                KeyCode::Enter => {
                    if let Some(entry) = hogs.get(*selected_index) {
                        if let Some(parent_id) = entry.parent_id {
                            self.current_dir_id = parent_id;
                            self.selected_index = 0;
                            self.modal = ModalState::None;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn handle_delete_key(&mut self, key: KeyEvent) {
        if let ModalState::Delete { target_path, is_dir, .. } = &self.modal {
            match key.code {
                KeyCode::Char('y') => {
                    let res = if *is_dir {
                        fs::remove_dir_all(target_path)
                    } else {
                        fs::remove_file(target_path)
                    };

                    match res {
                        Ok(_) => {
                            self.status_message = format!("Deleted: {}", target_path.display());
                            self.modal = ModalState::None;
                            if let Some(curr) = self.tree.get_node(self.current_dir_id) {
                                let p = curr.path.clone();
                                self.trigger_rescan(p);
                            }
                        }
                        Err(err) => {
                            self.status_message = format!("Delete error: {}", err);
                            self.modal = ModalState::None;
                        }
                    }
                }
                KeyCode::Char('n') | KeyCode::Esc | KeyCode::Char('q') => {
                    self.modal = ModalState::None;
                }
                _ => {}
            }
        }
    }

    fn handle_breakdown_key(&mut self, key: KeyEvent) {
        if let ModalState::Breakdown { categories, selected_index, .. } = &mut self.modal {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('b') => {
                    self.modal = ModalState::None;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if !categories.is_empty() {
                        *selected_index = (*selected_index + 1) % categories.len();
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if !categories.is_empty() {
                        *selected_index = if *selected_index == 0 {
                            categories.len() - 1
                        } else {
                            *selected_index - 1
                        };
                    }
                }
                _ => {}
            }
        }
    }

    fn handle_filter_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.modal = ModalState::None;
            }
            KeyCode::Enter => {
                self.modal = ModalState::None;
            }
            KeyCode::Backspace => {
                self.filter_query.pop();
                self.selected_index = 0;
            }
            KeyCode::Char(c) => {
                self.filter_query.push(c);
                self.selected_index = 0;
            }
            _ => {}
        }
    }

    pub fn render(&self, f: &mut Frame) {
        let size = f.area();
        self.last_terminal_size.set((size.width, size.height));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Top Volume & Mountpoint Bar
                Constraint::Min(3),    // Main File Table & Live Preview
                Constraint::Length(2), // Bottom Status Bar
            ])
            .split(size);

        render_volume_bar(f, chunks[0], self.volume_info.as_ref(), &self.theme);

        let children = self.current_children();
        let current_node = self.tree.get_node(self.current_dir_id);
        let current_path = current_node.map(|n| n.path.to_string_lossy().to_string()).unwrap_or_else(|| "/".into());
        let total_size = current_node.map(|n| n.disk_size).unwrap_or(0);
        let item_count = self.tree.nodes.len();

        let selected_child_id = children.get(self.selected_index).copied();

        // Check if preview pane split should be displayed
        if self.preview_enabled && size.width >= 80 {
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
                .split(chunks[1]);

            render_file_table(
                f,
                main_chunks[0],
                &self.tree,
                self.current_dir_id,
                &children,
                self.selected_index,
                self.sort_order,
                &self.filter_query,
                &self.theme,
            );

            render_preview_pane(
                f,
                main_chunks[1],
                &self.tree,
                selected_child_id,
                &self.theme,
            );
        } else {
            render_file_table(
                f,
                chunks[1],
                &self.tree,
                self.current_dir_id,
                &children,
                self.selected_index,
                self.sort_order,
                &self.filter_query,
                &self.theme,
            );
        }

        let mode = if self.is_scanning {
            AppMode::Scanning
        } else {
            match &self.modal {
                ModalState::None => AppMode::Normal,
                ModalState::Jump { .. } => AppMode::Jump,
                ModalState::Diff { .. } => AppMode::Diff,
                ModalState::Crypto { .. } => AppMode::Crypto,
                ModalState::Inode { .. } => AppMode::Inode,
                ModalState::TopHogs { .. } => AppMode::TopHogs,
                ModalState::Delete { .. } => AppMode::Delete,
                ModalState::Breakdown { .. } => AppMode::Breakdown,
                ModalState::Filter => AppMode::Filter,
            }
        };

        render_status_bar(
            f,
            chunks[2],
            mode,
            &current_path,
            total_size,
            item_count,
            &self.status_message,
            self.frame_tick,
            self.preview_enabled,
            &self.theme,
        );

        // Render modals on top
        match &self.modal {
            ModalState::None | ModalState::Filter => {}
            ModalState::Jump { query, results, selected_index } => {
                render_jump_modal(f, size, query, results, *selected_index, &self.theme);
            }
            ModalState::Diff { snapshots, active_snap_idx, diffs, selected_diff_idx } => {
                render_diff_modal(f, size, snapshots, *active_snap_idx, diffs, *selected_diff_idx, &self.theme);
            }
            ModalState::Crypto { action, target_path, password, confirm, is_confirm_field, status_msg } => {
                let confirm_len = if *action == CryptoAction::Encrypt { Some(confirm.len()) } else { None };
                render_crypto_modal(
                    f,
                    size,
                    *action,
                    target_path,
                    password.len(),
                    confirm_len,
                    *is_confirm_field,
                    status_msg.as_deref(),
                    &self.theme,
                );
            }
            ModalState::Inode { target_path } => {
                render_inode_modal(f, size, target_path, &self.theme);
            }
            ModalState::TopHogs { hogs, selected_index } => {
                render_top_hogs_modal(f, size, hogs, *selected_index, &self.theme);
            }
            ModalState::Delete { target_path, is_dir, reclaimed_bytes } => {
                render_delete_modal(f, size, target_path, *is_dir, *reclaimed_bytes, &self.theme);
            }
            ModalState::Breakdown { categories, total_bytes, selected_index } => {
                render_breakdown_modal(f, size, categories, *total_bytes, *selected_index, &self.theme);
            }
        }
    }
}
