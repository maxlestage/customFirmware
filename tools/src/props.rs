//! `cf props` : lit la sortie de `adb shell getprop` et en tire les faits dont le
//! portage a besoin, sous forme de rapport et de fragment Make (`facts.mk`).

use std::collections::BTreeMap;

/// Les propriétés du téléphone, telles que `getprop` les affiche : `[clé]: [valeur]`.
#[derive(Debug, Default)]
pub struct Props(BTreeMap<String, String>);

impl Props {
    /// Lit la sortie de `getprop`. Les lignes qui n'ont pas la forme attendue (suite
    /// d'une valeur sur plusieurs lignes, ligne vide) sont ignorées.
    pub fn parse(text: &str) -> Self {
        let mut map = BTreeMap::new();
        for line in text.lines() {
            let Some(rest) = line.trim().strip_prefix('[') else {
                continue;
            };
            let Some((key, value)) = rest.split_once("]: [") else {
                continue;
            };
            let Some(value) = value.strip_suffix(']') else {
                continue;
            };
            map.insert(key.to_owned(), value.to_owned());
        }
        Props(map)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// La première des clés données qui existe et n'est pas vide.
    pub fn first(&self, keys: &[&str]) -> Option<&str> {
        keys.iter()
            .filter_map(|key| self.0.get(*key))
            .map(String::as_str)
            .find(|value| !value.is_empty())
    }
}

/// Un fait relevé sur le téléphone.
pub struct Fact {
    /// Le nom affiché dans le rapport.
    pub label: &'static str,
    /// Les propriétés où le chercher, par ordre de préférence.
    pub keys: &'static [&'static str],
    /// La variable Make écrite dans `facts.mk`, s'il y en a une.
    pub make_var: Option<&'static str>,
    /// À quoi sert ce fait dans le portage.
    pub why: &'static str,
}

pub const FACTS: &[Fact] = &[
    Fact {
        label: "Appareil",
        keys: &["ro.product.device", "ro.product.vendor.device"],
        make_var: Some("CF_STOCK_DEVICE"),
        why: "le nom de code, à comparer avec SuperContra",
    },
    Fact {
        label: "Modèle",
        keys: &["ro.product.model", "ro.product.vendor.model"],
        make_var: Some("CF_STOCK_MODEL"),
        why: "le nom commercial du modèle",
    },
    Fact {
        label: "Version du système",
        keys: &["ro.build.display.id"],
        make_var: None,
        why: "la version exacte du firmware à télécharger (étape 03)",
    },
    Fact {
        label: "Empreinte",
        keys: &["ro.build.fingerprint", "ro.vendor.build.fingerprint"],
        make_var: Some("CF_STOCK_FINGERPRINT"),
        why: "identifie le build d'origine",
    },
    Fact {
        label: "Android",
        keys: &["ro.build.version.release"],
        make_var: None,
        why: "la version d'Android d'origine",
    },
    Fact {
        label: "Correctif de sécurité",
        keys: &["ro.build.version.security_patch"],
        make_var: Some("CF_STOCK_SECURITY_PATCH"),
        why: "un système plus ancien que ce correctif peut être refusé",
    },
    Fact {
        label: "Plateforme",
        keys: &["ro.board.platform"],
        make_var: Some("CF_BOARD_PLATFORM"),
        why: "TARGET_BOARD_PLATFORM dans BoardConfig.mk",
    },
    Fact {
        label: "Niveau d'API de lancement",
        keys: &["ro.product.first_api_level"],
        make_var: Some("CF_SHIPPING_API_LEVEL"),
        why: "PRODUCT_SHIPPING_API_LEVEL dans device.mk",
    },
    Fact {
        label: "Niveau d'API du vendor",
        keys: &["ro.board.api_level", "ro.vendor.api_level"],
        make_var: Some("CF_BOARD_API_LEVEL"),
        why: "la version de l'interface entre le système et les pilotes",
    },
    Fact {
        label: "Architectures",
        keys: &["ro.product.cpu.abilist"],
        make_var: None,
        why: "arm64-v8a seul : un système 64 bits uniquement",
    },
    Fact {
        label: "Treble",
        keys: &["ro.treble.enabled"],
        make_var: None,
        why: "doit valoir true pour démarrer une image système générique (GSI)",
    },
    Fact {
        label: "Partitions dynamiques",
        keys: &["ro.boot.dynamic_partitions"],
        make_var: Some("CF_DYNAMIC_PARTITIONS"),
        why: "system, vendor… vivent dans la partition super",
    },
    Fact {
        label: "Mises à jour A/B",
        keys: &["ro.build.ab_update"],
        make_var: Some("CF_AB_UPDATE"),
        why: "deux emplacements (a et b) pour les partitions de démarrage",
    },
    Fact {
        label: "A/B virtuel",
        keys: &["ro.virtual_ab.enabled"],
        make_var: Some("CF_VIRTUAL_AB"),
        why: "les partitions dynamiques sont mises à jour par instantanés",
    },
    Fact {
        label: "Emplacement actif",
        keys: &["ro.boot.slot_suffix"],
        make_var: None,
        why: "l'emplacement sur lequel le téléphone a démarré",
    },
    Fact {
        label: "Démarrage vérifié",
        keys: &["ro.boot.verifiedbootstate"],
        make_var: None,
        why: "green : bootloader verrouillé ; orange : déverrouillé",
    },
];

/// Le rapport lisible : une ligne par fait, puis ceux qui manquent.
pub fn report(props: &Props) -> String {
    let mut out = format!("{} propriétés lues.\n\n", props.len());
    let width = FACTS
        .iter()
        .map(|f| f.label.chars().count())
        .max()
        .unwrap_or(0);
    for fact in FACTS {
        let value = props.first(fact.keys).unwrap_or("(absent)");
        let pad = " ".repeat(width - fact.label.chars().count());
        out.push_str(&format!("  {}{pad}  {value}\n", fact.label));
        out.push_str(&format!("  {}  └ {}\n", " ".repeat(width), fact.why));
    }
    out
}

/// Le fragment Make : une variable `CF_…` par fait trouvé. Les valeurs viennent du
/// téléphone ; un caractère qui aurait un sens pour Make (`$`, `#`, retour à la ligne)
/// fait rejeter la valeur.
pub fn facts_mk(props: &Props) -> String {
    let mut out = String::from(
        "# Généré par `cf props` depuis la sortie de `adb shell getprop`.\n\
         # Ne pas modifier à la main : relancer la commande.\n\n",
    );
    for fact in FACTS {
        let Some(var) = fact.make_var else { continue };
        match props.first(fact.keys) {
            Some(value) if is_make_safe(value) => out.push_str(&format!("{var} := {value}\n")),
            Some(_) => out.push_str(&format!("# {var} : valeur refusée (caractères spéciaux)\n")),
            None => out.push_str(&format!("# {var} : absent de getprop ({})\n", fact.keys[0])),
        }
    }
    out
}

fn is_make_safe(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| !c.is_control() && !matches!(c, '$' | '#' | '\\'))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
[ro.board.platform]: [exemple]
[ro.product.device]: [SuperContra]
[ro.product.first_api_level]: [36]
[ro.build.fingerprint]: [Nothing/SuperContra/SuperContra:16/BP2A/1:user/release-keys]
[ro.virtual_ab.enabled]: [true]
[ro.empty]: []
une ligne qui n'est pas une propriété
[ro.evil]: [$(shell rm -rf /)]
";

    #[test]
    fn parses_getprop_lines() {
        let props = Props::parse(SAMPLE);
        assert_eq!(props.len(), 7);
        assert_eq!(props.first(&["ro.board.platform"]), Some("exemple"));
        assert_eq!(
            props.first(&["ro.missing", "ro.product.device"]),
            Some("SuperContra")
        );
        // Une valeur vide compte comme absente.
        assert_eq!(props.first(&["ro.empty"]), None);
    }

    #[test]
    fn writes_make_variables_for_known_facts() {
        let mk = facts_mk(&Props::parse(SAMPLE));
        assert!(mk.contains("CF_BOARD_PLATFORM := exemple\n"));
        assert!(mk.contains("CF_SHIPPING_API_LEVEL := 36\n"));
        assert!(mk.contains("CF_VIRTUAL_AB := true\n"));
        assert!(mk.contains("# CF_AB_UPDATE : absent de getprop (ro.build.ab_update)\n"));
    }

    #[test]
    fn refuses_values_that_make_would_interpret() {
        assert!(!is_make_safe("$(shell rm -rf /)"));
        assert!(!is_make_safe("a # b"));
        assert!(!is_make_safe(""));
        assert!(is_make_safe(
            "Nothing/SuperContra/SuperContra:16/BP2A/1:user/release-keys"
        ));
        let props = Props::parse("[ro.board.platform]: [$(shell id)]\n");
        let mk = facts_mk(&props);
        assert!(!mk.contains("$(shell"));
        assert!(mk.contains("# CF_BOARD_PLATFORM : valeur refusée"));
    }

    #[test]
    fn report_lists_every_fact() {
        let text = report(&Props::parse(SAMPLE));
        for fact in FACTS {
            assert!(text.contains(fact.label), "{}", fact.label);
        }
        assert!(text.contains("(absent)"));
    }
}
