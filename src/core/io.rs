//! Reading concept types from disk. Kept apart from the types themselves so
//! that `core` stays free of IO concerns beyond this file.

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::Path;

use process_mining::core::process_models::case_centric::petri_net::io::PetriNetIOError;
use process_mining::core::process_models::case_centric::petri_net::PetriNet;
use process_mining::Importable;

use super::Lpm;

#[derive(Debug, thiserror::Error)]
pub enum IoError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("zip: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("pnml `{name}`: {source}")]
    Pnml {
        name: String,
        #[source]
        source: PetriNetIOError,
    },
    #[error("two pnml entries named `{0}`")]
    DuplicateName(String),
}

/// Read every `*.pnml` entry of a zip archive as one LPM.
///
/// Each LPM's name is the entry's file stem, and LPMs are returned in archive
/// order. All other entries (including any CSV) and directories are ignored.
pub fn read_lpm_zip(path: impl AsRef<Path>) -> Result<Vec<Lpm>, IoError> {
    read_lpm_zip_from(BufReader::new(File::open(path)?))
}

/// Like [`read_lpm_zip`], for any seekable reader.
pub fn read_lpm_zip_from<R: Read + Seek>(reader: R) -> Result<Vec<Lpm>, IoError> {
    let mut archive = zip::ZipArchive::new(reader)?;
    let mut lpms = Vec::new();
    let mut names = HashSet::new();

    for i in 0..archive.len() {
        let entry = archive.by_index(i)?;
        if entry.is_dir() {
            continue;
        }
        let entry_name = entry.name().to_owned();
        let path = Path::new(&entry_name);
        let is_pnml = path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("pnml"));
        if !is_pnml {
            continue;
        }
        let net = PetriNet::import_from_reader(entry, "pnml")
            .map_err(|source| IoError::Pnml { name: entry_name.clone(), source })?;
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&entry_name)
            .to_owned();
        if !names.insert(name.clone()) {
            return Err(IoError::DuplicateName(name));
        }
        lpms.push(Lpm { name, net });
    }
    Ok(lpms)
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Write};

    use process_mining::Exportable;
    use zip::write::SimpleFileOptions;

    use super::*;

    fn pnml(label: &str) -> Vec<u8> {
        let mut net = PetriNet::new();
        net.add_transition(Some(label.to_owned()), None);
        let mut out = Vec::new();
        net.export_to_writer(&mut out, "pnml").unwrap();
        out
    }

    fn archive(entries: &[(&str, &[u8])]) -> Cursor<Vec<u8>> {
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for (name, bytes) in entries {
            zip.start_file(*name, SimpleFileOptions::default()).unwrap();
            zip.write_all(bytes).unwrap();
        }
        let mut cursor = zip.finish().unwrap();
        cursor.set_position(0);
        cursor
    }

    #[test]
    fn reads_pnmls_and_ignores_other_entries() {
        let lpms = read_lpm_zip_from(archive(&[
            ("a.pnml", &pnml("x")),
            ("info.csv", b"model,support\na,0.9\n"),
            ("dir/b.PNML", &pnml("y")),
        ]))
        .unwrap();
        let names: Vec<_> = lpms.iter().map(|l| l.name.as_str()).collect();
        assert_eq!(names, ["a", "b"]);
        assert_eq!(lpms[1].net.transitions.len(), 1);
    }

    #[test]
    fn rejects_duplicate_names() {
        let result = read_lpm_zip_from(archive(&[("a.pnml", &pnml("x")), ("dir/a.pnml", &pnml("y"))]));
        assert!(matches!(result, Err(IoError::DuplicateName(n)) if n == "a"));
    }

    #[test]
    fn rejects_non_zip_input() {
        let result = read_lpm_zip_from(Cursor::new(b"not a zip".to_vec()));
        assert!(matches!(result, Err(IoError::Zip(_))));
    }
}