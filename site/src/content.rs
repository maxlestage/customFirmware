//! Le contenu du site : les faits sur l'appareil, le périmètre Rust, la feuille de
//! route et les sources. Modifier ce fichier suffit pour mettre le site à jour.

/// Date de la dernière vérification des faits ci-dessous.
pub const VERIFIED_ON: &str = "26 novembre 2026";

/// Une ligne du tableau « L'appareil ».
pub struct Fact {
    pub label: &'static str,
    pub value: &'static str,
    /// Une précision ou une réserve, affichée sous la valeur.
    pub note: Option<&'static str>,
}

pub const FACTS: &[Fact] = &[
    Fact {
        label: "Modèle",
        value: "Nothing Phone (4b)",
        note: Some("Sorti en juillet 2026."),
    },
    Fact {
        label: "Processeur",
        value: "Qualcomm Snapdragon 6 Gen 4 (SM6650), GPU Adreno 810",
        note: None,
    },
    Fact {
        label: "Système d'origine",
        value: "Nothing OS 4.1, sur Android 16",
        note: Some(
            "Dernière version connue : B4.1-261111-1740, correctif de sécurité de novembre 2026. Android 17 est annoncé, sans date officielle.",
        ),
    },
    Fact {
        label: "Noyau",
        value: "Linux 6.1, sources publiées par Nothing",
        note: Some("Dépôt NothingOSS/android_kernel_msm-6.1_nothing_sm6650."),
    },
    Fact {
        label: "Nom de code",
        value: "SuperContra (modèle A009P)",
        note: Some(
            "Le nom des mises à jour officielles de Nothing OS, par exemple SuperContra-B4.1-261111-1740.",
        ),
    },
    Fact {
        label: "Bootloader",
        value: "Verrouillé sur le téléphone de développement",
        note: Some("La procédure de déverrouillage reste à confirmer pour ce modèle."),
    },
    Fact {
        label: "Portages existants",
        value: "Aucun connu",
        note: Some("Des portages existent pour les Phone (4a) et (4a) Pro, sur d'autres puces."),
    },
];

/// Les mots qui défilent sous l'introduction.
pub const TICKER: &[&str] = &[
    "Snapdragon 6 Gen 4",
    "SM6650",
    "Adreno 810",
    "Linux 6.1",
    "Android 16",
    "AOSP",
    "Rust",
    "AIDL",
    "SuperContra",
    "Nothing OS 4.1",
];

/// Un élément des deux listes « en Rust » et « reste tel quel ».
pub struct Part {
    pub title: &'static str,
    pub detail: &'static str,
}

pub const IN_RUST: &[Part] = &[
    Part {
        title: "Couches matérielles (HAL)",
        detail: "Les services AIDL propres à l'appareil : lumières, vibreur, gestion d'énergie, capteurs simples.",
    },
    Part {
        title: "Services et outils système",
        detail: "Les programmes que le portage ajoute au système, compilés par AOSP comme modules Rust.",
    },
    Part {
        title: "Outils sur l'ordinateur",
        detail: "Extraction des binaires du fabricant, génération de la configuration, installation.",
    },
    Part {
        title: "Ce site",
        detail: "Une application Paquets : serveur, pages et sécurité en Rust.",
    },
];

pub const NOT_IN_RUST: &[Part] = &[
    Part {
        title: "Android (AOSP) lui-même",
        detail: "Des millions de lignes en C++, Java et Kotlin. Le projet les compile, il ne les réécrit pas.",
    },
    Part {
        title: "Le noyau Linux 6.1",
        detail: "En C. Le premier noyau Android avec Rust activé est le 6.12 : pas de pilote Rust sur cet appareil.",
    },
    Part {
        title: "Les binaires Qualcomm",
        detail: "Modem, GPU, caméra, zone sécurisée : fermés et signés. Ils sont repris du système d'origine.",
    },
    Part {
        title: "Le bootloader",
        detail: "Signé par le fabricant. Il peut être déverrouillé, pas remplacé.",
    },
];

/// L'état d'une étape de la feuille de route.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Done,
    Next,
    Todo,
}

impl Status {
    pub fn label(self) -> &'static str {
        match self {
            Status::Done => "Fait",
            Status::Next => "Prochaine étape",
            Status::Todo => "À faire",
        }
    }

    /// La classe CSS de la pastille.
    pub fn class(self) -> &'static str {
        match self {
            Status::Done => "pill done",
            Status::Next => "pill next",
            Status::Todo => "pill todo",
        }
    }
}

pub struct Step {
    pub title: &'static str,
    pub detail: &'static str,
    pub status: Status,
}

pub const ROADMAP: &[Step] = &[
    Step {
        title: "Identifier l'appareil et l'existant",
        detail: "Puce, version du noyau, sources publiées par Nothing, portages déjà faits.",
        status: Status::Done,
    },
    Step {
        title: "Présenter le projet",
        detail: "Ce site, écrit en Rust avec Paquets.",
        status: Status::Done,
    },
    Step {
        title: "Écrire le guide",
        detail: "Toutes les étapes, de la sauvegarde à l'installation, avec leurs commandes et leurs sources.",
        status: Status::Done,
    },
    Step {
        title: "Déverrouiller le bootloader",
        detail: "Sans cela, rien ne peut être installé. L'opération efface le téléphone.",
        status: Status::Next,
    },
    Step {
        title: "Sauvegarder le système d'origine",
        detail: "Une copie complète du firmware de Nothing, pour pouvoir revenir en arrière.",
        status: Status::Todo,
    },
    Step {
        title: "Extraire les binaires du fabricant",
        detail: "Un outil Rust dresse la liste des fichiers propriétaires et les copie depuis le système d'origine.",
        status: Status::Todo,
    },
    Step {
        title: "Écrire la configuration de l'appareil",
        detail: "Le « device tree » SuperContra : partitions, démarrage, règles de sécurité.",
        status: Status::Todo,
    },
    Step {
        title: "Compiler le noyau",
        detail: "Depuis les sources de Nothing, sans modification pour commencer.",
        status: Status::Todo,
    },
    Step {
        title: "Écrire la première couche matérielle en Rust",
        detail: "Le vibreur ou les lumières : un service court, facile à vérifier sur le téléphone.",
        status: Status::Todo,
    },
    Step {
        title: "Démarrer AOSP sur le téléphone",
        detail: "Écran, tactile, Wi-Fi, appels : chaque fonction est vérifiée une par une.",
        status: Status::Todo,
    },
];

pub const RISKS: &[&str] = &[
    "Déverrouiller le bootloader efface toutes les données du téléphone et peut annuler la garantie.",
    "Une installation ratée peut rendre le téléphone inutilisable. Une sauvegarde du système d'origine est faite avant toute écriture.",
    "Un système signé avec des clés de test ne bénéficie pas du démarrage vérifié d'Android.",
];

/// Une source citée en bas de page.
pub struct Reference {
    pub title: &'static str,
    pub url: &'static str,
}

pub const REFERENCES: &[Reference] = &[
    Reference {
        title: "GSMArena : fiche du Nothing Phone (4b)",
        url: "https://www.gsmarena.com/nothing_phone_(4b)_5g-14781.php",
    },
    Reference {
        title: "NothingOSS : sources du noyau du Phone (4b)",
        url: "https://github.com/NothingOSS/android_kernel_msm-6.1_nothing_sm6650",
    },
    Reference {
        title: "Android Central : le Phone (4b) et ses mises à jour",
        url: "https://www.androidcentral.com/phones/nothing-phones/the-nothing-phone-4b-is-a-flashy-budget-phone-with-flagship-software-promises",
    },
    Reference {
        title: "Nothing Archive : appareils et firmware Nothing",
        url: "https://github.com/spike0en/nothing_archive/blob/main/website/docs/devices.md",
    },
    Reference {
        title: "Google : Rust dans Android",
        url: "https://blog.google/security/rust-in-android-move-fast-fix-things/",
    },
    Reference {
        title: "AOSP : compiler des modules Rust",
        url: "https://source.android.com/docs/setup/build/rust/building-rust-modules/overview",
    },
    Reference {
        title: "Phoronix : le pilote Binder en Rust dans Linux 6.18",
        url: "https://phoronix.com/news/Rust-Binder-For-Linux-6.18",
    },
];
