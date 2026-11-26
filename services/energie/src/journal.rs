//! Le journal de consommation et son rapport : ce que chaque profil coûte vraiment.
//!
//! Le démon ajoute une ligne CSV par mesure. Le rapport calcule, pour chaque profil et
//! chaque état de l'écran, la puissance moyenne et la vitesse de décharge.

use crate::capteurs::{self, Batterie, Ecran};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::Path;

pub const ENTETE: &str =
    "horodatage,profil,ecran,statut,capacite,courant_ua,tension_uv,temperature";

/// Au-delà, le journal est renommé en `.1` et un nouveau commence.
const TAILLE_MAX: u64 = 4 * 1024 * 1024;

pub fn ajouter(
    fichier: &Path,
    horodatage: u64,
    profil: &str,
    ecran: Ecran,
    batterie: &Batterie,
) -> std::io::Result<()> {
    if fs::metadata(fichier)
        .map(|m| m.len() > TAILLE_MAX)
        .unwrap_or(false)
    {
        fs::rename(fichier, fichier.with_extension("csv.1"))?;
    }
    let nouveau = !fichier.exists();
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(fichier)?;
    if nouveau {
        writeln!(f, "{ENTETE}")?;
    }
    let nombre = |n: Option<i64>| n.map(|n| n.to_string()).unwrap_or_default();
    writeln!(
        f,
        "{horodatage},{profil},{},{},{},{},{},{}",
        ecran.nom(),
        batterie.statut.replace(',', " "),
        nombre(batterie.capacite),
        nombre(batterie.courant_ua),
        nombre(batterie.tension_uv),
        nombre(batterie.temperature),
    )
}

struct Mesure {
    temps: u64,
    etat: String,
    decharge: bool,
    capacite: Option<i64>,
    puissance: Option<i64>,
}

fn lire_mesure(ligne: &str) -> Option<Mesure> {
    let champs: Vec<&str> = ligne.split(',').collect();
    if champs.len() < 7 {
        return None;
    }
    let nombre = |i: usize| champs.get(i).and_then(|c| c.parse::<i64>().ok());
    Some(Mesure {
        temps: champs[0].parse().ok()?,
        etat: format!("{} · écran {}", champs[1], champs[2]),
        decharge: champs[3] == "Discharging",
        capacite: nombre(4),
        puissance: Some(capteurs::puissance_mw(nombre(5)?, nombre(6)?)),
    })
}

#[derive(Default)]
struct Totaux {
    secondes: u64,
    pourcents: i64,
    puissance_somme: i64,
    puissance_n: i64,
}

/// Le rapport, prêt à afficher.
pub fn rapport(texte: &str) -> String {
    let mesures: Vec<Mesure> = texte.lines().filter_map(lire_mesure).collect();
    let mut totaux: BTreeMap<String, Totaux> = BTreeMap::new();
    for m in mesures.iter().filter(|m| m.decharge) {
        if let Some(p) = m.puissance {
            let t = totaux.entry(m.etat.clone()).or_default();
            t.puissance_somme += p;
            t.puissance_n += 1;
        }
    }
    // Entre deux mesures sur batterie dans le même état, le temps passé et les % perdus
    // comptent pour cet état. Le temps en veille compte aussi : c'est du temps réel.
    for paire in mesures.windows(2) {
        let (a, b) = (&paire[0], &paire[1]);
        if !(a.decharge && b.decharge && a.etat == b.etat) || b.temps <= a.temps {
            continue;
        }
        if let (Some(ca), Some(cb)) = (a.capacite, b.capacite) {
            if cb <= ca {
                let t = totaux.entry(a.etat.clone()).or_default();
                t.secondes += b.temps - a.temps;
                t.pourcents += ca - cb;
            }
        }
    }
    if totaux.is_empty() {
        return "Pas encore de mesure sur batterie dans le journal.".into();
    }
    let mut sortie = format!(
        "{:<36} {:>9} {:>8} {:>10} {:>11}\n",
        "profil · écran", "mesuré", "%/heure", "puissance", "autonomie"
    );
    for (etat, t) in &totaux {
        let heures = t.secondes as f64 / 3600.0;
        let vitesse = (heures > 0.0).then(|| t.pourcents as f64 / heures);
        let puissance = if t.puissance_n > 0 {
            format!("{} mW", t.puissance_somme / t.puissance_n)
        } else {
            "?".into()
        };
        let (vitesse_txt, autonomie) = match vitesse {
            // Moins d'une heure ou moins de 2 % perdus : trop peu pour conclure.
            Some(v) if heures >= 1.0 && t.pourcents >= 2 => {
                (format!("{v:.1}"), format!("{:.0} h", 100.0 / v))
            }
            _ => ("?".into(), "?".into()),
        };
        sortie.push_str(&format!(
            "{etat:<36} {:>7.1} h {vitesse_txt:>8} {puissance:>10} {autonomie:>11}\n",
            heures
        ));
    }
    sortie.push_str(
        "\nautonomie : de 100 % à 0 % en restant dans cet état. « ? » : pas assez de mesures \
         (il faut au moins 1 h et 2 % perdus).",
    );
    sortie
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ecrit_puis_resume_le_journal() {
        let dossier = std::env::temp_dir().join(format!("energie-journal-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dossier);
        fs::create_dir_all(&dossier).unwrap();
        let fichier = dossier.join("journal.csv");
        let mut b = Batterie {
            statut: "Discharging".into(),
            courant_ua: Some(-250_000),
            tension_uv: Some(4_000_000),
            ..Batterie::default()
        };
        // Écran éteint : 8 heures, de 90 % à 86 %.
        for (h, c) in [(0, 90), (4, 88), (8, 86)] {
            b.capacite = Some(c);
            ajouter(&fichier, 1_000 + h * 3600, "nuit", Ecran::Eteint, &b).unwrap();
        }
        // Branché : ne compte pas.
        b.statut = "Charging".into();
        ajouter(&fichier, 40_000, "origine", Ecran::Allume, &b).unwrap();

        let texte = fs::read_to_string(&fichier).unwrap();
        assert!(texte.starts_with(ENTETE));
        let r = rapport(&texte);
        let ligne = r
            .lines()
            .find(|l| l.starts_with("nuit · écran eteint"))
            .unwrap();
        assert!(ligne.contains("8.0 h"), "{r}");
        assert!(ligne.contains("0.5"), "{r}");
        assert!(ligne.contains("1000 mW"), "{r}");
        assert!(ligne.contains("200 h"), "{r}");
        assert!(!r.contains("origine"));
        fs::remove_dir_all(&dossier).unwrap();
    }

    #[test]
    fn journal_vide() {
        assert!(rapport(ENTETE).starts_with("Pas encore"));
    }
}
