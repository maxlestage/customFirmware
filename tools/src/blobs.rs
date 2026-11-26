//! `cf blobs` : copie les fichiers propriétaires listés dans `proprietary-files.txt`
//! depuis le système d'origine, et écrit le fragment Make qui les installe.
//!
//! Format de la liste, une entrée par ligne :
//!
//! ```text
//! # un commentaire
//! vendor/lib64/libexemple.so
//! vendor/etc/origine.xml:vendor/etc/destination.xml
//! ```
//!
//! Le premier dossier du chemin est la partition (`vendor`, `odm`, `product`…).

use std::fmt;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

/// Les partitions connues, avec la variable Make de leur dossier d'installation.
const PARTITIONS: &[(&str, &str)] = &[
    ("vendor", "$(TARGET_COPY_OUT_VENDOR)"),
    ("odm", "$(TARGET_COPY_OUT_ODM)"),
    ("product", "$(TARGET_COPY_OUT_PRODUCT)"),
    ("system_ext", "$(TARGET_COPY_OUT_SYSTEM_EXT)"),
    ("system", "$(TARGET_COPY_OUT_SYSTEM)"),
    ("vendor_dlkm", "$(TARGET_COPY_OUT_VENDOR_DLKM)"),
    ("odm_dlkm", "$(TARGET_COPY_OUT_ODM_DLKM)"),
];

/// Un fichier à reprendre : où le lire dans le système d'origine, où l'installer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub src: String,
    pub dst: String,
}

impl Entry {
    /// La destination sur le téléphone, écrite pour Make :
    /// `vendor/lib64/x.so` devient `$(TARGET_COPY_OUT_VENDOR)/lib64/x.so`.
    pub fn install_path(&self) -> String {
        let (partition, rest) = self.dst.split_once('/').expect("validé par parse_list");
        let var = PARTITIONS
            .iter()
            .find(|(name, _)| *name == partition)
            .map(|(_, var)| *var)
            .expect("validé par parse_list");
        format!("{var}/{rest}")
    }
}

/// Une ligne de la liste qui ne peut pas être lue.
#[derive(Debug, PartialEq, Eq)]
pub struct ListError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for ListError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ligne {} : {}", self.line, self.message)
    }
}

/// Lit la liste. Toute ligne invalide est une erreur : mieux vaut s'arrêter que
/// copier un fichier au mauvais endroit.
pub fn parse_list(text: &str) -> Result<Vec<Entry>, ListError> {
    let mut entries: Vec<Entry> = Vec::new();
    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let error = |message: String| ListError {
            line: index + 1,
            message,
        };
        let (src, dst) = line.split_once(':').unwrap_or((line, line));
        for path in [src, dst] {
            check_path(path).map_err(error)?;
        }
        if entries.iter().any(|e| e.dst == dst) {
            return Err(error(format!("« {dst} » est déjà dans la liste")));
        }
        entries.push(Entry {
            src: src.to_owned(),
            dst: dst.to_owned(),
        });
    }
    Ok(entries)
}

/// Un chemin de la liste doit être relatif, sans `..`, et commencer par une partition.
fn check_path(path: &str) -> Result<(), String> {
    if path.is_empty() || path.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(format!("« {path} » : chemin vide ou avec des espaces"));
    }
    // `$` et `#` auraient un sens dans le fragment Make.
    if path.contains(['$', '#', '\\']) {
        return Err(format!("« {path} » : caractère interdit ($, # ou \\)"));
    }
    let normal = Path::new(path)
        .components()
        .all(|c| matches!(c, Component::Normal(_)));
    if !normal || path.ends_with('/') {
        return Err(format!(
            "« {path} » : le chemin doit être relatif, sans « .. »"
        ));
    }
    match path.split_once('/') {
        Some((partition, _)) if PARTITIONS.iter().any(|(name, _)| *name == partition) => Ok(()),
        _ => Err(format!(
            "« {path} » : le chemin doit commencer par une partition ({})",
            PARTITIONS
                .iter()
                .map(|(n, _)| *n)
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

/// Le résultat d'une extraction.
#[derive(Debug, Default)]
pub struct Outcome {
    pub copied: Vec<Entry>,
    /// Les fichiers de la liste introuvables dans le système d'origine.
    pub missing: Vec<Entry>,
}

/// Copie chaque fichier de `from/<src>` vers `out/proprietary/<dst>`. Un fichier
/// introuvable est noté et n'arrête pas l'extraction.
pub fn extract(entries: &[Entry], from: &Path, out: &Path) -> io::Result<Outcome> {
    let mut outcome = Outcome::default();
    for entry in entries {
        let source = from.join(&entry.src);
        if !source.is_file() {
            outcome.missing.push(entry.clone());
            continue;
        }
        let target = proprietary_dir(out).join(&entry.dst);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&source, &target)?;
        outcome.copied.push(entry.clone());
    }
    Ok(outcome)
}

fn proprietary_dir(out: &Path) -> PathBuf {
    out.join("proprietary")
}

/// Le fragment Make qui installe les fichiers copiés. `tree_path` est le chemin du
/// dossier de sortie dans l'arbre d'AOSP, par exemple `vendor/nothing/SuperContra`.
pub fn vendor_mk(copied: &[Entry], tree_path: &str) -> String {
    let mut out = String::from(
        "# Généré par `cf blobs` depuis proprietary-files.txt.\n\
         # Ne pas modifier à la main : relancer la commande.\n",
    );
    if copied.is_empty() {
        return out;
    }
    out.push_str("\nPRODUCT_COPY_FILES += \\\n");
    let tree_path = tree_path.trim_end_matches('/');
    for (i, entry) in copied.iter().enumerate() {
        let end = if i + 1 == copied.len() { "\n" } else { " \\\n" };
        out.push_str(&format!(
            "    {tree_path}/proprietary/{}:{}{end}",
            entry.dst,
            entry.install_path()
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(src: &str, dst: &str) -> Entry {
        Entry {
            src: src.into(),
            dst: dst.into(),
        }
    }

    #[test]
    fn parses_paths_renames_and_comments() {
        let list = "# pilotes\n\nvendor/lib64/liba.so\n  odm/etc/a.xml:vendor/etc/b.xml  \n";
        assert_eq!(
            parse_list(list).unwrap(),
            vec![
                entry("vendor/lib64/liba.so", "vendor/lib64/liba.so"),
                entry("odm/etc/a.xml", "vendor/etc/b.xml"),
            ]
        );
    }

    #[test]
    fn refuses_unsafe_or_unknown_paths() {
        for (list, line) in [
            ("vendor/../etc/passwd", 1),
            ("/vendor/lib64/liba.so", 1),
            ("vendor/ok.so\nboot/kernel", 2),
            ("vendor/lib/$(shell id).so", 1),
            ("vendor/a b.so", 1),
            ("vendor", 1),
            ("vendor/", 1),
            ("vendor/a.so\nodm/x.so:vendor/a.so", 2),
        ] {
            let error = parse_list(list).unwrap_err();
            assert_eq!(error.line, line, "{list:?} → {error}");
        }
    }

    #[test]
    fn install_path_uses_the_partition_variable() {
        assert_eq!(
            entry("vendor/lib64/a.so", "vendor/lib64/a.so").install_path(),
            "$(TARGET_COPY_OUT_VENDOR)/lib64/a.so"
        );
        assert_eq!(
            entry("x", "system_ext/etc/a.xml").install_path(),
            "$(TARGET_COPY_OUT_SYSTEM_EXT)/etc/a.xml"
        );
    }

    #[test]
    fn extracts_existing_files_and_reports_missing_ones() {
        let root = std::env::temp_dir().join(format!("cf-blobs-{}", std::process::id()));
        let (from, out) = (root.join("stock"), root.join("out"));
        fs::create_dir_all(from.join("vendor/lib64")).unwrap();
        fs::write(from.join("vendor/lib64/liba.so"), b"ELF").unwrap();

        let entries =
            parse_list("vendor/lib64/liba.so:vendor/lib64/libb.so\nvendor/etc/absent.xml").unwrap();
        let outcome = extract(&entries, &from, &out).unwrap();
        assert_eq!(outcome.copied, vec![entries[0].clone()]);
        assert_eq!(outcome.missing, vec![entries[1].clone()]);
        assert_eq!(
            fs::read(out.join("proprietary/vendor/lib64/libb.so")).unwrap(),
            b"ELF"
        );

        let mk = vendor_mk(&outcome.copied, "vendor/nothing/SuperContra/");
        assert!(mk.contains(
            "    vendor/nothing/SuperContra/proprietary/vendor/lib64/libb.so:$(TARGET_COPY_OUT_VENDOR)/lib64/libb.so\n"
        ));
        assert!(!mk.contains("absent.xml"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn empty_list_gives_a_makefile_without_copies() {
        assert!(!vendor_mk(&[], "vendor/nothing/SuperContra").contains("PRODUCT_COPY_FILES"));
    }
}
