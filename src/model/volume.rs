use std::ffi::CString;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct VolumeInfo {
    pub device: String,
    pub mount_point: String,
    pub fs_type: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub total_inodes: u64,
    pub free_inodes: u64,
}

impl VolumeInfo {
    pub fn used_ratio(&self) -> f64 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.used_bytes as f64) / (self.total_bytes as f64)
        }
    }

    pub fn inode_used_ratio(&self) -> f64 {
        if self.total_inodes == 0 {
            0.0
        } else {
            let used = self.total_inodes.saturating_sub(self.free_inodes);
            (used as f64) / (self.total_inodes as f64)
        }
    }
}

/// Query the kernel for volume mount metadata and statvfs partition metrics
pub fn get_volume_info(path: &Path) -> Option<VolumeInfo> {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let c_path = CString::new(canonical.to_str()?).ok()?;

    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    let res = unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) };
    if res != 0 {
        return None;
    }

    let fragment_size = if stat.f_frsize > 0 { stat.f_frsize as u64 } else { stat.f_bsize as u64 };
    let total_bytes = (stat.f_blocks as u64) * fragment_size;
    let free_bytes = (stat.f_bfree as u64) * fragment_size;
    let available_bytes = (stat.f_bavail as u64) * fragment_size;
    let used_bytes = total_bytes.saturating_sub(free_bytes);

    let total_inodes = stat.f_files as u64;
    let free_inodes = stat.f_ffree as u64;

    let (device, mount_point, fs_type) = find_mount_entry(&canonical)
        .unwrap_or_else(|| ("storage".into(), "/".into(), "posix".into()));

    Some(VolumeInfo {
        device,
        mount_point,
        fs_type,
        total_bytes,
        available_bytes,
        used_bytes,
        total_inodes,
        free_inodes,
    })
}

/// Parse /proc/mounts to find the best matching mountpoint for a given path
fn find_mount_entry(target_path: &Path) -> Option<(String, String, String)> {
    let content = fs::read_to_string("/proc/mounts").ok()?;
    let target_str = target_path.to_str()?;

    let mut best_match: Option<(String, String, String, usize)> = None;

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }

        let device = parts[0];
        let mount_point = parts[1];
        let fs_type = parts[2];

        // Skip virtual or pseudo filesystems
        if fs_type == "tmpfs" || fs_type == "devtmpfs" || fs_type == "proc" || fs_type == "sysfs" {
            continue;
        }

        if target_str.starts_with(mount_point) {
            let len = mount_point.len();
            if let Some((_, _, _, best_len)) = &best_match {
                if len > *best_len {
                    best_match = Some((device.into(), mount_point.into(), fs_type.into(), len));
                }
            } else {
                best_match = Some((device.into(), mount_point.into(), fs_type.into(), len));
            }
        }
    }

    best_match.map(|(d, m, f, _)| (d, m, f))
}
