//! Ce que le noyau dit de la batterie et de l'écran.

use crate::systeme::Systeme;

#[derive(Debug, Default, Clone)]
pub struct Batterie {
    /// En %.
    pub capacite: Option<i64>,
    /// `Charging`, `Discharging`, `Full`, `Not charging`…
    pub statut: String,
    /// En microampères. Le signe dépend du pilote : on ne s'en sert pas.
    pub courant_ua: Option<i64>,
    /// En microvolts.
    pub tension_uv: Option<i64>,
    /// En dixièmes de degré.
    pub temperature: Option<i64>,
}

impl Batterie {
    pub fn lire(sys: &Systeme, dossier: &str) -> Batterie {
        Batterie {
            capacite: sys.nombre(&format!("{dossier}/capacity")),
            statut: sys.lire(&format!("{dossier}/status")).unwrap_or_default(),
            courant_ua: sys.nombre(&format!("{dossier}/current_now")),
            tension_uv: sys.nombre(&format!("{dossier}/voltage_now")),
            temperature: sys.nombre(&format!("{dossier}/temp")),
        }
    }

    /// Branché sur un chargeur (en charge, pleine, ou charge en pause).
    pub fn branche(&self) -> bool {
        !self.statut.is_empty() && self.statut != "Discharging" && self.statut != "Unknown"
    }

    /// La puissance tirée de la batterie, en milliwatts.
    pub fn puissance_mw(&self) -> Option<i64> {
        Some(puissance_mw(self.courant_ua?, self.tension_uv?))
    }
}

pub fn puissance_mw(courant_ua: i64, tension_uv: i64) -> i64 {
    // µA × µV = pW ; 10⁹ pW = 1 mW.
    (courant_ua.abs() as i128 * tension_uv.abs() as i128 / 1_000_000_000) as i64
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ecran {
    Allume,
    Eteint,
    Inconnu,
}

impl Ecran {
    /// L'écran est éteint quand tous les rétroéclairages trouvés sont à 0.
    pub fn lire(sys: &Systeme, motif: &str) -> Ecran {
        let valeurs: Vec<i64> = sys
            .developper(motif)
            .iter()
            .filter_map(|c| sys.nombre(c))
            .collect();
        if valeurs.is_empty() {
            Ecran::Inconnu
        } else if valeurs.iter().all(|&v| v == 0) {
            Ecran::Eteint
        } else {
            Ecran::Allume
        }
    }

    pub fn nom(self) -> &'static str {
        match self {
            Ecran::Allume => "allume",
            Ecran::Eteint => "eteint",
            Ecran::Inconnu => "inconnu",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systeme::tests::FauxArbre;

    #[test]
    fn batterie_et_ecran() {
        let arbre = FauxArbre::nouveau("capteurs");
        arbre
            .fichier("/sys/class/power_supply/battery/capacity", "76\n")
            .fichier("/sys/class/power_supply/battery/status", "Discharging\n")
            .fichier("/sys/class/power_supply/battery/current_now", "-412000\n")
            .fichier("/sys/class/power_supply/battery/voltage_now", "3910000\n")
            .fichier("/sys/class/backlight/panel0-backlight/brightness", "0\n");
        let sys = arbre.systeme();
        let b = Batterie::lire(&sys, "/sys/class/power_supply/battery");
        assert_eq!(b.capacite, Some(76));
        assert!(!b.branche());
        assert_eq!(b.puissance_mw(), Some(1610));
        assert_eq!(b.temperature, None);
        assert_eq!(
            Ecran::lire(&sys, "/sys/class/backlight/*/brightness"),
            Ecran::Eteint
        );
        arbre.fichier("/sys/class/backlight/panel0-backlight/brightness", "800");
        assert_eq!(
            Ecran::lire(&sys, "/sys/class/backlight/*/brightness"),
            Ecran::Allume
        );
        assert_eq!(Ecran::lire(&sys, "/sys/class/absent/*"), Ecran::Inconnu);
    }
}
