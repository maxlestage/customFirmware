//! Le démon : il regarde l'écran et la batterie, choisit le profil selon les règles de
//! `energie.conf`, l'applique quand il change, et tient le journal de consommation.
//!
//! Il n'empêche jamais le téléphone de dormir : son attente s'arrête avec lui en veille.
//! Il relit ses réglages dès qu'un fichier change : modifier un profil suffit.

use crate::capteurs::{Batterie, Ecran};
use crate::profil::{self, Profil};
use crate::systeme::Systeme;
use crate::{config, journal};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct Regles {
    pub defaut: String,
    pub ecran_eteint: Option<String>,
    pub en_charge: Option<String>,
    pub batterie_faible: Option<String>,
    pub seuil_batterie_faible: i64,
    pub intervalle_s: u64,
    pub journal_s: u64,
    pub batterie: String,
    pub ecran: String,
}

impl Default for Regles {
    fn default() -> Self {
        Regles {
            defaut: profil::ORIGINE.into(),
            ecran_eteint: None,
            en_charge: None,
            batterie_faible: None,
            seuil_batterie_faible: 20,
            intervalle_s: 2,
            journal_s: 60,
            batterie: "/sys/class/power_supply/battery".into(),
            ecran: "/sys/class/backlight/*/brightness".into(),
        }
    }
}

impl Regles {
    pub fn depuis_texte(texte: &str) -> Result<Regles, String> {
        let mut r = Regles::default();
        let (_, lignes) = config::lire(texte).map_err(|e| format!("energie.conf : {e}"))?;
        for l in lignes {
            let erreur = |quoi: &str| format!("energie.conf, ligne {} : {quoi}", l.numero);
            let nombre = || {
                l.valeur
                    .parse::<u64>()
                    .map_err(|_| erreur("nombre attendu"))
            };
            let profil = || -> Result<Option<String>, String> {
                if l.valeur == "aucun" {
                    Ok(None)
                } else if profil::nom_valide(&l.valeur) {
                    Ok(Some(l.valeur.clone()))
                } else {
                    Err(erreur("nom de profil invalide"))
                }
            };
            match (l.section.as_str(), l.cle.as_str()) {
                ("regles", "defaut") => {
                    r.defaut = profil()?.ok_or_else(|| erreur("il faut un profil par défaut"))?
                }
                ("regles", "ecran_eteint") => r.ecran_eteint = profil()?,
                ("regles", "en_charge") => r.en_charge = profil()?,
                ("regles", "batterie_faible") => r.batterie_faible = profil()?,
                ("regles", "seuil_batterie_faible") => {
                    r.seuil_batterie_faible = nombre()?.min(100) as i64
                }
                ("demon", "intervalle_s") => r.intervalle_s = nombre()?.max(1),
                ("demon", "journal_s") => r.journal_s = nombre()?,
                ("capteurs", "batterie") => r.batterie = l.valeur.clone(),
                ("capteurs", "ecran") => r.ecran = l.valeur.clone(),
                (section, cle) => return Err(erreur(&format!("[{section}] {cle} inconnu"))),
            }
        }
        Ok(r)
    }

    /// Lit `<dossier>/energie.conf` ; sans ce fichier, les règles par défaut.
    pub fn charger(dossier: &Path) -> Result<Regles, String> {
        match std::fs::read_to_string(dossier.join("energie.conf")) {
            Ok(texte) => Regles::depuis_texte(&texte),
            Err(_) => Ok(Regles::default()),
        }
    }

    /// Le profil voulu, par ordre de priorité : branché, écran éteint, batterie faible.
    pub fn choisir(&self, batterie: &Batterie, ecran: Ecran) -> &str {
        let faible = batterie
            .capacite
            .is_some_and(|c| c <= self.seuil_batterie_faible);
        let regles = [
            (batterie.branche(), &self.en_charge),
            (ecran == Ecran::Eteint, &self.ecran_eteint),
            (faible && !batterie.branche(), &self.batterie_faible),
        ];
        regles
            .iter()
            .find_map(|(vrai, profil)| profil.as_deref().filter(|_| *vrai))
            .unwrap_or(&self.defaut)
    }

    fn profils(&self) -> impl Iterator<Item = &String> {
        [&self.ecran_eteint, &self.en_charge, &self.batterie_faible]
            .into_iter()
            .flatten()
            .chain(std::iter::once(&self.defaut))
    }
}

fn maintenant() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn note(texte: &str) {
    eprintln!("energie [{}] {texte}", maintenant());
}

/// Le profil imposé à la main (`energie forcer <profil>`), s'il y en a un.
pub fn force(dossier: &Path) -> Option<String> {
    let nom = std::fs::read_to_string(dossier.join("force")).ok()?;
    let nom = nom.trim();
    profil::nom_valide(nom).then(|| nom.to_string())
}

/// La date et la taille de chaque fichier de réglages : si elles changent, on relit tout.
fn signature(dossier: &Path) -> Vec<(String, Option<SystemTime>, u64)> {
    let mut fichiers = vec![dossier.join("energie.conf"), dossier.join("force")];
    if let Ok(entrees) = std::fs::read_dir(dossier.join("profils")) {
        fichiers.extend(entrees.flatten().map(|e| e.path()));
    }
    let mut signature: Vec<_> = fichiers
        .iter()
        .map(|f| {
            let m = std::fs::metadata(f).ok();
            (
                f.display().to_string(),
                m.as_ref().and_then(|m| m.modified().ok()),
                m.map_or(0, |m| m.len()),
            )
        })
        .collect();
    signature.sort();
    signature
}

/// Lit les règles et vérifie tous les profils qu'elles nomment.
fn charger(dossier: &Path) -> Result<Regles, String> {
    let regles = Regles::charger(dossier)?;
    for nom in regles.profils() {
        Profil::charger(dossier, nom)?;
    }
    Ok(regles)
}

/// Tourne jusqu'à l'arrêt du téléphone. Une erreur dans les réglages ne l'arrête pas :
/// elle est notée, et les réglages précédents restent en place jusqu'à la correction.
pub fn lancer(sys: &Systeme, dossier: &Path) -> Result<(), String> {
    note("démarré");
    let fichier_journal = dossier.join("journal.csv");
    let mut regles = Regles::default();
    let mut vu = None;
    let mut actif: Option<String> = None;
    let mut dernier_journal = 0;
    loop {
        let signature = signature(dossier);
        if vu.as_ref() != Some(&signature) {
            match charger(dossier) {
                Ok(r) => {
                    note(&format!("réglages lus : {r:?}"));
                    regles = r;
                    actif = None; // un profil a pu changer : on réapplique
                }
                Err(e) => note(&format!("réglages refusés, les précédents restent : {e}")),
            }
            vu = Some(signature);
        }
        let batterie = Batterie::lire(sys, &regles.batterie);
        let ecran = Ecran::lire(sys, &regles.ecran);
        let voulu = force(dossier).unwrap_or_else(|| regles.choisir(&batterie, ecran).to_string());
        if actif.as_deref() != Some(voulu.as_str()) {
            let resultat =
                Profil::charger(dossier, &voulu).and_then(|p| profil::appliquer(sys, dossier, &p));
            match resultat {
                Ok(bilan) => {
                    note(&format!(
                        "profil {voulu} (écran {}, batterie {} {}) : {} changé(s), {} erreur(s)",
                        ecran.nom(),
                        batterie
                            .capacite
                            .map(|c| format!("{c} %"))
                            .unwrap_or_default(),
                        batterie.statut,
                        bilan.changes.len(),
                        bilan.erreurs.len()
                    ));
                    for e in &bilan.erreurs {
                        note(&format!("  erreur : {e}"));
                    }
                }
                Err(e) => note(&format!("profil {voulu} non appliqué : {e}")),
            }
            // Même en cas d'erreur : on ne réessaie qu'au prochain changement.
            actif = Some(voulu.clone());
        }
        let t = maintenant();
        if regles.journal_s > 0 && t.saturating_sub(dernier_journal) >= regles.journal_s {
            if let Err(e) = journal::ajouter(&fichier_journal, t, &voulu, ecran, &batterie) {
                note(&format!("journal : {e}"));
            }
            dernier_journal = t;
        }
        std::thread::sleep(Duration::from_secs(regles.intervalle_s));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn batterie(statut: &str, capacite: i64) -> Batterie {
        Batterie {
            statut: statut.into(),
            capacite: Some(capacite),
            ..Batterie::default()
        }
    }

    #[test]
    fn choisit_selon_les_regles() {
        let r = Regles::depuis_texte(
            "[regles]\ndefaut = economie\necran_eteint = nuit\nen_charge = origine\nbatterie_faible = survie\nseuil_batterie_faible = 15\n",
        )
        .unwrap();
        assert_eq!(
            r.choisir(&batterie("Discharging", 80), Ecran::Allume),
            "economie"
        );
        assert_eq!(
            r.choisir(&batterie("Discharging", 80), Ecran::Eteint),
            "nuit"
        );
        assert_eq!(
            r.choisir(&batterie("Discharging", 10), Ecran::Allume),
            "survie"
        );
        assert_eq!(
            r.choisir(&batterie("Discharging", 10), Ecran::Eteint),
            "nuit"
        );
        assert_eq!(
            r.choisir(&batterie("Charging", 10), Ecran::Eteint),
            "origine"
        );
        assert_eq!(
            r.choisir(&batterie("Not charging", 80), Ecran::Allume),
            "origine"
        );
    }

    #[test]
    fn regles_par_defaut_et_erreurs() {
        let r = Regles::depuis_texte("").unwrap();
        assert_eq!(
            r.choisir(&batterie("Discharging", 5), Ecran::Eteint),
            "origine"
        );
        assert!(Regles::depuis_texte("[regles]\nautre = x\n").is_err());
        assert!(Regles::depuis_texte("[regles]\ndefaut = aucun\n").is_err());
        assert!(Regles::depuis_texte("[demon]\nintervalle_s = vite\n").is_err());
    }

    #[test]
    fn les_reglages_livres_sont_valides() {
        let dossier = Path::new(env!("CARGO_MANIFEST_DIR")).join("reglages");
        let regles = Regles::charger(&dossier).unwrap();
        for nom in profil::liste(&dossier) {
            Profil::charger(&dossier, &nom).unwrap();
        }
        for nom in regles.profils() {
            Profil::charger(&dossier, nom).unwrap();
        }
    }

    #[test]
    fn l_integration_android_est_coherente() {
        let racine = Path::new(env!("CARGO_MANIFEST_DIR"));
        let lire = |f: &str| std::fs::read_to_string(racine.join(f)).unwrap();
        let (bp, rc, contextes) = (
            lire("Android.bp"),
            lire("energie.rc"),
            lire("sepolicy/file_contexts"),
        );
        // Chaque profil livré est installé par AOSP.
        for nom in profil::liste(&racine.join("reglages")).iter().skip(1) {
            assert!(
                bp.contains(&format!("\"reglages/profils/{nom}.conf\"")),
                "{nom} manque dans Android.bp"
            );
        }
        // Le binaire lancé par init est celui que la politique SELinux étiquette.
        assert!(
            rc.contains("service customfirmware-energie /system_ext/bin/customfirmware-energie ")
        );
        assert!(contextes.contains("/bin/customfirmware-energie "));
        assert!(bp.contains("name: \"customfirmware-energie\""));
    }
}
