//! Temp-file helpers shared by unit tests.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// A uniquely named path under the OS temp dir, deleted when dropped so a
/// failing assertion leaves nothing behind.
pub struct Scratch(PathBuf);

impl Scratch {
    /// `name` ends the file name, so it carries the extension a test needs.
    pub fn new(name: &str) -> Self {
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        Self(std::env::temp_dir().join(format!("qobuz-dl-test-{pid}-{n}-{name}")))
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// Write the smallest stream lofty accepts as FLAC: the marker and a zeroed,
/// last STREAMINFO block.
pub fn write_minimal_flac(path: &Path) {
    let mut bytes = b"fLaC".to_vec();
    bytes.extend_from_slice(&[0x80, 0x00, 0x00, 0x22]);
    bytes.extend_from_slice(&[0; 0x22]);
    std::fs::write(path, bytes).unwrap();
}

/// Write two silent MPEG-1 Layer III frames (128 kbps, 44.1 kHz): enough for
/// lofty to recognize the file as MP3.
pub fn write_minimal_mp3(path: &Path) {
    const FRAME_LEN: usize = 417;
    let mut frame = vec![0xFF, 0xFB, 0x90, 0x00];
    frame.resize(FRAME_LEN, 0);
    std::fs::write(path, frame.repeat(2)).unwrap();
}
