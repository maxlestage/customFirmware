//! `cf doctor` : vérifie que l'ordinateur peut compiler AOSP et parler au téléphone.
//! Les seuils sont ceux de la documentation d'AOSP : 64 Go de mémoire, 400 Go de
//! disque (250 pour les sources, 150 pour la compilation), Linux x86 64 bits.

use std::path::Path;
use std::process::Command;

pub const RAM_REQUIRED_GB: u64 = 64;
pub const DISK_REQUIRED_GB: u64 = 400;
const KB_PER_GB: u64 = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Ok,
    Warning,
    Failure,
}

pub struct Check {
    pub level: Level,
    pub message: String,
}

impl Check {
    fn new(level: Level, message: impl Into<String>) -> Self {
        Check {
            level,
            message: message.into(),
        }
    }
}

/// La mémoire totale, en kilo-octets, lue dans le texte de `/proc/meminfo`.
pub fn mem_total_kb(meminfo: &str) -> Option<u64> {
    meminfo
        .lines()
        .find_map(|line| line.strip_prefix("MemTotal:"))?
        .split_whitespace()
        .next()?
        .parse()
        .ok()
}

/// L'espace libre, en kilo-octets, lu dans la sortie de `df -Pk <dossier>`.
pub fn df_available_kb(df_output: &str) -> Option<u64> {
    // Deuxième ligne : système de fichiers, taille, utilisé, disponible, …
    df_output
        .lines()
        .nth(1)?
        .split_whitespace()
        .nth(3)?
        .parse()
        .ok()
}

pub fn check_ram(mem_kb: Option<u64>) -> Check {
    match mem_kb {
        // Une machine vendue « 64 Go » en annonce un peu moins au système.
        Some(kb) if kb >= (RAM_REQUIRED_GB - 2) * KB_PER_GB => {
            Check::new(Level::Ok, format!("mémoire : {} Go", kb / KB_PER_GB))
        }
        Some(kb) if kb >= 30 * KB_PER_GB => Check::new(
            Level::Warning,
            format!(
                "mémoire : {} Go ; AOSP en demande {RAM_REQUIRED_GB}, la compilation peut échouer à l'édition de liens",
                kb / KB_PER_GB
            ),
        ),
        Some(kb) => Check::new(
            Level::Failure,
            format!(
                "mémoire : {} Go ; AOSP en demande {RAM_REQUIRED_GB}",
                kb / KB_PER_GB
            ),
        ),
        None => Check::new(
            Level::Warning,
            "mémoire : impossible de la lire (/proc/meminfo)",
        ),
    }
}

pub fn check_disk(available_kb: Option<u64>, dir: &Path) -> Check {
    match available_kb {
        Some(kb) if kb >= DISK_REQUIRED_GB * KB_PER_GB => Check::new(
            Level::Ok,
            format!(
                "disque : {} Go libres dans {}",
                kb / KB_PER_GB,
                dir.display()
            ),
        ),
        Some(kb) => Check::new(
            Level::Failure,
            format!(
                "disque : {} Go libres dans {} ; AOSP en demande {DISK_REQUIRED_GB}",
                kb / KB_PER_GB,
                dir.display()
            ),
        ),
        None => Check::new(
            Level::Warning,
            format!(
                "disque : impossible de mesurer l'espace libre de {}",
                dir.display()
            ),
        ),
    }
}

pub fn check_platform(os: &str, arch: &str) -> Check {
    if os == "linux" && arch == "x86_64" {
        Check::new(Level::Ok, "système : Linux x86 64 bits")
    } else {
        Check::new(
            Level::Failure,
            format!("système : {os} {arch} ; AOSP se compile sur Linux x86 64 bits"),
        )
    }
}

/// Cherche un programme dans les dossiers de `path_var` (la valeur de `PATH`).
pub fn find_in_path(program: &str, path_var: &str) -> bool {
    std::env::split_paths(path_var).any(|dir| dir.join(program).is_file())
}

pub fn check_tool(program: &str, purpose: &str, path_var: &str) -> Check {
    if find_in_path(program, path_var) {
        Check::new(Level::Ok, format!("{program} : trouvé"))
    } else {
        Check::new(
            Level::Failure,
            format!("{program} : introuvable ({purpose})"),
        )
    }
}

/// Toutes les vérifications, sur cette machine. `dir` est le dossier où AOSP sera téléchargé.
pub fn run(dir: &Path) -> Vec<Check> {
    let meminfo = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let df = Command::new("df")
        .arg("-Pk")
        .arg(dir)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
        .unwrap_or_default();
    let path = std::env::var("PATH").unwrap_or_default();
    let mut checks = vec![
        check_platform(std::env::consts::OS, std::env::consts::ARCH),
        check_ram(mem_total_kb(&meminfo)),
        check_disk(df_available_kb(&df), dir),
    ];
    for (program, purpose) in [
        ("git", "télécharger les sources"),
        ("repo", "télécharger AOSP : sudo apt-get install repo"),
        ("adb", "parler au téléphone : platform-tools de Google"),
        (
            "fastboot",
            "installer sur le téléphone : platform-tools de Google",
        ),
    ] {
        checks.push(check_tool(program, purpose, &path));
    }
    checks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_meminfo_and_df() {
        let meminfo = "MemTotal:       65806540 kB\nMemFree:         1234 kB\n";
        assert_eq!(mem_total_kb(meminfo), Some(65_806_540));
        assert_eq!(mem_total_kb("rien"), None);

        let df = "Filesystem     1024-blocks      Used Available Capacity Mounted on\n/dev/nvme0n1p2  1921724608 512000000 1311000000      29% /\n";
        assert_eq!(df_available_kb(df), Some(1_311_000_000));
        assert_eq!(df_available_kb(""), None);
    }

    #[test]
    fn ram_thresholds() {
        assert_eq!(check_ram(Some(65_806_540)).level, Level::Ok); // « 64 Go »
        assert_eq!(check_ram(Some(32 * KB_PER_GB)).level, Level::Warning);
        assert_eq!(check_ram(Some(16 * KB_PER_GB)).level, Level::Failure);
        assert_eq!(check_ram(None).level, Level::Warning);
    }

    #[test]
    fn disk_threshold() {
        let dir = Path::new("/aosp");
        assert_eq!(check_disk(Some(500 * KB_PER_GB), dir).level, Level::Ok);
        assert_eq!(check_disk(Some(399 * KB_PER_GB), dir).level, Level::Failure);
    }

    #[test]
    fn platform_must_be_linux_x86_64() {
        assert_eq!(check_platform("linux", "x86_64").level, Level::Ok);
        assert_eq!(check_platform("macos", "aarch64").level, Level::Failure);
    }

    #[test]
    fn finds_tools_in_path() {
        let dir = std::env::temp_dir().join(format!("cf-doctor-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("outil"), "").unwrap();
        let path = format!("/dossier/absent:{}", dir.display());
        assert!(find_in_path("outil", &path));
        assert!(!find_in_path("autre", &path));
        assert_eq!(check_tool("autre", "exemple", &path).level, Level::Failure);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
