use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use strata::crypto::CryptoEngine;
use strata::frecency::FrecencyStore;
use strata::model::{DirectoryTree, NodeType};
use strata::snapshot::{DiffKind, Snapshot, SnapshotItem, SnapshotManager};

#[test]
fn test_tree_aggregation() {
    let mut tree = DirectoryTree::new(PathBuf::from("/test"));
    let root_id = tree.root_id;

    let sub_id = tree.insert_entry(PathBuf::from("/test/sub"), NodeType::Directory, 0, 1000);

    tree.insert_entry(
        PathBuf::from("/test/sub/file1.txt"),
        NodeType::File,
        1024,
        1000,
    );
    tree.insert_entry(
        PathBuf::from("/test/sub/file2.txt"),
        NodeType::File,
        2048,
        1000,
    );

    tree.aggregate_sizes();

    let sub_node = tree.get_node(sub_id).unwrap();
    assert_eq!(sub_node.disk_size, 3072);

    let root_node = tree.get_node(root_id).unwrap();
    assert_eq!(root_node.disk_size, 3072);
}

#[test]
fn test_frecency_scoring() {
    let mut store = FrecencyStore::default();
    let path_a = PathBuf::from("/home/user/workspace/strata");
    let path_b = PathBuf::from("/home/user/downloads");

    store.record_visit(&path_a);
    store.record_visit(&path_a);
    store.record_visit(&path_b);

    let query_results = store.query("strat");
    assert!(!query_results.is_empty());
    assert_eq!(query_results[0].0, path_a);
}

#[test]
fn test_snapshot_diff() {
    let old_snap = Snapshot {
        id: "snap_old".into(),
        created_at: 1000,
        root_path: PathBuf::from("/proj"),
        total_size: 5000,
        total_items: 2,
        items: vec![
            SnapshotItem {
                relative_path: PathBuf::from("keep.txt"),
                is_dir: false,
                disk_size: 2000,
                modified: 1000,
            },
            SnapshotItem {
                relative_path: PathBuf::from("deleted.txt"),
                is_dir: false,
                disk_size: 3000,
                modified: 1000,
            },
        ],
    };

    let new_snap = Snapshot {
        id: "snap_new".into(),
        created_at: 2000,
        root_path: PathBuf::from("/proj"),
        total_size: 7000,
        total_items: 2,
        items: vec![
            SnapshotItem {
                relative_path: PathBuf::from("keep.txt"),
                is_dir: false,
                disk_size: 4000, // Grown by 2000
                modified: 2000,
            },
            SnapshotItem {
                relative_path: PathBuf::from("new.txt"),
                is_dir: false,
                disk_size: 3000, // Added
                modified: 2000,
            },
        ],
    };

    let diffs = SnapshotManager::diff(&old_snap, &new_snap);

    let grown = diffs
        .iter()
        .find(|d| d.relative_path == std::path::Path::new("keep.txt"))
        .unwrap();
    assert_eq!(grown.kind, DiffKind::Grown);
    assert_eq!(grown.size_delta, 2000);

    let added = diffs
        .iter()
        .find(|d| d.relative_path == std::path::Path::new("new.txt"))
        .unwrap();
    assert_eq!(added.kind, DiffKind::Added);

    let removed = diffs
        .iter()
        .find(|d| d.relative_path == std::path::Path::new("deleted.txt"))
        .unwrap();
    assert_eq!(removed.kind, DiffKind::Removed);
}

#[test]
fn test_aes_gcm_crypto_roundtrip() {
    let temp_dir = std::env::temp_dir().join("strata_test_crypto");
    let _ = fs::create_dir_all(&temp_dir);

    let secret_file = temp_dir.join("secret.txt");
    {
        let mut f = File::create(&secret_file).unwrap();
        f.write_all(b"Strata AES-256-GCM authenticated payload")
            .unwrap();
    }

    let password = "SuperSecretPassword123!";

    // Encrypt
    let enc_path = CryptoEngine::encrypt_path(&secret_file, password).unwrap();
    assert!(!secret_file.exists());
    assert!(enc_path.exists());
    assert!(enc_path.to_string_lossy().ends_with(".strata"));

    // Decrypt
    let dec_path = CryptoEngine::decrypt_path(&enc_path, password).unwrap();
    assert!(dec_path.exists());
    let contents = fs::read_to_string(&dec_path).unwrap();
    assert_eq!(contents, "Strata AES-256-GCM authenticated payload");

    // Clean up
    let _ = fs::remove_dir_all(temp_dir);
}
