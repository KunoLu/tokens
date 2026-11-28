use super::{
    append_path_suffix, cache_shard_dir, ensure_cache_dir, metadata_signature, sample_offsets,
    warn_cache_failure_once, CachedPath, FingerprintStatus, SourceFingerprint, SourceMessageCache,
    CACHE_FORMAT_VERSION, FINGERPRINT_SAMPLE_BYTES, MAX_CACHE_SHARD_BYTES,
};
use crate::sessions::qoder::{
    begin_qoder_ide_read, scan_qoder_ide_db, scan_qoder_ide_snapshot, with_qoder_ide_connection,
    QoderObservation, QoderSource, QODER_PARSER_BASE_VERSION,
};
use bincode::Options;
use rusqlite::{ffi, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BinaryHeap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

// The local layout also embeds the shared fingerprint and message layouts.
const VERSION: [u32; 3] = [2, QODER_PARSER_BASE_VERSION, CACHE_FORMAT_VERSION];
const MAX_HEADER_BYTES: u64 = 128 * 1024;
const MAX_SWEEP_ENTRIES: usize = 256;
const ENTRY_PREFIX: &str = "observations-";
const ENTRY_SUFFIX: &str = ".bin";
const ENTRY_NAME_LEN: usize = ENTRY_PREFIX.len() + 64 + ENTRY_SUFFIX.len();

#[derive(PartialEq, Eq, Serialize, Deserialize)]
enum ObservationFingerprint {
    Jsonl(SourceFingerprint),
    Rollback(RollbackFingerprint),
}

#[derive(PartialEq, Eq, Serialize, Deserialize)]
struct RollbackFingerprint {
    size: u64,
    modified_ns: u64,
    samples: [u8; 32],
}

struct Entry {
    path: PathBuf,
    source_path: CachedPath,
}

pub(crate) struct QoderSourceCache {
    root: PathBuf,
    writable: [OnceLock<bool>; 2],
}

impl SourceMessageCache {
    pub(crate) fn qoder_cache(&self) -> Option<QoderSourceCache> {
        if !self.persistent {
            return None;
        }
        Some(QoderSourceCache {
            root: cache_shard_dir()?,
            writable: Default::default(),
        })
    }
}

impl QoderSourceCache {
    pub(crate) fn prune_orphans(&self) {
        if !is_real_dir(&self.root) {
            return;
        }
        for source in [QoderSource::International, QoderSource::China] {
            let dir = self.root.join(source.as_str());
            if !is_real_dir(&dir) {
                continue;
            }
            let lock_path = dir.join("observations.lock");
            if !is_regular_file(&lock_path) {
                continue;
            }
            let Ok(mut lock) = OpenOptions::new().read(true).write(true).open(lock_path) else {
                continue;
            };
            if fs2::FileExt::try_lock_exclusive(&lock).is_err() {
                continue;
            }
            let Ok(entries) = dir.read_dir() else {
                continue;
            };
            // Directory order is unspecified. Keep the next page by name, not
            // by offset, so removals and different enumeration orders cannot
            // skip a shard. Each pass lists names but opens at most one page.
            // The publishing lock also stores the last name across processes.
            let mut cursor = [0; ENTRY_NAME_LEN];
            let has_cursor = lock.read_exact(&mut cursor).is_ok();
            let after = if has_cursor { cursor.as_slice() } else { &[] };
            let mut page = BinaryHeap::with_capacity(MAX_SWEEP_ENTRIES);
            for entry in entries.flatten() {
                let Ok(name) = entry.file_name().into_string() else {
                    continue;
                };
                let bytes = name.as_bytes();
                if bytes <= after
                    || bytes.len() != ENTRY_NAME_LEN
                    || !bytes.starts_with(ENTRY_PREFIX.as_bytes())
                    || !bytes.ends_with(ENTRY_SUFFIX.as_bytes())
                {
                    continue;
                }
                if page.len() < MAX_SWEEP_ENTRIES {
                    page.push(name);
                } else if let Some(mut last) = page.peek_mut() {
                    if name < *last {
                        *last = name;
                    }
                }
            }
            for name in &page {
                let path = dir.join(name);
                if !is_regular_file(&path) {
                    continue;
                }
                if let Ok(Some((_, _, source))) = open_header(&path) {
                    if entry_name(&source).as_str() != name {
                        continue;
                    }
                    let source = source.to_path_buf();
                    // Follow source links, but keep shards on permission and other errors.
                    if source.is_absolute()
                        && source
                            .metadata()
                            .is_err_and(|error| error.kind() == io::ErrorKind::NotFound)
                    {
                        let _ = fs::remove_file(&path);
                    }
                }
            }
            if let Some(last) = page.peek().filter(|_| page.len() == MAX_SWEEP_ENTRIES) {
                let _ = lock.rewind().and_then(|()| lock.write_all(last.as_bytes()));
            } else if has_cursor {
                let _ = lock.set_len(0);
            }
        }
    }

    pub(crate) fn load_or_scan_ide(&self, path: &Path, source: QoderSource) -> Vec<QoderObservation> {
        let Ok(resolved) = path.canonicalize() else {
            return scan_qoder_ide_db(path, source).messages;
        };
        let path = resolved.as_path();
        let entry = self.entry(path, source);
        with_qoder_ide_connection(path, |mut conn| {
            if let Some((reader, ObservationFingerprint::Rollback(cached))) = entry.open() {
                if rollback_fingerprint(&conn, path).as_ref() == Some(&cached) {
                    if let Some(observations) = entry.read(reader) {
                        return Ok(observations);
                    }
                }
            }

            // A rollback read lock pins the file sampled here. WAL reads use
            // SQLite's own snapshot, but the bundled SQLite exposes no portable
            // snapshot ID to validate a persisted hit across connections.
            let tx = begin_qoder_ide_read(&mut conn)?;
            let fingerprint = rollback_fingerprint(&tx, path);
            let scan = scan_qoder_ide_snapshot(&tx, path, source);
            drop(tx);
            if scan.complete {
                if let Some(fingerprint) = fingerprint.map(ObservationFingerprint::Rollback) {
                    self.save(source, &entry, &fingerprint, &scan.messages, || {
                        rollback_fingerprint(&conn, path)
                            .map(ObservationFingerprint::Rollback)
                            .as_ref()
                            == Some(&fingerprint)
                    });
                }
            }
            Ok(scan.messages)
        })
        .unwrap_or_default()
    }

    pub(crate) fn load_or_parse(
        &self,
        path: &Path,
        source: QoderSource,
        parse: impl FnOnce(&Path, QoderSource) -> (Vec<QoderObservation>, bool),
    ) -> Vec<QoderObservation> {
        debug_assert!(path.is_absolute());
        let entry = self.entry(path, source);
        let cached = match entry.open() {
            Some((reader, ObservationFingerprint::Jsonl(fingerprint))) => {
                Some((reader, fingerprint))
            }
            _ => None,
        };
        let check = |previous: Option<&SourceFingerprint>| {
            SourceFingerprint::check_qoder_path(path, previous)
        };
        let status = check(cached.as_ref().map(|(_, fingerprint)| fingerprint));
        let fingerprint = match (status, cached) {
            (Some(FingerprintStatus::Unchanged), Some((reader, fingerprint))) => {
                if let Some(observations) = entry.read(reader) {
                    return observations;
                }
                fingerprint
            }
            (Some(FingerprintStatus::Changed(fingerprint)), _) => fingerprint,
            _ => return parse(path, source).0,
        };

        let (observations, complete) = parse(path, source);
        if complete && !observations.is_empty() {
            let fingerprint = ObservationFingerprint::Jsonl(fingerprint);
            self.save(source, &entry, &fingerprint, &observations, || {
                let ObservationFingerprint::Jsonl(fingerprint) = &fingerprint else {
                    return false;
                };
                matches!(check(Some(fingerprint)), Some(FingerprintStatus::Unchanged))
            });
        }
        observations
    }

    fn entry(&self, path: &Path, source: QoderSource) -> Entry {
        let source_path = CachedPath::from_path(path);
        Entry {
            path: self.root.join(source.as_str()).join(entry_name(&source_path)),
            source_path,
        }
    }

    fn save(
        &self,
        source: QoderSource,
        entry: &Entry,
        fingerprint: &ObservationFingerprint,
        observations: &[QoderObservation],
        still_current: impl FnOnce() -> bool,
    ) {
        let dir = entry.path.parent().expect("entry in the Qoder cache directory");
        let writable = self.writable[usize::from(source == QoderSource::China)].get_or_init(|| {
            match ensure_cache_dir(&self.root).and_then(|()| ensure_cache_dir(dir)) {
                Ok(()) => true,
                Err(error) => {
                    warn_cache_failure_once(
                        "Qoder observation cache directory is unavailable",
                        dir,
                        &error,
                    );
                    false
                }
            }
        });
        if *writable {
            if let Err(error) = save_entry(dir, entry, fingerprint, observations, still_current) {
                warn_cache_failure_once(
                    "Qoder observation cache could not be saved",
                    &entry.path,
                    &error,
                );
            }
        }
    }
}

impl Entry {
    fn open(&self) -> Option<(BufReader<File>, ObservationFingerprint)> {
        let cached: io::Result<_> = (|| {
            let Some((mut reader, version, source)) = open_header(&self.path)? else {
                return Ok(None);
            };
            if version != VERSION || source != self.source_path {
                return Ok(None);
            }
            let fingerprint = options().deserialize_from(&mut reader).map_err(io::Error::other)?;
            Ok(Some((reader, fingerprint)))
        })();
        match cached {
            Ok(cached) => cached,
            Err(error) => {
                warn_cache_failure_once("Qoder observation cache is unreadable", &self.path, &error);
                None
            }
        }
    }

    fn read(&self, reader: BufReader<File>) -> Option<Vec<QoderObservation>> {
        match read_observations(reader) {
            Ok(observations) => Some(observations),
            Err(error) => {
                warn_cache_failure_once("Qoder observation cache is invalid", &self.path, &error);
                None
            }
        }
    }
}

fn entry_name(source_path: &CachedPath) -> String {
    let mut digest = Sha256::new();
    source_path.update_digest(&mut digest);
    format!("{ENTRY_PREFIX}{:x}{ENTRY_SUFFIX}", digest.finalize())
}

fn is_real_dir(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_dir())
}

fn is_regular_file(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_file())
}

fn is_missing(path: &Path) -> bool {
    fs::symlink_metadata(path).is_err_and(|error| error.kind() == io::ErrorKind::NotFound)
}

fn no_sqlite_sidecars(path: &Path) -> bool {
    ["-wal", "-journal"]
        .into_iter()
        .all(|suffix| is_missing(&append_path_suffix(path, suffix)))
}

fn rollback_fingerprint(conn: &Connection, path: &Path) -> Option<RollbackFingerprint> {
    if !no_sqlite_sidecars(path) {
        return None;
    }
    let (size, modified_ns) = metadata_signature(path).ok()?;
    if size < 100 {
        return None;
    }

    // Reuse SQLite's file handle: closing a separate descriptor on Unix can
    // drop SQLite's POSIX locks even while a read transaction is active.
    let mut file = std::ptr::null_mut::<ffi::sqlite3_file>();
    // SAFETY: FILE_POINTER receives a writable pointer to sqlite3_file*.
    // SQLite owns that file and its methods for the lifetime of this connection.
    let read = unsafe {
        let status = ffi::sqlite3_file_control(
            conn.handle(),
            c"main".as_ptr(),
            ffi::SQLITE_FCNTL_FILE_POINTER,
            (&mut file as *mut *mut ffi::sqlite3_file).cast(),
        );
        if status != ffi::SQLITE_OK {
            return None;
        }
        file.as_ref()?.pMethods.as_ref()?.xRead?
    };
    let mut digest = Sha256::new();
    let mut sample = [0; FINGERPRINT_SAMPLE_BYTES];
    for (offset, len) in sqlite_sample_offsets(size) {
        // SAFETY: The connection still owns `file`; `sample` has room for len bytes.
        let status = unsafe { read(file, sample.as_mut_ptr().cast(), len as i32, offset as i64) };
        if status != ffi::SQLITE_OK {
            return None;
        }
        // Bytes 18 and 19 identify the SQLite read/write format. A WAL-mode
        // database remains ineligible even if its sidecar is absent.
        if offset == 0 && (&sample[..16] != b"SQLite format 3\0" || sample[18..20] != [1, 1]) {
            return None;
        }
        digest.update(&sample[..len]);
    }

    let mut moved = 0_i32;
    // SAFETY: HAS_MOVED receives a writable pointer to an int.
    let status = unsafe {
        ffi::sqlite3_file_control(
            conn.handle(),
            c"main".as_ptr(),
            ffi::SQLITE_FCNTL_HAS_MOVED,
            (&mut moved as *mut i32).cast(),
        )
    };
    if (status != ffi::SQLITE_OK && status != ffi::SQLITE_NOTFOUND)
        || moved != 0
        || metadata_signature(path).ok() != Some((size, modified_ns))
        || !no_sqlite_sidecars(path)
    {
        return None;
    }
    Some(RollbackFingerprint {
        size,
        modified_ns,
        samples: digest.finalize().into(),
    })
}

const SQLITE_LOCK_START: u64 = 1 << 30;
const SQLITE_LOCK_END: u64 = SQLITE_LOCK_START + 512;

fn sqlite_sample_offsets(size: u64) -> impl Iterator<Item = (u64, usize)> {
    // Windows byte-range locks can reject reads overlapping these reserved bytes.
    sample_offsets(size).into_iter().filter(|&(offset, len)| {
        offset + len as u64 <= SQLITE_LOCK_START || offset >= SQLITE_LOCK_END
    })
}

fn options() -> impl Options {
    bincode::options().with_limit(MAX_CACHE_SHARD_BYTES)
}

type OpenHeader = (BufReader<File>, [u32; 3], CachedPath);

fn open_header(path: &Path) -> io::Result<Option<OpenHeader>> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    if file.metadata()?.len() > MAX_CACHE_SHARD_BYTES {
        return Err(io::Error::other("Qoder observation entry exceeds size limit"));
    }
    let mut reader = BufReader::new(file);
    let header_options = || options().with_limit(MAX_HEADER_BYTES);
    let version: [u32; 3] = header_options().deserialize_from(&mut reader).map_err(io::Error::other)?;
    if version[0] != VERSION[0] {
        // V1 did not record the primary path. Do not guess ownership or liveness.
        return Ok(None);
    }
    let source_path = header_options().deserialize_from(&mut reader).map_err(io::Error::other)?;
    Ok(Some((reader, version, source_path)))
}

fn read_observations(mut reader: BufReader<File>) -> io::Result<Vec<QoderObservation>> {
    let observations = options().deserialize_from(&mut reader).map_err(io::Error::other)?;
    if !reader.fill_buf()?.is_empty() {
        return Err(io::Error::other("trailing data in Qoder observation entry"));
    }
    Ok(observations)
}

fn save_entry(
    dir: &Path,
    entry: &Entry,
    fingerprint: &ObservationFingerprint,
    observations: &[QoderObservation],
    still_current: impl FnOnce() -> bool,
) -> io::Result<()> {
    static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);
    let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
    let tmp = entry.path.with_extension(format!("{}.{id}.tmp", std::process::id()));
    let file = OpenOptions::new().write(true).create_new(true).open(&tmp)?;
    let result = (|| {
        let mut writer = BufWriter::new(file);
        options()
            .serialize_into(&mut writer, &(VERSION, &entry.source_path, fingerprint, observations))
            .map_err(io::Error::other)?;
        writer.flush()?;
        writer.get_ref().sync_all()?;
        drop(writer);

        // Publish the whole observation set, never a subset. Verify the source
        // under the publishing lock so an older scan cannot replace a newer hit.
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(dir.join("observations.lock"))?;
        fs2::FileExt::lock_exclusive(&lock)?;
        if still_current() {
            crate::fs_atomic::replace_file(&tmp, &entry.path)?;
        }
        Ok(())
    })();
    let _ = fs::remove_file(&tmp);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqlite_fingerprint_samples_skip_the_windows_lock_bytes() {
        let size = 2 * SQLITE_LOCK_START;
        assert!(sample_offsets(size).into_iter().any(|(offset, len)| {
            offset < SQLITE_LOCK_END && offset + len as u64 > SQLITE_LOCK_START
        }));
        let samples: Vec<_> = sqlite_sample_offsets(size).collect();
        assert_eq!(samples.first().map(|&(offset, _)| offset), Some(0));
        assert!(samples.iter().any(|&(offset, _)| offset >= SQLITE_LOCK_END));
        assert!(samples.iter().all(|&(offset, len)| {
            offset + len as u64 <= SQLITE_LOCK_START || offset >= SQLITE_LOCK_END
        }));
    }
}
