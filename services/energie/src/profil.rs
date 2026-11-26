//! Les profils : des listes de réglages du noyau, et leur application.
//!
//! Avant de changer un réglage pour la première fois, sa valeur d'origine est notée dans
//! le fichier `origine.etat`. Passer d'un profil à un autre remet d'abord les réglages
//! que le nouveau profil ne touche pas ; le profil `origine` les remet tous.

use crate::config;
use crate::systeme::{self, Systeme};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Le profil qui remet le noyau dans l'état où on l'a trouvé.
pub const ORIGINE: &str = "origine";

#[derive(Debug, PartialEq)]
pub enum Valeur {
    /// Écrite telle quelle.
    Fixe(String),
    /// `85% cpuinfo_max_freq` : 85 % du nombre lu dans le fichier voisin, arrondi à la
    /// fréquence disponible juste en dessous s'il y a un `scaling_available_frequencies`.
    Pourcentage { pourcent: u64, voisin: String },
}

#[derive(Debug)]
pub struct Reglage {
    pub motif: String,
    pub valeur: Valeur,
    pub ligne: usize,
}

#[derive(Debug)]
pub struct Profil {
    pub nom: String,
    pub description: String,
    pub reglages: Vec<Reglage>,
}

pub fn nom_valide(nom: &str) -> bool {
    !nom.is_empty()
        && nom
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

fn lire_valeur(texte: &str) -> Valeur {
    if let Some((pourcent, voisin)) = texte.split_once('%') {
        let voisin = voisin.trim();
        if let Ok(pourcent) = pourcent.trim().parse::<u64>() {
            if !voisin.is_empty() && !voisin.contains('/') && !voisin.contains(' ') {
                return Valeur::Pourcentage {
                    pourcent,
                    voisin: voisin.to_string(),
                };
            }
        }
    }
    Valeur::Fixe(texte.to_string())
}

impl Profil {
    pub fn depuis_texte(nom: &str, texte: &str) -> Result<Profil, String> {
        let (description, lignes) = config::lire(texte).map_err(|e| format!("{nom} : {e}"))?;
        let mut reglages = Vec::new();
        for ligne in lignes {
            if !systeme::chemin_autorise(&ligne.cle) {
                return Err(format!(
                    "{nom}, ligne {} : « {} » n'est pas un réglage du noyau (/sys/… ou /proc/sys/…)",
                    ligne.numero, ligne.cle
                ));
            }
            if let Valeur::Pourcentage { pourcent, .. } = lire_valeur(&ligne.valeur) {
                if pourcent == 0 || pourcent > 100 {
                    return Err(format!(
                        "{nom}, ligne {} : le pourcentage doit être entre 1 et 100",
                        ligne.numero
                    ));
                }
            }
            reglages.push(Reglage {
                motif: ligne.cle,
                valeur: lire_valeur(&ligne.valeur),
                ligne: ligne.numero,
            });
        }
        Ok(Profil {
            nom: nom.to_string(),
            description,
            reglages,
        })
    }

    /// Lit `<dossier>/profils/<nom>.conf`. `origine` existe toujours et ne règle rien.
    pub fn charger(dossier: &Path, nom: &str) -> Result<Profil, String> {
        if !nom_valide(nom) {
            return Err(format!(
                "nom de profil invalide : « {nom} » (minuscules, chiffres, - et _)"
            ));
        }
        if nom == ORIGINE {
            return Ok(Profil {
                nom: ORIGINE.into(),
                description: "Les réglages du noyau tels qu'ils étaient avant energie.".into(),
                reglages: Vec::new(),
            });
        }
        let chemin = dossier.join("profils").join(format!("{nom}.conf"));
        let texte = fs::read_to_string(&chemin)
            .map_err(|e| format!("profil « {nom} » introuvable ({}) : {e}", chemin.display()))?;
        Profil::depuis_texte(nom, &texte)
    }

    /// Les chemins réels et les valeurs à y écrire, plus les avertissements (motif sans
    /// fichier, voisin illisible). Si deux lignes touchent le même fichier, la dernière gagne.
    pub fn resoudre(&self, sys: &Systeme) -> (Vec<(String, String)>, Vec<String>) {
        let mut cibles: Vec<(String, String)> = Vec::new();
        let mut avertissements = Vec::new();
        for reglage in &self.reglages {
            let chemins = sys.developper(&reglage.motif);
            if chemins.is_empty() {
                avertissements.push(format!(
                    "ligne {} : aucun fichier pour {}",
                    reglage.ligne, reglage.motif
                ));
            }
            for chemin in chemins {
                let valeur = match &reglage.valeur {
                    Valeur::Fixe(v) => v.clone(),
                    Valeur::Pourcentage { pourcent, voisin } => {
                        match pourcentage(sys, &chemin, *pourcent, voisin) {
                            Some(v) => v.to_string(),
                            None => {
                                avertissements.push(format!(
                                    "ligne {} : {}/{voisin} illisible, {chemin} n'est pas réglé",
                                    reglage.ligne,
                                    systeme::dossier(&chemin)
                                ));
                                continue;
                            }
                        }
                    }
                };
                cibles.retain(|(c, _)| *c != chemin);
                cibles.push((chemin, valeur));
            }
        }
        (cibles, avertissements)
    }
}

fn pourcentage(sys: &Systeme, chemin: &str, pourcent: u64, voisin: &str) -> Option<u64> {
    let dossier = systeme::dossier(chemin);
    let base: u64 = sys
        .lire(&format!("{dossier}/{voisin}"))
        .ok()?
        .parse()
        .ok()?;
    let cible = base * pourcent / 100;
    let disponibles: Vec<u64> = sys
        .lire(&format!("{dossier}/scaling_available_frequencies"))
        .map(|t| {
            t.split_whitespace()
                .filter_map(|f| f.parse().ok())
                .collect()
        })
        .unwrap_or_default();
    if disponibles.is_empty() {
        return Some(cible);
    }
    disponibles
        .iter()
        .copied()
        .filter(|&f| f <= cible)
        .max()
        .or_else(|| disponibles.iter().copied().min())
}

/// Les valeurs d'origine des réglages changés, gardées dans `origine.etat`
/// (une ligne `chemin<TAB>valeur` par réglage).
pub struct Origines {
    fichier: std::path::PathBuf,
    pub valeurs: BTreeMap<String, String>,
}

impl Origines {
    pub fn charger(dossier: &Path) -> Result<Origines, String> {
        let fichier = dossier.join("origine.etat");
        let mut valeurs = BTreeMap::new();
        if let Ok(texte) = fs::read_to_string(&fichier) {
            for ligne in texte.lines().filter(|l| !l.is_empty()) {
                let (chemin, valeur) = ligne
                    .split_once('\t')
                    .ok_or_else(|| format!("{} abîmé : « {ligne} »", fichier.display()))?;
                valeurs.insert(chemin.to_string(), valeur.to_string());
            }
        }
        Ok(Origines { fichier, valeurs })
    }

    fn enregistrer(&self) -> Result<(), String> {
        let texte: String = self
            .valeurs
            .iter()
            .map(|(c, v)| format!("{c}\t{v}\n"))
            .collect();
        // Écrit à côté puis renomme : un fichier à moitié écrit ferait perdre les origines.
        let provisoire = self.fichier.with_extension("etat.tmp");
        fs::write(&provisoire, texte)
            .and_then(|()| fs::rename(&provisoire, &self.fichier))
            .map_err(|e| format!("impossible d'écrire {} : {e}", self.fichier.display()))
    }
}

/// Ce qu'a fait l'application d'un profil.
#[derive(Default, Debug)]
pub struct Bilan {
    /// (chemin, avant, après)
    pub changes: Vec<(String, String, String)>,
    pub inchanges: usize,
    pub erreurs: Vec<String>,
    pub avertissements: Vec<String>,
}

/// Applique un profil : remet d'abord les réglages qu'il ne touche pas, puis écrit les siens.
pub fn appliquer(sys: &Systeme, dossier: &Path, profil: &Profil) -> Result<Bilan, String> {
    let mut origines = Origines::charger(dossier)?;
    let (cibles, avertissements) = profil.resoudre(sys);
    let mut bilan = Bilan {
        avertissements,
        ..Bilan::default()
    };

    // 1. Noter les valeurs d'origine avant toute écriture.
    let mut cibles_lisibles = Vec::new();
    for (chemin, valeur) in cibles {
        if !origines.valeurs.contains_key(&chemin) {
            match sys.lire(&chemin) {
                Ok(actuelle) if actuelle.contains(['\n', '\t']) => {
                    bilan
                        .erreurs
                        .push(format!("{chemin} : valeur sur plusieurs lignes, non gérée"));
                    continue;
                }
                Ok(actuelle) => {
                    origines.valeurs.insert(chemin.clone(), actuelle);
                }
                Err(e) => {
                    bilan
                        .erreurs
                        .push(format!("{chemin} : lecture impossible ({e})"));
                    continue;
                }
            }
        }
        cibles_lisibles.push((chemin, valeur));
    }
    origines.enregistrer()?;

    // 2. Remettre ce que le nouveau profil ne touche pas.
    let a_remettre: Vec<(String, String)> = origines
        .valeurs
        .iter()
        .filter(|(c, _)| !cibles_lisibles.iter().any(|(cible, _)| cible == *c))
        .map(|(c, v)| (c.clone(), v.clone()))
        .collect();
    for (chemin, valeur) in a_remettre {
        if ecrire(sys, &chemin, &valeur, &mut bilan) {
            origines.valeurs.remove(&chemin);
        }
    }

    // 3. Écrire les réglages du profil.
    for (chemin, valeur) in &cibles_lisibles {
        ecrire(sys, chemin, valeur, &mut bilan);
    }
    origines.enregistrer()?;
    let _ = ecrire_partage(&dossier.join("actif"), &profil.nom);
    Ok(bilan)
}

/// Écrit un fichier que l'application Énergie (uid system) doit pouvoir lire : 0644,
/// quel que soit le masque de création du démon (init le règle à 077).
pub fn ecrire_partage(fichier: &Path, texte: &str) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::write(fichier, texte)?;
    fs::set_permissions(fichier, fs::Permissions::from_mode(0o644))
}

/// Écrit si la valeur change ; vrai si le fichier a la bonne valeur à la fin.
fn ecrire(sys: &Systeme, chemin: &str, valeur: &str, bilan: &mut Bilan) -> bool {
    let avant = sys.lire(chemin).unwrap_or_default();
    if avant == valeur {
        bilan.inchanges += 1;
        return true;
    }
    match sys.ecrire(chemin, valeur) {
        Ok(()) => {
            bilan
                .changes
                .push((chemin.to_string(), avant, valeur.to_string()));
            true
        }
        Err(e) => {
            bilan.erreurs.push(format!("{chemin} = {valeur} : {e}"));
            false
        }
    }
}

/// Le profil appliqué en dernier, s'il y en a un.
pub fn actif(dossier: &Path) -> Option<String> {
    fs::read_to_string(dossier.join("actif"))
        .ok()
        .map(|n| n.trim().to_string())
}

/// Les profils disponibles dans `<dossier>/profils`, plus `origine`.
pub fn liste(dossier: &Path) -> Vec<String> {
    let mut noms: Vec<String> = fs::read_dir(dossier.join("profils"))
        .into_iter()
        .flatten()
        .filter_map(|e| {
            let nom = e.ok()?.file_name().into_string().ok()?;
            Some(nom.strip_suffix(".conf")?.to_string())
        })
        .filter(|n| nom_valide(n) && n != ORIGINE)
        .collect();
    noms.sort();
    noms.insert(0, ORIGINE.to_string());
    noms
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systeme::tests::FauxArbre;

    const POLICY: &str = "/sys/devices/system/cpu/cpufreq";

    fn telephone(nom: &str) -> FauxArbre {
        let arbre = FauxArbre::nouveau(nom);
        for (n, max) in [(0, 2_000_000), (4, 3_000_000)] {
            arbre
                .fichier(
                    &format!("{POLICY}/policy{n}/cpuinfo_max_freq"),
                    &max.to_string(),
                )
                .fichier(
                    &format!("{POLICY}/policy{n}/scaling_max_freq"),
                    &max.to_string(),
                );
        }
        arbre.fichier(
            &format!("{POLICY}/policy4/scaling_available_frequencies"),
            "1000000 2000000 2400000 2800000 3000000",
        );
        arbre.fichier("/proc/sys/walt/sched_boost", "0\n");
        arbre.fichier("/proc/sys/vm/dirty_writeback_centisecs", "500\n");
        arbre.fichier("/data/profils/.garde", "");
        arbre
    }

    fn profil(nom: &str, texte: &str) -> Profil {
        Profil::depuis_texte(nom, texte).unwrap()
    }

    #[test]
    fn lit_les_valeurs() {
        assert_eq!(
            lire_valeur("85% cpuinfo_max_freq"),
            Valeur::Pourcentage {
                pourcent: 85,
                voisin: "cpuinfo_max_freq".into()
            }
        );
        assert_eq!(lire_valeur("0 1 2"), Valeur::Fixe("0 1 2".into()));
        assert_eq!(lire_valeur("50%"), Valeur::Fixe("50%".into()));
        assert!(Profil::depuis_texte("x", "/data/x = 1").is_err());
        assert!(Profil::depuis_texte("x", "/sys/x = 150% y").is_err());
    }

    #[test]
    fn pourcentage_arrondi_aux_frequences_disponibles() {
        let arbre = telephone("pourcentage");
        let sys = arbre.systeme();
        let p = profil(
            "eco",
            &format!("{POLICY}/policy*/scaling_max_freq = 85% cpuinfo_max_freq"),
        );
        let (cibles, avertissements) = p.resoudre(&sys);
        assert!(avertissements.is_empty());
        assert_eq!(
            cibles,
            [
                // Pas de liste de fréquences : 85 % tout rond.
                (
                    format!("{POLICY}/policy0/scaling_max_freq"),
                    "1700000".into()
                ),
                // 2 550 000 n'existe pas : la fréquence juste en dessous.
                (
                    format!("{POLICY}/policy4/scaling_max_freq"),
                    "2400000".into()
                ),
            ]
        );
    }

    #[test]
    fn passe_d_un_profil_a_l_autre_puis_revient_a_l_origine() {
        let arbre = telephone("appliquer");
        let sys = arbre.systeme();
        let dossier = arbre.racine.join("data");
        let lire = |c: &str| sys.lire(c).unwrap();
        let max0 = format!("{POLICY}/policy0/scaling_max_freq");
        let ecriture = "/proc/sys/vm/dirty_writeback_centisecs";

        let eco = profil("eco", &format!("{POLICY}/policy*/scaling_max_freq = 50% cpuinfo_max_freq\n/proc/sys/walt/sched_boost = 0"));
        let bilan = appliquer(&sys, &dossier, &eco).unwrap();
        assert!(bilan.erreurs.is_empty(), "{:?}", bilan.erreurs);
        assert_eq!(bilan.changes.len(), 2);
        assert_eq!(bilan.inchanges, 1);
        assert_eq!(lire(&max0), "1000000");
        assert_eq!(actif(&dossier).as_deref(), Some("eco"));
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(dossier.join("actif"))
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o644, "lisible par l'application");
        }

        // Le profil suivant ne touche plus aux fréquences : elles reviennent.
        let nuit = profil("nuit", &format!("{ecriture} = 3000"));
        appliquer(&sys, &dossier, &nuit).unwrap();
        assert_eq!(lire(&max0), "2000000");
        assert_eq!(lire(ecriture), "3000");

        // Réappliquer après un redémarrage d'energie ne perd pas la vraie origine.
        appliquer(&sys, &dossier, &nuit).unwrap();
        appliquer(&sys, &dossier, &Profil::charger(&dossier, ORIGINE).unwrap()).unwrap();
        assert_eq!(lire(ecriture), "500");
        assert_eq!(
            lire(&format!("{POLICY}/policy4/scaling_max_freq")),
            "3000000"
        );
        assert!(Origines::charger(&dossier).unwrap().valeurs.is_empty());
    }

    #[test]
    fn signale_les_reglages_absents() {
        let arbre = telephone("absents");
        let p = profil("x", "/sys/class/kgsl/kgsl-3d0/max_pwrlevel = 1");
        let (cibles, avertissements) = p.resoudre(&arbre.systeme());
        assert!(cibles.is_empty());
        assert_eq!(avertissements.len(), 1);
    }

    #[test]
    fn liste_les_profils() {
        let arbre = telephone("liste");
        arbre
            .fichier("/data/profils/nuit.conf", "")
            .fichier("/data/profils/eco.conf", "")
            .fichier("/data/profils/notes.txt", "");
        assert_eq!(
            liste(&arbre.racine.join("data")),
            ["origine", "eco", "nuit"]
        );
    }
}
