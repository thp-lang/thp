//! Content-addressed persistent cache for verified THP bytecode.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use thp_bytecode::{Program, decode, encode};

pub const CACHE_FORMAT_VERSION: u16 = 1;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CacheKey(String);

impl CacheKey {
    pub fn calculate(source: &[u8], compiler_version: &str, configuration: &[u8]) -> Self {
        let mut hash = blake3::Hasher::new();
        hash.update(b"THP opcache");
        hash.update(&CACHE_FORMAT_VERSION.to_le_bytes());
        hash.update(&thp_bytecode::BYTECODE_SCHEMA_VERSION.to_le_bytes());
        hash.update(compiler_version.as_bytes());
        hash.update(std::env::consts::ARCH.as_bytes());
        hash.update(std::env::consts::OS.as_bytes());
        hash.update(configuration);
        hash.update(source);
        Self(hash.finalize().to_hex().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Restores a key from a BLAKE3-style hexadecimal digest.
    ///
    /// # Errors
    ///
    /// Returns invalid-data for a malformed digest.
    pub fn from_digest(value: impl Into<String>) -> io::Result<Self> {
        let value = value.into();
        if value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            Ok(Self(value))
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "cache key is not a 256-bit hexadecimal digest",
            ))
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArtifactKind {
    Interface,
    Object,
}

impl ArtifactKind {
    const fn extension(self) -> &'static str {
        match self {
            Self::Interface => "thpi",
            Self::Object => "thpo",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrozenManifest {
    pub manifest_version: u16,
    pub compiler_version: String,
    pub project_fingerprint: String,
    pub entry_id: String,
    pub program_key: CacheKey,
    pub interface_hashes: Vec<String>,
    pub object_hashes: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CacheStatus {
    Hit,
    Miss,
    Corrupt,
}

#[derive(Debug)]
pub struct Lookup {
    pub status: CacheStatus,
    pub program: Option<Program>,
}

#[derive(Clone, Debug)]
pub struct Store {
    directory: PathBuf,
}

impl Store {
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
        }
    }

    pub fn directory(&self) -> &Path {
        &self.directory
    }

    /// Loads and verifies a cached bytecode artifact.
    ///
    /// Corrupt artifacts are reported as cache misses and never executed.
    ///
    /// # Errors
    ///
    /// Returns filesystem read errors other than a missing cache entry.
    pub fn lookup(&self, key: &CacheKey) -> io::Result<Lookup> {
        let path = self.entry_path(key);
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(Lookup {
                    status: CacheStatus::Miss,
                    program: None,
                });
            }
            Err(error) => return Err(error),
        };
        match decode(&bytes) {
            Ok(program) => Ok(Lookup {
                status: CacheStatus::Hit,
                program: Some(program),
            }),
            Err(_) => Ok(Lookup {
                status: CacheStatus::Corrupt,
                program: None,
            }),
        }
    }

    /// Atomically publishes an immutable cache entry.
    ///
    /// Concurrent writers are harmless: the first complete artifact wins and
    /// every artifact for a key is byte-for-byte equivalent.
    ///
    /// # Errors
    ///
    /// Returns directory creation, write, flush, or persistence failures.
    pub fn store(&self, key: &CacheKey, program: &Program) -> io::Result<()> {
        fs::create_dir_all(&self.directory)?;
        let mut temporary = tempfile::NamedTempFile::new_in(&self.directory)?;
        temporary.write_all(&encode(program))?;
        temporary.as_file().sync_all()?;
        temporary
            .persist(self.entry_path(key))
            .map(|_| ())
            .map_err(|error| error.error)
    }

    /// Atomically stores a checksummed module interface or object.
    ///
    /// # Errors
    ///
    /// Returns filesystem publication failures.
    pub fn store_artifact(
        &self,
        kind: ArtifactKind,
        key: &CacheKey,
        payload: &[u8],
    ) -> io::Result<()> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"THPA");
        bytes.extend_from_slice(&CACHE_FORMAT_VERSION.to_le_bytes());
        bytes.push(match kind {
            ArtifactKind::Interface => 1,
            ArtifactKind::Object => 2,
        });
        bytes.extend_from_slice(blake3::hash(payload).as_bytes());
        bytes.extend_from_slice(payload);
        self.publish(&self.artifact_path(key, kind.extension()), &bytes)
    }

    /// Loads and validates a module interface or object.
    ///
    /// # Errors
    ///
    /// Returns filesystem failures or invalid-data for corrupt artifacts.
    pub fn lookup_artifact(
        &self,
        kind: ArtifactKind,
        key: &CacheKey,
    ) -> io::Result<Option<Vec<u8>>> {
        let path = self.artifact_path(key, kind.extension());
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };
        if bytes.len() < 39
            || &bytes[..4] != b"THPA"
            || u16::from_le_bytes([bytes[4], bytes[5]]) != CACHE_FORMAT_VERSION
            || bytes[6]
                != match kind {
                    ArtifactKind::Interface => 1,
                    ArtifactKind::Object => 2,
                }
            || blake3::hash(&bytes[39..]).as_bytes() != &bytes[7..39]
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "corrupt or incompatible module artifact",
            ));
        }
        Ok(Some(bytes[39..].to_vec()))
    }

    /// Atomically publishes a checksummed frozen manifest.
    ///
    /// # Errors
    ///
    /// Returns filesystem publication failures.
    pub fn store_manifest(&self, key: &CacheKey, manifest: &FrozenManifest) -> io::Result<()> {
        self.publish(&self.artifact_path(key, "thpm"), &encode_manifest(manifest))
    }

    /// Loads and validates a frozen manifest.
    ///
    /// # Errors
    ///
    /// Returns filesystem, corruption, or incompatible-version failures.
    pub fn load_manifest(&self, key: &CacheKey) -> io::Result<FrozenManifest> {
        let path = self.artifact_path(key, "thpm");
        let bytes = fs::read(path)?;
        decode_manifest(&bytes)
    }

    /// Evicts the oldest entries until the cache fits within `maximum_bytes`.
    ///
    /// # Errors
    ///
    /// Returns errors while listing, inspecting, or removing cache entries.
    pub fn prune(&self, maximum_bytes: u64) -> io::Result<PruneResult> {
        let entries = match fs::read_dir(&self.directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(PruneResult::default());
            }
            Err(error) => return Err(error),
        };
        let mut entries = entries
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let path = entry.path();
                if !matches!(
                    path.extension().and_then(|value| value.to_str()),
                    Some("thpi" | "thpo" | "thpbc" | "thpm")
                ) {
                    return None;
                }
                let metadata = entry.metadata().ok()?;
                let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                Some((path, metadata.len(), modified))
            })
            .collect::<Vec<_>>();
        let mut bytes = entries.iter().map(|(_, bytes, _)| bytes).sum::<u64>();
        entries.sort_by_key(|(_, _, modified)| *modified);
        let mut result = PruneResult {
            remaining_bytes: bytes,
            ..PruneResult::default()
        };
        for (path, entry_bytes, _) in entries {
            if bytes <= maximum_bytes {
                break;
            }
            fs::remove_file(path)?;
            bytes = bytes.saturating_sub(entry_bytes);
            result.removed_entries += 1;
            result.removed_bytes += entry_bytes;
        }
        result.remaining_bytes = bytes;
        Ok(result)
    }

    fn entry_path(&self, key: &CacheKey) -> PathBuf {
        self.directory.join(format!("{}.thpbc", key.as_str()))
    }

    fn artifact_path(&self, key: &CacheKey, extension: &str) -> PathBuf {
        self.directory.join(format!("{}.{extension}", key.as_str()))
    }

    fn publish(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        fs::create_dir_all(&self.directory)?;
        let mut temporary = tempfile::NamedTempFile::new_in(&self.directory)?;
        temporary.write_all(bytes)?;
        temporary.as_file().sync_all()?;
        temporary
            .persist(path)
            .map(|_| ())
            .map_err(|error| error.error)
    }
}

fn encode_manifest(manifest: &FrozenManifest) -> Vec<u8> {
    let mut encoded = format!(
        "THP-MANIFEST\nversion={}\ncompiler={}\nproject={}\nentry={}\nprogram={}\n",
        manifest.manifest_version,
        manifest.compiler_version,
        manifest.project_fingerprint,
        manifest.entry_id,
        manifest.program_key.as_str(),
    );
    for hash in &manifest.interface_hashes {
        encoded.push_str("interface=");
        encoded.push_str(hash);
        encoded.push('\n');
    }
    for hash in &manifest.object_hashes {
        encoded.push_str("object=");
        encoded.push_str(hash);
        encoded.push('\n');
    }
    let checksum = blake3::hash(encoded.as_bytes()).to_hex();
    encoded.push_str("checksum=");
    encoded.push_str(checksum.as_str());
    encoded.push('\n');
    encoded.into_bytes()
}

fn decode_manifest(bytes: &[u8]) -> io::Result<FrozenManifest> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "manifest is not UTF-8"))?;
    let (payload, checksum_line) = text.rsplit_once("checksum=").ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "manifest checksum is missing")
    })?;
    let checksum = checksum_line.trim_end();
    if blake3::hash(payload.as_bytes()).to_hex().as_str() != checksum {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "manifest checksum mismatch",
        ));
    }
    let mut lines = payload.lines();
    if lines.next() != Some("THP-MANIFEST") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid manifest header",
        ));
    }
    let mut version = None;
    let mut compiler = None;
    let mut project = None;
    let mut entry = None;
    let mut program = None;
    let mut interfaces = Vec::new();
    let mut objects = Vec::new();
    for line in lines {
        let (key, value) = line.split_once('=').ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "malformed manifest record")
        })?;
        match key {
            "version" => {
                version = Some(value.parse::<u16>().map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "invalid manifest version")
                })?);
            }
            "compiler" => compiler = Some(value.to_owned()),
            "project" => project = Some(value.to_owned()),
            "entry" => entry = Some(value.to_owned()),
            "program" => program = Some(CacheKey::from_digest(value)?),
            "interface" => interfaces.push(value.to_owned()),
            "object" => objects.push(value.to_owned()),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "unknown manifest record",
                ));
            }
        }
    }
    let manifest = FrozenManifest {
        manifest_version: version
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing version"))?,
        compiler_version: compiler
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing compiler"))?,
        project_fingerprint: project
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing project"))?,
        entry_id: entry
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing entry"))?,
        program_key: program
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing program"))?,
        interface_hashes: interfaces,
        object_hashes: objects,
    };
    if manifest.manifest_version != CACHE_FORMAT_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsupported frozen manifest version",
        ));
    }
    Ok(manifest)
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PruneResult {
    pub removed_entries: usize,
    pub removed_bytes: u64,
    pub remaining_bytes: u64,
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    use thp_bytecode::lower as lower_bytecode;
    use thp_diagnostics::SourceFile;
    use thp_hir::lower as lower_hir;
    use thp_mir::lower as lower_mir;
    use thp_syntax::parse;

    use super::{ArtifactKind, CACHE_FORMAT_VERSION, CacheKey, CacheStatus, FrozenManifest, Store};

    fn program() -> thp_bytecode::Program {
        let source = SourceFile::new("cache.thp", "<?thp\necho \"cached\";");
        let parsed = parse(&source);
        let hir = lower_hir(&parsed.program);
        lower_bytecode(&lower_mir(&hir.module))
    }

    #[test]
    fn cache_round_trip_and_invalidation() {
        let directory = tempdir().unwrap();
        let store = Store::new(directory.path());
        let key = CacheKey::calculate(b"one", "test", b"config");
        assert_eq!(store.lookup(&key).unwrap().status, CacheStatus::Miss);
        store.store(&key, &program()).unwrap();
        assert_eq!(store.lookup(&key).unwrap().status, CacheStatus::Hit);

        let changed = CacheKey::calculate(b"two", "test", b"config");
        assert_ne!(key, changed);
        assert_eq!(store.lookup(&changed).unwrap().status, CacheStatus::Miss);
    }

    #[test]
    fn corrupt_entries_are_not_loaded() {
        let directory = tempdir().unwrap();
        let store = Store::new(directory.path());
        let key = CacheKey::calculate(b"source", "test", b"");
        std::fs::create_dir_all(store.directory()).unwrap();
        std::fs::write(
            store.directory().join(format!("{}.thpbc", key.as_str())),
            b"corrupt",
        )
        .unwrap();
        assert_eq!(store.lookup(&key).unwrap().status, CacheStatus::Corrupt);
        store.store(&key, &program()).unwrap();
        assert_eq!(store.lookup(&key).unwrap().status, CacheStatus::Hit);
    }

    #[test]
    fn eviction_removes_old_entries_to_the_byte_limit() {
        let directory = tempdir().unwrap();
        let store = Store::new(directory.path());
        for source in [b"one".as_slice(), b"two".as_slice()] {
            let key = CacheKey::calculate(source, "test", b"");
            store.store(&key, &program()).unwrap();
        }
        let result = store.prune(0).unwrap();
        assert_eq!(result.removed_entries, 2);
        assert_eq!(result.remaining_bytes, 0);
    }

    #[test]
    fn module_artifacts_and_frozen_manifests_round_trip() {
        let directory = tempdir().unwrap();
        let store = Store::new(directory.path());
        let interface_key = CacheKey::calculate(b"interface", "test", b"");
        store
            .store_artifact(
                ArtifactKind::Interface,
                &interface_key,
                b"canonical interface",
            )
            .unwrap();
        assert_eq!(
            store
                .lookup_artifact(ArtifactKind::Interface, &interface_key)
                .unwrap()
                .unwrap(),
            b"canonical interface"
        );

        let manifest_key = CacheKey::calculate(b"manifest", "test", b"");
        let manifest = FrozenManifest {
            manifest_version: CACHE_FORMAT_VERSION,
            compiler_version: "test".to_owned(),
            project_fingerprint: "project".to_owned(),
            entry_id: "App\\Main".to_owned(),
            program_key: CacheKey::calculate(b"program", "test", b""),
            interface_hashes: vec![interface_key.as_str().to_owned()],
            object_hashes: vec![
                CacheKey::calculate(b"object", "test", b"")
                    .as_str()
                    .to_owned(),
            ],
        };
        store.store_manifest(&manifest_key, &manifest).unwrap();
        assert_eq!(store.load_manifest(&manifest_key).unwrap(), manifest);
    }

    #[test]
    fn corrupt_module_artifacts_are_rejected() {
        let directory = tempdir().unwrap();
        let store = Store::new(directory.path());
        let key = CacheKey::calculate(b"object", "test", b"");
        store
            .store_artifact(ArtifactKind::Object, &key, b"object")
            .unwrap();
        std::fs::write(
            store.directory().join(format!("{}.thpo", key.as_str())),
            b"corrupt",
        )
        .unwrap();
        assert!(store.lookup_artifact(ArtifactKind::Object, &key).is_err());
    }
}
