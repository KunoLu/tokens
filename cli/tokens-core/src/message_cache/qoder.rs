use super::{
    cache_shard_dir, ensure_cache_dir, warn_cache_failure_once, CachedPath, FingerprintStatus,
    SourceFingerprint, SourceMessageCache, CACHE_FORMAT_VERSION, MAX_CACHE_SHARD_BYTES,
};
use crate::sessions::qoder::{QoderObservation, QoderSource, QODER_PARSER_BASE_VERSION};
use bincode::Options;
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

// The local layout also embeds the shared fingerprint and message layouts.
const VERSION: [u32; 3] = [1, QODER_PARSER_BASE_VERSION, CACHE_FORMAT_VERSION];

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
    pub(crate) fn load_or_parse(
        &self,
        path: &Path,
        source: QoderSource,
        is_ide: bool,
        parse: impl FnOnce() -> (Vec<QoderObservation>, bool),
    ) -> Vec<QoderObservation> {
        let dir = self.root.join(source.as_str());
        let mut digest = Sha256::new();
        CachedPath::from_path(path).update_digest(&mut digest);
        let entry = dir.join(format!("observations-{:x}.bin", digest.finalize()));
        let cached = match open_entry(&entry) {
            Ok(cached) => cached,
            Err(error) => {
                warn_cache_failure_once("Qoder observation cache is unreadable", &entry, &error);
                None
            }
        };
        let check = |previous: Option<&SourceFingerprint>| {
            if is_ide {
                SourceFingerprint::check_sqlite_path(path, previous)
            } else {
                SourceFingerprint::check_qoder_path(path, previous)
            }
        };
        let status = check(cached.as_ref().map(|(_, fingerprint)| fingerprint));
        let fingerprint = match (status, cached) {
            (Some(FingerprintStatus::Unchanged), Some((reader, fingerprint))) => {
                match read_observations(reader) {
                    Ok(observations) => return observations,
                    Err(error) => {
                        warn_cache_failure_once("Qoder observation cache is invalid", &entry, &error);
                    }
                }
                fingerprint
            }
            (Some(FingerprintStatus::Changed(fingerprint)), _) => fingerprint,
            _ => return parse().0,
        };

        let (observations, complete) = parse();
        if complete && (is_ide || !observations.is_empty()) {
            let writable =
                self.writable[usize::from(source == QoderSource::China)].get_or_init(|| {
                    match ensure_cache_dir(&self.root).and_then(|()| ensure_cache_dir(&dir)) {
                        Ok(()) => true,
                        Err(error) => {
                            warn_cache_failure_once(
                                "Qoder observation cache directory is unavailable",
                                &dir,
                                &error,
                            );
                            false
                        }
                    }
                });
            if *writable {
                let result = save_entry(&dir, &entry, &fingerprint, &observations, || {
                    matches!(check(Some(&fingerprint)), Some(FingerprintStatus::Unchanged))
                });
                if let Err(error) = result {
                    warn_cache_failure_once(
                        "Qoder observation cache could not be saved",
                        &entry,
                        &error,
                    );
                }
            }
        }
        observations
    }
}

fn options() -> impl Options {
    bincode::options().with_limit(MAX_CACHE_SHARD_BYTES)
}

fn open_entry(path: &Path) -> io::Result<Option<(BufReader<File>, SourceFingerprint)>> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    if file.metadata()?.len() > MAX_CACHE_SHARD_BYTES {
        return Err(io::Error::other("Qoder observation entry exceeds size limit"));
    }
    let mut reader = BufReader::new(file);
    let version: [u32; 3] = options().deserialize_from(&mut reader).map_err(io::Error::other)?;
    if version != VERSION {
        return Ok(None);
    }
    let fingerprint = options().deserialize_from(&mut reader).map_err(io::Error::other)?;
    Ok(Some((reader, fingerprint)))
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
    path: &Path,
    fingerprint: &SourceFingerprint,
    observations: &[QoderObservation],
    still_current: impl FnOnce() -> bool,
) -> io::Result<()> {
    static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);
    let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
    let tmp = path.with_extension(format!("{}.{id}.tmp", std::process::id()));
    let file = OpenOptions::new().write(true).create_new(true).open(&tmp)?;
    let result = (|| {
        let mut writer = BufWriter::new(file);
        options()
            .serialize_into(&mut writer, &(VERSION, fingerprint, observations))
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
            crate::fs_atomic::replace_file(&tmp, path)?;
        }
        Ok(())
    })();
    let _ = fs::remove_file(&tmp);
    result
}
