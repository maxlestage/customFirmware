//! `cf doctor` : vérifie que l'ordinateur peut compiler AOSP et parler au téléphone.
//! Les seuils sont ceux de la documentation d'AOSP : 64 Go de mémoire, 400 Go de
//! disque (250 pour les sources, 150 pour la compilation), Linux x86 64 bits.
//!
//! Il reconnaît la famille du système (Ubuntu ou Debian, Fedora, Arch, macOS) : les
//! commandes qu'il propose et la page du guide qu'il indique sont celles de ce système.

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

/// La famille du système : elle décide des commandes d'installation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    Debian,
    Fedora,
    Arch,
    Macos,
    Other,
}

impl Family {
    /// La famille, d'après le texte de `/etc/os-release` (ID, puis ID_LIKE).
    pub fn from_os_release(text: &str) -> Family {
        let value = |key: &str| {
            text.lines()
                .find_map(|line| line.strip_prefix(key)?.strip_prefix('='))
                .map(|v| v.trim().trim_matches('"').to_lowercase())
                .unwrap_or_default()
        };
        let ids = format!("{} {}", value("ID"), value("ID_LIKE"));
        let has = |id: &str| ids.split_whitespace().any(|i| i == id);
        if has("debian") || has("ubuntu") {
            Family::Debian
        } else if has("fedora") || has("rhel") {
            Family::Fedora
        } else if has("arch") {
            Family::Arch
        } else {
            Family::Other
        }
    }

    /// La page du guide qui donne les commandes de ce système.
    pub fn guide(self) -> &'static str {
        match self {
            Family::Debian | Family::Other => "https://maxlestage.github.io/customFirmware/guide/",
            Family::Fedora => "https://maxlestage.github.io/customFirmware/guide/fedora/",
            Family::Arch => "https://maxlestage.github.io/customFirmware/guide/arch/",
            Family::Macos => "https://maxlestage.github.io/customFirmware/guide/macos/",
        }
    }

    /// La commande qui installe un paquet, ou `None` si le gestionnaire est inconnu.
    /// Sur Mac, `mac` est la commande entière (Homebrew ou outils d'Apple).
    fn install(self, debian: &str, fedora: &str, arch: &str, mac: &str) -> Option<String> {
        match self {
            Family::Debian => Some(format!("sudo apt-get install {debian}")),
            Family::Fedora => Some(format!("sudo dnf install {fedora}")),
            Family::Arch => Some(format!("sudo pacman -S {arch}")),
            Family::Macos => Some(mac.to_string()),
            Family::Other => None,
        }
    }
}

/// Le nom lisible du système, d'après `/etc/os-release` (PRETTY_NAME).
pub fn pretty_name(os_release: &str) -> Option<String> {
    os_release
        .lines()
        .find_map(|line| line.strip_prefix("PRETTY_NAME="))
        .map(|v| v.trim().trim_matches('"').to_string())
        .filter(|v| !v.is_empty())
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
        None => Check::new(Level::Warning, "mémoire : impossible de la lire"),
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

/// `name` : le nom lisible du système (« Fedora Linux 43 », « macOS 15.1 »).
pub fn check_platform(os: &str, arch: &str, name: &str) -> Check {
    if os == "linux" && arch == "x86_64" {
        Check::new(Level::Ok, format!("système : {name}, x86 64 bits"))
    } else if os == "macos" {
        Check::new(
            Level::Failure,
            format!(
                "système : {name} ({arch}) ; AOSP ne se compile plus sur macOS : le Mac suffit jusqu'à l'étape 05, ensuite il faut Linux x86 64 bits"
            ),
        )
    } else {
        Check::new(
            Level::Failure,
            format!("système : {name} ({arch}) ; AOSP se compile sur Linux x86 64 bits"),
        )
    }
}

/// Cherche un programme dans les dossiers de `path_var` (la valeur de `PATH`).
pub fn find_in_path(program: &str, path_var: &str) -> bool {
    std::env::split_paths(path_var).any(|dir| dir.join(program).is_file())
}

/// `install` : la commande qui l'installe sur ce système, si on la connaît.
pub fn check_tool(program: &str, purpose: &str, install: Option<String>, path_var: &str) -> Check {
    if find_in_path(program, path_var) {
        return Check::new(Level::Ok, format!("{program} : trouvé"));
    }
    let install = install.map(|i| format!(" : {i}")).unwrap_or_default();
    Check::new(
        Level::Failure,
        format!("{program} : introuvable, pour {purpose}{install}"),
    )
}

/// Les outils à trouver sur ce système : (programme, à quoi il sert, comment l'installer).
pub fn tools(family: Family) -> Vec<(&'static str, &'static str, Option<String>)> {
    let platform_tools = match family {
        Family::Macos => Some("brew install --cask android-platform-tools".to_string()),
        _ => Some("les platform-tools de Google (guide, étape 01)".to_string()),
    };
    let mut tools = vec![
        (
            "git",
            "télécharger les sources",
            family.install("git", "git", "git", "xcode-select --install"),
        ),
        ("adb", "parler au téléphone", platform_tools.clone()),
        ("fastboot", "installer sur le téléphone", platform_tools),
    ];
    // Les archives du firmware d'origine (étape 03) : `7zz` sur Mac, `7z` ailleurs.
    if family == Family::Macos {
        tools.push((
            "7zz",
            "décompresser le firmware d'origine",
            Some("brew install sevenzip".into()),
        ));
    } else {
        tools.push((
            "7z",
            "décompresser le firmware d'origine",
            family.install("p7zip-full", "7zip", "7zip", ""),
        ));
        // repo ne sert qu'à télécharger AOSP, qui ne se compile pas sur Mac.
        tools.push((
            "repo",
            "télécharger AOSP",
            family
                .install("repo", "repo", "repo", "")
                .or_else(|| Some("voir https://source.android.com/docs/setup/download".into())),
        ));
    }
    tools
}

/// La sortie d'une commande, si elle réussit.
fn output(program: &str, args: &[&str]) -> Option<String> {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

/// La famille et le nom lisible du système de cette machine.
pub fn this_system() -> (Family, String) {
    if std::env::consts::OS == "macos" {
        let version = output("sw_vers", &["-productVersion"]).unwrap_or_default();
        return (Family::Macos, format!("macOS {version}").trim().to_string());
    }
    let os_release = std::fs::read_to_string("/etc/os-release").unwrap_or_default();
    (
        Family::from_os_release(&os_release),
        pretty_name(&os_release).unwrap_or_else(|| std::env::consts::OS.to_string()),
    )
}

/// La mémoire totale en kilo-octets : /proc/meminfo sous Linux, sysctl sur Mac.
fn memory_kb() -> Option<u64> {
    if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
        return mem_total_kb(&meminfo);
    }
    let bytes: u64 = output("sysctl", &["-n", "hw.memsize"])?.parse().ok()?;
    Some(bytes / 1024)
}

/// Toutes les vérifications, sur cette machine. `dir` est le dossier où AOSP sera téléchargé.
pub fn run(dir: &Path, family: Family, name: &str) -> Vec<Check> {
    let df = output("df", &["-Pk", &dir.to_string_lossy()]).unwrap_or_default();
    let path = std::env::var("PATH").unwrap_or_default();
    let mut checks = vec![
        check_platform(std::env::consts::OS, std::env::consts::ARCH, name),
        check_ram(memory_kb()),
        check_disk(df_available_kb(&df), dir),
    ];
    for (program, purpose, install) in tools(family) {
        checks.push(check_tool(program, purpose, install, &path));
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
        assert_eq!(
            check_platform("linux", "x86_64", "Fedora Linux 43").level,
            Level::Ok
        );
        let mac = check_platform("macos", "aarch64", "macOS 15.1");
        assert_eq!(mac.level, Level::Failure);
        assert!(mac.message.contains("étape 05"));
        assert_eq!(
            check_platform("linux", "aarch64", "Ubuntu").level,
            Level::Failure
        );
    }

    #[test]
    fn recognises_the_distribution() {
        let ubuntu =
            "NAME=\"Ubuntu\"\nID=ubuntu\nID_LIKE=debian\nPRETTY_NAME=\"Ubuntu 24.04.1 LTS\"\n";
        assert_eq!(Family::from_os_release(ubuntu), Family::Debian);
        assert_eq!(pretty_name(ubuntu).as_deref(), Some("Ubuntu 24.04.1 LTS"));
        assert_eq!(
            Family::from_os_release("ID=linuxmint\nID_LIKE=\"ubuntu debian\"\n"),
            Family::Debian
        );
        assert_eq!(Family::from_os_release("ID=fedora\n"), Family::Fedora);
        assert_eq!(
            Family::from_os_release("ID=nobara\nID_LIKE=\"rhel centos fedora\"\n"),
            Family::Fedora
        );
        assert_eq!(Family::from_os_release("ID=arch\n"), Family::Arch);
        assert_eq!(
            Family::from_os_release("ID=endeavouros\nID_LIKE=arch\n"),
            Family::Arch
        );
        assert_eq!(
            Family::from_os_release("ID=opensuse-tumbleweed\nID_LIKE=\"opensuse suse\"\n"),
            Family::Other
        );
        assert_eq!(Family::from_os_release(""), Family::Other);
        assert!(Family::Fedora.guide().ends_with("/guide/fedora/"));
    }

    #[test]
    fn install_commands_follow_the_system() {
        let hint = |family: Family, program: &str| {
            tools(family)
                .into_iter()
                .find(|(p, _, _)| *p == program)
                .and_then(|(_, _, install)| install)
        };
        assert_eq!(
            hint(Family::Debian, "repo").as_deref(),
            Some("sudo apt-get install repo")
        );
        assert_eq!(
            hint(Family::Fedora, "7z").as_deref(),
            Some("sudo dnf install 7zip")
        );
        assert_eq!(
            hint(Family::Arch, "git").as_deref(),
            Some("sudo pacman -S git")
        );
        assert_eq!(
            hint(Family::Macos, "git").as_deref(),
            Some("xcode-select --install")
        );
        assert_eq!(
            hint(Family::Macos, "7zz").as_deref(),
            Some("brew install sevenzip")
        );
        assert!(hint(Family::Macos, "repo").is_none(), "pas de repo sur Mac");
        assert!(
            hint(Family::Other, "repo")
                .unwrap()
                .contains("source.android.com")
        );
        assert!(
            hint(Family::Macos, "adb")
                .unwrap()
                .contains("brew install --cask")
        );
    }

    #[test]
    fn finds_tools_in_path() {
        let dir = std::env::temp_dir().join(format!("cf-doctor-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("outil"), "").unwrap();
        let path = format!("/dossier/absent:{}", dir.display());
        assert!(find_in_path("outil", &path));
        assert!(!find_in_path("autre", &path));
        let missing = check_tool(
            "autre",
            "essayer",
            Some("sudo dnf install autre".into()),
            &path,
        );
        assert_eq!(missing.level, Level::Failure);
        assert_eq!(
            missing.message,
            "autre : introuvable, pour essayer : sudo dnf install autre"
        );
        std::fs::remove_dir_all(dir).unwrap();
    }
}
