//! L'accès aux fichiers du noyau (/sys et /proc/sys).
//!
//! Tous les chemins manipulés sont des chemins « logiques », comme sur le téléphone
//! (`/sys/devices/...`). Une racine permet de les faire pointer vers un faux arbre pour
//! les tests.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub struct Systeme {
    racine: PathBuf,
}

/// Seuls les réglages du noyau peuvent être écrits : rien en dehors de /sys et /proc/sys.
pub fn chemin_autorise(chemin: &str) -> bool {
    (chemin.starts_with("/sys/") || chemin.starts_with("/proc/sys/"))
        && !chemin.split('/').any(|morceau| morceau == "..")
}

/// `*` remplace n'importe quelle suite de caractères (même vide) dans un nom.
pub fn correspond(motif: &str, nom: &str) -> bool {
    let morceaux: Vec<&str> = motif.split('*').collect();
    if morceaux.len() == 1 {
        return motif == nom;
    }
    let (premier, dernier) = (morceaux[0], morceaux[morceaux.len() - 1]);
    if nom.len() < premier.len() + dernier.len()
        || !nom.starts_with(premier)
        || !nom.ends_with(dernier)
    {
        return false;
    }
    let mut reste = &nom[premier.len()..nom.len() - dernier.len()];
    for morceau in &morceaux[1..morceaux.len() - 1] {
        match reste.find(morceau) {
            Some(i) => reste = &reste[i + morceau.len()..],
            None => return false,
        }
    }
    true
}

impl Systeme {
    /// Le vrai système : les chemins sont pris tels quels.
    pub fn reel() -> Self {
        Self::avec_racine("/")
    }

    pub fn avec_racine(racine: impl Into<PathBuf>) -> Self {
        Systeme {
            racine: racine.into(),
        }
    }

    fn vrai_chemin(&self, chemin: &str) -> PathBuf {
        self.racine.join(chemin.trim_start_matches('/'))
    }

    /// Le contenu d'un fichier, sans les blancs autour.
    pub fn lire(&self, chemin: &str) -> io::Result<String> {
        Ok(fs::read_to_string(self.vrai_chemin(chemin))?
            .trim()
            .to_string())
    }

    /// Le contenu d'un fichier lu comme un nombre ; `None` s'il manque ou n'en est pas un.
    pub fn nombre(&self, chemin: &str) -> Option<i64> {
        self.lire(chemin).ok()?.parse().ok()
    }

    pub fn existe(&self, chemin: &str) -> bool {
        self.vrai_chemin(chemin).exists()
    }

    /// Écrit une valeur dans un fichier existant, comme `echo valeur > fichier`.
    pub fn ecrire(&self, chemin: &str, valeur: &str) -> io::Result<()> {
        let mut fichier = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(self.vrai_chemin(chemin))?;
        fichier.write_all(valeur.as_bytes())
    }

    /// Les chemins existants qui correspondent au motif, dans l'ordre alphabétique.
    /// `*` peut se trouver dans n'importe quel morceau du chemin.
    pub fn developper(&self, motif: &str) -> Vec<String> {
        let mut trouves = vec![String::new()];
        for morceau in motif.split('/').filter(|m| !m.is_empty()) {
            let mut suivants = Vec::new();
            for parent in &trouves {
                if !morceau.contains('*') {
                    let chemin = format!("{parent}/{morceau}");
                    if self.vrai_chemin(&chemin).exists() {
                        suivants.push(chemin);
                    }
                    continue;
                }
                let Ok(entrees) = fs::read_dir(self.vrai_chemin(parent)) else {
                    continue;
                };
                let mut noms: Vec<String> = entrees
                    .filter_map(|e| e.ok()?.file_name().into_string().ok())
                    .filter(|nom| correspond(morceau, nom))
                    .collect();
                noms.sort_by(|a, b| tri_naturel(a, b));
                suivants.extend(noms.into_iter().map(|nom| format!("{parent}/{nom}")));
            }
            trouves = suivants;
        }
        trouves.retain(|c| !c.is_empty());
        trouves
    }
}

/// Trie `policy2` avant `policy10`.
fn tri_naturel(a: &str, b: &str) -> std::cmp::Ordering {
    let cle = |s: &str| {
        let chiffres = s.trim_start_matches(|c: char| !c.is_ascii_digit());
        let prefixe = &s[..s.len() - chiffres.len()];
        (
            prefixe.to_string(),
            chiffres.parse::<u64>().unwrap_or(u64::MAX),
            s.to_string(),
        )
    };
    cle(a).cmp(&cle(b))
}

/// Le dossier qui contient un chemin logique.
pub fn dossier(chemin: &str) -> &str {
    Path::new(chemin)
        .parent()
        .and_then(Path::to_str)
        .unwrap_or("/")
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// Un faux arbre /sys, effacé à la fin du test.
    pub struct FauxArbre {
        pub racine: PathBuf,
    }

    impl FauxArbre {
        pub fn nouveau(nom: &str) -> Self {
            let racine =
                std::env::temp_dir().join(format!("energie-test-{nom}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&racine);
            fs::create_dir_all(&racine).unwrap();
            FauxArbre { racine }
        }

        pub fn fichier(&self, chemin: &str, contenu: &str) -> &Self {
            let vrai = self.racine.join(chemin.trim_start_matches('/'));
            fs::create_dir_all(vrai.parent().unwrap()).unwrap();
            fs::write(vrai, contenu).unwrap();
            self
        }

        pub fn systeme(&self) -> Systeme {
            Systeme::avec_racine(&self.racine)
        }
    }

    impl Drop for FauxArbre {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.racine);
        }
    }

    #[test]
    fn motifs() {
        assert!(correspond("policy*", "policy0"));
        assert!(correspond("*", ""));
        assert!(correspond("a*b*c", "aXXbYYc"));
        assert!(!correspond("a*b*c", "acb"));
        assert!(!correspond("policy*", "cpu0"));
        assert!(correspond("exact", "exact"));
    }

    #[test]
    fn chemins_autorises() {
        assert!(chemin_autorise("/sys/devices/system/cpu/cpu0/online"));
        assert!(chemin_autorise("/proc/sys/vm/swappiness"));
        assert!(!chemin_autorise("/data/system/x"));
        assert!(!chemin_autorise("/proc/1/mem"));
        assert!(!chemin_autorise("/sys/../data/x"));
    }

    #[test]
    fn developpe_les_motifs_dans_l_ordre_naturel() {
        let arbre = FauxArbre::nouveau("developper");
        for n in [0, 2, 10] {
            arbre.fichier(&format!("/sys/cpufreq/policy{n}/scaling_max_freq"), "1");
        }
        arbre.fichier("/sys/cpufreq/autre/scaling_max_freq", "1");
        let sys = arbre.systeme();
        assert_eq!(
            sys.developper("/sys/cpufreq/policy*/scaling_max_freq"),
            [
                "/sys/cpufreq/policy0/scaling_max_freq",
                "/sys/cpufreq/policy2/scaling_max_freq",
                "/sys/cpufreq/policy10/scaling_max_freq"
            ]
        );
        assert!(sys.developper("/sys/absent/*").is_empty());
        sys.ecrire("/sys/cpufreq/policy0/scaling_max_freq", "42")
            .unwrap();
        assert_eq!(
            sys.nombre("/sys/cpufreq/policy0/scaling_max_freq"),
            Some(42)
        );
        assert!(sys.ecrire("/sys/cpufreq/inexistant", "1").is_err());
    }
}
