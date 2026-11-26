//! Le contenu du guide. Chaque étape est une suite de blocs : texte, listes,
//! commandes, fichiers, mises en garde. `guide_page.rs` les met en page.
//!
//! Les fichiers du dépôt montrés dans le guide sont inclus par `include_str!` :
//! le guide ne peut pas s'écarter du code.

/// Date de la dernière vérification des commandes.
pub const GUIDE_UPDATED: &str = "26 novembre 2026";

pub const INTRO: &str = "Ce guide décrit, dans l'ordre, tout le chemin entre un Nothing Phone (4b) d'origine et un système AOSP compilé par vous, avec des services écrits en Rust. Il avance en deux temps : d'abord un système AOSP qui garde le noyau et les pilotes d'origine (étapes 05 à 08), puis le remplacement de ces pilotes (étapes 09 à 11). Chaque étape dit d'où viennent ses commandes et si elles ont déjà été essayées.";

/// Ce que l'on sait d'une étape.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Verified {
    /// Les commandes viennent d'une documentation officielle ou reconnue.
    Yes,
    /// La méthode est connue, mais personne ne l'a encore publiée pour le 4b.
    Partly,
    /// Le code est dans le dépôt, mais n'a encore jamais été compilé dans AOSP.
    Untested,
    /// Le projet doit encore écrire cette partie : le code est un point de départ.
    No,
}

impl Verified {
    pub const ALL: [Verified; 4] = [
        Verified::Yes,
        Verified::Partly,
        Verified::Untested,
        Verified::No,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Verified::Yes => "Sources officielles",
            Verified::Partly => "À confirmer sur le 4b",
            Verified::Untested => "Écrit, pas encore compilé",
            Verified::No => "À écrire par le projet",
        }
    }

    pub fn meaning(self) -> &'static str {
        match self {
            Verified::Yes => {
                "Les commandes viennent de la documentation d'AOSP ou de Nothing Archive."
            }
            Verified::Partly => {
                "La méthode marche sur des appareils proches ; elle n'a pas encore été essayée sur le Phone (4b)."
            }
            Verified::Untested => {
                "Le code est dans le dépôt et suit les exemples d'AOSP, mais n'a encore jamais été compilé ni essayé."
            }
            Verified::No => {
                "Le projet doit encore écrire cette partie ; le code montré est un point de départ."
            }
        }
    }

    pub fn class(self) -> &'static str {
        match self {
            Verified::Yes => "badge yes",
            Verified::Partly => "badge partly",
            Verified::Untested => "badge untested",
            Verified::No => "badge no",
        }
    }
}

/// Un morceau d'étape.
pub enum Block {
    Text(&'static str),
    /// Une liste à puces.
    List(&'static [&'static str]),
    /// Une liste numérotée d'actions à faire sur le téléphone.
    Steps(&'static [&'static str]),
    /// Des commandes à taper dans un terminal.
    Code {
        title: &'static str,
        code: &'static str,
    },
    /// Le contenu d'un fichier.
    File {
        path: &'static str,
        code: &'static str,
    },
    Warning(&'static str),
    Tip(&'static str),
    /// Comment savoir que l'étape a réussi.
    Check(&'static str),
    Source {
        title: &'static str,
        url: &'static str,
    },
}

pub struct Step {
    /// L'ancre de l'étape dans la page.
    pub id: &'static str,
    pub title: &'static str,
    pub goal: &'static str,
    /// Une estimation de la durée.
    pub time: &'static str,
    pub verified: Verified,
    pub blocks: &'static [Block],
}

use Block::*;

pub const STEPS: &[Step] = &[
    Step {
        id: "avant",
        title: "Avant de commencer",
        goal: "Savoir ce que l'on risque et réunir le matériel.",
        time: "10 minutes de lecture",
        verified: Verified::Yes,
        blocks: &[
            Text(
                "Rien de ce guide n'est réversible sans effort. Lisez-le une fois en entier avant de taper la première commande.",
            ),
            List(&[
                "Un Nothing Phone (4b), modèle A009P, nom de code SuperContra.",
                "Un câble USB-C qui transporte les données, pas seulement la charge.",
                "Un ordinateur avec un port USB, pour parler au téléphone : n'importe quel ordinateur suffit jusqu'à l'étape 05.",
                "Pour compiler AOSP (à partir de l'étape 06) : un ordinateur Linux 64 bits x86, avec au moins 64 Go de mémoire et 400 Go d'espace libre (250 Go pour les sources, 150 Go pour la compilation). Un serveur loué à l'heure convient.",
            ]),
            Warning(
                "Déverrouiller le bootloader efface toutes les données du téléphone et peut annuler la garantie. Sur les puces Qualcomm, le niveau de protection Widevine passe de L1 à L3 (vidéo en définition réduite dans certaines applications), et les applications qui vérifient l'intégrité de l'appareil (banques, paiement) peuvent refuser de fonctionner.",
            ),
            Warning(
                "Une erreur d'installation peut rendre le téléphone inutilisable. Gardez toujours de côté le firmware d'origine (étape 03) et ne reverrouillez jamais le bootloader sur un système modifié.",
            ),
            Source {
                title: "AOSP : configuration requise",
                url: "https://source.android.com/docs/setup/start/requirements",
            },
        ],
    },
    Step {
        id: "ordinateur",
        title: "Préparer l'ordinateur",
        goal: "Installer adb et fastboot, Rust et l'outil cf du projet ; et, pour compiler, les paquets d'AOSP.",
        time: "20 minutes",
        verified: Verified::Yes,
        blocks: &[
            Text(
                "Les commandes sont pour Ubuntu (18.04 ou plus récent), la distribution que la documentation d'AOSP prend pour exemple.",
            ),
            Code {
                title: "adb et fastboot (platform-tools de Google)",
                code: r#"
curl -LO https://dl.google.com/android/repository/platform-tools-latest-linux.zip
unzip platform-tools-latest-linux.zip -d ~
echo 'export PATH="$HOME/platform-tools:$PATH"' >> ~/.bashrc
source ~/.bashrc
adb --version && fastboot --version
"#,
            },
            Code {
                title: "Rust, puis cf, l'outil du projet",
                code: r#"
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://github.com/maxlestage/customFirmware.git ~/customFirmware
cargo install --path ~/customFirmware/tools
cf doctor ~          # vérifie le système, la mémoire, le disque et les outils
"#,
            },
            Code {
                title: "Pour compiler AOSP : ses paquets et repo",
                code: r#"
sudo apt-get update
sudo apt-get install git-core gnupg flex bison build-essential zip curl \
  zlib1g-dev libc6-dev-i386 x11proto-core-dev libx11-dev lib32z1-dev \
  libgl1-mesa-dev libxml2-utils xsltproc unzip fontconfig
sudo apt-get install repo
repo version        # 2.4 ou plus récent
"#,
            },
            Check(
                "cf doctor affiche une coche devant chaque ligne. Sans les 64 Go de mémoire et les 400 Go de disque, vous pouvez quand même suivre le guide jusqu'à l'étape 05.",
            ),
            Tip(
                "Si adb ne voit pas le téléphone sans sudo, installez les règles udev d'Android : sudo apt-get install android-sdk-platform-tools-common, puis débranchez et rebranchez le câble.",
            ),
            Source {
                title: "AOSP : télécharger les sources (installation de repo)",
                url: "https://source.android.com/docs/setup/download",
            },
        ],
    },
    Step {
        id: "informations",
        title: "Relever les informations du téléphone",
        goal: "Noter la version exacte du système et la configuration de l'appareil, avant d'y toucher.",
        time: "10 minutes",
        verified: Verified::Yes,
        blocks: &[
            Steps(&[
                "Ouvrez Paramètres → À propos du téléphone et appuyez sept fois sur « Numéro de build » : les options pour les développeurs apparaissent.",
                "Dans Paramètres → Système → Options pour les développeurs, activez « Débogage USB ».",
                "Branchez le téléphone et acceptez la demande d'autorisation qui s'affiche.",
            ]),
            Code {
                title: "Relever les propriétés, puis les lire avec cf",
                code: r#"
adb devices                                   # le téléphone doit être listé « device »
mkdir -p ~/supercontra && cd ~/supercontra
adb shell getprop > getprop.txt
cf props getprop.txt
"#,
            },
            Text(
                "cf props affiche ce dont le portage a besoin : la version exacte du système, la plateforme Qualcomm, le niveau d'API de lancement, la compatibilité Treble, les partitions dynamiques et l'état du démarrage vérifié.",
            ),
            Check(
                "La ligne « Version du système » donne une version de la forme SuperContra-B4.1-…, et la ligne « Treble » vaut true.",
            ),
            Tip(
                "Notez aussi la table des partitions : adb shell ls /vendor/etc | grep -i fstab, puis adb pull /vendor/etc/<nom du fichier> .",
            ),
            Source {
                title: "Nothing Archive : activer les options pour les développeurs",
                url: "https://nothingarchive.tech/docs/guides#unlocking-bootloader",
            },
        ],
    },
    Step {
        id: "sauvegarde",
        title: "Sauvegarder le système d'origine",
        goal: "Avoir sur l'ordinateur le firmware complet de votre version, pour pouvoir revenir en arrière.",
        time: "30 minutes",
        verified: Verified::Partly,
        blocks: &[
            Text(
                "Sauvegardez d'abord vos données personnelles (photos, messages, comptes) : elles seront effacées. Puis téléchargez le firmware d'origine correspondant exactement à la version relevée à l'étape 02.",
            ),
            Text(
                "Nothing ne publie pas d'images d'usine. Le projet communautaire Nothing Archive conserve les mises à jour officielles du Phone (4b), déjà découpées en images : image-boot.7z, image-firmware.7z, image-logical.7z.001, .002… et un fichier d'empreintes -hash.sha256.",
            ),
            Code {
                title: "Décompresser et vérifier les images",
                code: r#"
sudo apt-get install p7zip-full
cd ~/supercontra/stock          # le dossier où vous avez mis les fichiers .7z
sha256sum -c *-hash.sha256      # si le fichier d'empreintes est fourni
7za -y x "*.7z*"
ls *.img
"#,
            },
            Code {
                title: "Si vous n'avez qu'une mise à jour complète (fichier OTA .zip)",
                code: r#"
cargo install payload_dumper              # un extracteur écrit en Rust
payload_dumper ota.zip --list             # les partitions contenues
payload_dumper ota.zip -o ~/supercontra/stock
"#,
            },
            Check(
                "Le dossier contient des fichiers .img, dont boot.img, init_boot.img, vendor_boot.img, vbmeta.img et les partitions logiques (system, vendor, product…).",
            ),
            Warning(
                "Ne téléchargez que la version exacte de votre téléphone, ou une plus récente : revenir à une version plus ancienne peut être refusé par le téléphone.",
            ),
            Source {
                title: "Nothing Archive : firmware du Phone (4b)",
                url: "https://nothingarchive.tech/docs/firmware#supercontra",
            },
            Source {
                title: "payload_dumper (Rust)",
                url: "https://docs.rs/crate/payload_dumper/latest",
            },
        ],
    },
    Step {
        id: "deverrouiller",
        title: "Déverrouiller le bootloader",
        goal: "Autoriser le téléphone à démarrer un système qui n'est pas signé par Nothing.",
        time: "15 minutes",
        verified: Verified::Partly,
        blocks: &[
            Text(
                "La procédure ci-dessous est celle de Nothing Archive pour les téléphones Nothing ; aucune source ne mentionne encore le Phone (4b) en particulier. Aucun code de déverrouillage n'est demandé par Nothing pour les modèles documentés.",
            ),
            Steps(&[
                "Dans Paramètres → Système → Options pour les développeurs, activez « Déverrouillage OEM » et « Débogage USB ».",
                "Si l'option « Déverrouillage OEM » est grisée, connectez le téléphone à Internet quelques minutes, puis réessayez.",
            ]),
            Code {
                title: "Déverrouiller",
                code: r#"
adb devices
adb reboot bootloader
fastboot devices          # le numéro de série doit s'afficher
fastboot flashing unlock
"#,
            },
            Steps(&[
                "Sur le téléphone, choisissez l'option de déverrouillage avec les touches de volume, puis confirmez avec le bouton marche.",
                "Le téléphone s'efface et redémarre. Refaites la configuration initiale, réactivez les options pour les développeurs et vérifiez que « Déverrouillage OEM » est toujours activé.",
            ]),
            Check(
                "Au démarrage, le téléphone affiche un avertissement « Orange State » : c'est normal, il signale un bootloader déverrouillé. cf props affiche alors « orange » sur la ligne « Démarrage vérifié ».",
            ),
            Warning(
                "Pour revenir en arrière, il faudra d'abord réinstaller le système d'origine complet (étape 12). Reverrouiller (fastboot flashing lock) avec une partition modifiée peut rendre le téléphone inutilisable.",
            ),
            Source {
                title: "Nothing Archive : déverrouiller le bootloader",
                url: "https://nothingarchive.tech/docs/guides#unlocking-bootloader",
            },
        ],
    },
    Step {
        id: "gsi",
        title: "Démarrer une image AOSP générique",
        goal: "Voir AOSP tourner sur le téléphone sans rien compiler, avec le noyau et les pilotes d'origine.",
        time: "30 minutes",
        verified: Verified::Partly,
        blocks: &[
            Text(
                "Depuis Android 9, le système (partition system) et les pilotes (partition vendor) sont séparés : c'est Treble. Google publie des images système génériques d'AOSP, les GSI, qui démarrent sur tout téléphone compatible. C'est le moyen le plus court de savoir ce qui marche sur le Phone (4b) : aucune compilation, aucun ordinateur puissant.",
            ),
            Code {
                title: "Vérifier que le téléphone est compatible",
                code: r#"
adb shell getprop ro.treble.enabled      # true
adb shell getprop ro.product.cpu.abi     # arm64-v8a : il faut une image ARM64
"#,
            },
            Text(
                "Téléchargez sur le site d'Android pour les développeurs l'image ARM64 d'AOSP (sans les services Google), pour Android 16 ou plus récent. L'archive contient system.img et vbmeta.img.",
            ),
            Code {
                title: "Installer l'image (procédure officielle, donnée pour les Pixel)",
                code: r#"
adb reboot bootloader
fastboot reboot fastboot                  # le mode fastboot « utilisateur »
fastboot erase system
fastboot flash system system.img
fastboot -w                               # efface les données
fastboot reboot-bootloader
fastboot --disable-verification flash vbmeta vbmeta.img
fastboot reboot
"#,
            },
            Check(
                "Le téléphone démarre sur AOSP. Notez ce qui fonctionne : écran, tactile, Wi-Fi, appels, caméra, son. Cette liste dit ce que l'étape 2 du projet devra réparer.",
            ),
            Tip(
                "Si fastboot répond « Not enough space to resize partition », la documentation propose de supprimer la partition product : fastboot delete-logical-partition product_a. Elle sera recréée en réinstallant le système d'origine (étape 12).",
            ),
            Warning(
                "La documentation officielle précise qu'il n'existe pas de procédure valable pour tous les téléphones. Si le téléphone ne démarre pas, réinstallez le système d'origine (étape 12).",
            ),
            Source {
                title: "Android : images système génériques (téléchargement et compatibilité)",
                url: "https://developer.android.com/topic/generic-system-image",
            },
            Source {
                title: "AOSP : installer une GSI",
                url: "https://source.android.com/docs/core/tests/vts/gsi",
            },
        ],
    },
    Step {
        id: "aosp",
        title: "Télécharger AOSP et le projet",
        goal: "Récupérer les sources d'Android, et y placer customFirmware.",
        time: "1 à 3 heures",
        verified: Verified::Yes,
        blocks: &[
            Code {
                title: "Initialiser AOSP",
                code: r#"
mkdir ~/aosp && cd ~/aosp
repo init --partial-clone --no-use-superproject \
  -b android-latest-release \
  -u https://android.googlesource.com/platform/manifest
"#,
            },
            Text(
                "Un manifeste local ajoute le dépôt du projet à l'arbre d'AOSP, dans vendor/customfirmware : c'est là qu'AOSP cherche les produits et les modules supplémentaires.",
            ),
            File {
                path: ".repo/local_manifests/customfirmware.xml",
                code: include_str!("../../manifests/customfirmware.xml"),
            },
            Code {
                title: "Ajouter le manifeste et tout télécharger",
                code: r#"
mkdir -p .repo/local_manifests
cp ~/customFirmware/manifests/customfirmware.xml .repo/local_manifests/
repo sync -c -j8
"#,
            },
            Text(
                "android-latest-release désigne toujours la dernière version publiée d'Android (Android 17 en novembre 2026).",
            ),
            Check(
                "repo sync se termine sans erreur, et ~/aosp/vendor/customfirmware contient products/ et services/.",
            ),
            Source {
                title: "AOSP : télécharger les sources",
                url: "https://source.android.com/docs/setup/download",
            },
            Source {
                title: "repo : manifestes locaux",
                url: "https://gerrit.googlesource.com/git-repo/+/HEAD/docs/manifest-format.md",
            },
        ],
    },
    Step {
        id: "rust",
        title: "Le premier service en Rust",
        goal: "Comprendre le service Binder du dépôt : la brique de base de tous les services du projet.",
        time: "30 minutes de lecture",
        verified: Verified::Untested,
        blocks: &[
            Text(
                "Sous Android, les programmes se parlent par Binder, et chaque service décrit ce qu'il offre dans une interface AIDL. AOSP sait générer le code Rust d'une interface AIDL : on écrit l'interface, puis le service en Rust. Les trois fichiers ci-dessous sont dans le dépôt, dans services/hello.",
            ),
            File {
                path: "services/hello/aidl/customfirmware/hello/IHello.aidl",
                code: include_str!("../../services/hello/aidl/customfirmware/hello/IHello.aidl"),
            },
            File {
                path: "services/hello/Android.bp",
                code: include_str!("../../services/hello/Android.bp"),
            },
            File {
                path: "services/hello/src/main.rs",
                code: include_str!("../../services/hello/src/main.rs"),
            },
            Text(
                "Ce service ne se compile pas avec Cargo : c'est AOSP qui le compile, avec sa bibliothèque Binder (libbinder_rs) et le code généré depuis l'interface AIDL.",
            ),
            Source {
                title: "AOSP : exemple de service AIDL en Rust",
                url: "https://source.android.com/docs/setup/build/rust/building-rust-modules/android-rust-patterns",
            },
            Source {
                title: "AOSP : les backends AIDL",
                url: "https://source.android.com/docs/core/architecture/aidl/aidl-backends",
            },
        ],
    },
    Step {
        id: "compiler",
        title: "Compiler et installer notre système",
        goal: "Construire customfirmware_arm64, une image AOSP générique avec le service Rust, et l'installer.",
        time: "1 à 6 heures de compilation",
        verified: Verified::Untested,
        blocks: &[
            Text(
                "Le produit customfirmware_arm64 reprend l'image générique d'AOSP et lui ajoute les services du projet. Il s'installe comme la GSI de l'étape 05, en gardant le noyau et les pilotes d'origine.",
            ),
            File {
                path: "products/customfirmware_arm64.mk",
                code: include_str!("../../products/customfirmware_arm64.mk"),
            },
            Code {
                title: "Compiler",
                code: r#"
cd ~/aosp
source build/envsetup.sh
lunch customfirmware_arm64-aosp_current-userdebug
m
"#,
            },
            Text(
                "La documentation d'AOSP compte environ 40 minutes sur une machine de 72 cœurs, et 6 heures sur 6 cœurs. Le résultat est dans out/target/product/generic_arm64.",
            ),
            Code {
                title: "Installer (la vérification est déjà désactivée depuis l'étape 05)",
                code: r#"
adb reboot bootloader
fastboot reboot fastboot
fastboot erase system
fastboot flash system out/target/product/generic_arm64/system.img
fastboot -w
fastboot reboot
"#,
            },
            Code {
                title: "Essayer le service Rust",
                code: r#"
adb root
adb shell setenforce 0                 # SELinux permissif, pour ce premier essai seulement
adb shell /system_ext/bin/customfirmware-hello &
adb shell service check customfirmware.hello
adb shell service call customfirmware.hello 1 s16 Max
"#,
            },
            Check(
                "service check répond « found », et service call renvoie un texte qui contient « Bonjour Max, depuis Rust ».",
            ),
            Warning(
                "Le service n'a pas encore de politique SELinux : init ne peut pas le lancer au démarrage, d'où le lancement à la main en mode permissif. Écrire cette politique est la suite du travail.",
            ),
            Source {
                title: "AOSP : compiler (lunch et cibles)",
                url: "https://source.android.com/docs/setup/build/building",
            },
            Source {
                title: "AOSP : installer une GSI",
                url: "https://source.android.com/docs/core/tests/vts/gsi",
            },
        ],
    },
    Step {
        id: "extraire",
        title: "Extraire les fichiers propriétaires",
        goal: "Récupérer, depuis le système d'origine, les binaires de Qualcomm et de Nothing qu'AOSP ne fournit pas.",
        time: "1 à 2 heures",
        verified: Verified::Partly,
        blocks: &[
            Text(
                "Ici commence l'étape 2 du projet : construire aussi la partition vendor, pour pouvoir y mettre nos propres couches matérielles en Rust. Le modem, le GPU, la caméra ou le Wi-Fi ont besoin de pilotes fermés ; on les reprend tels quels depuis les images de l'étape 03.",
            ),
            Code {
                title: "Ouvrir les partitions d'origine en lecture seule",
                code: r#"
cd ~/supercontra/stock
file vendor.img                         # ext4 ou EROFS
sudo mkdir -p /mnt/stock/vendor /mnt/stock/odm
sudo mount -o loop,ro vendor.img /mnt/stock/vendor
sudo mount -o loop,ro odm.img /mnt/stock/odm
ls /mnt/stock/vendor/lib64 | head
"#,
            },
            Text(
                "La liste des fichiers à reprendre s'écrit dans proprietary-files.txt, un chemin par ligne. cf blobs lit cette liste, copie les fichiers hors du dépôt et écrit le fragment Make qui les installe. Les fichiers introuvables sont signalés.",
            ),
            File {
                path: "device/SuperContra/proprietary-files.txt",
                code: include_str!("../../device/SuperContra/proprietary-files.txt"),
            },
            Code {
                title: "Copier les fichiers de la liste",
                code: r#"
cd ~/aosp/vendor/customfirmware
cf blobs --list device/SuperContra/proprietary-files.txt \
         --from /mnt/stock \
         --out ~/aosp/vendor/nothing/SuperContra
sudo umount /mnt/stock/vendor /mnt/stock/odm
"#,
            },
            Check(
                "cf blobs affiche « Introuvables : 0 », et ~/aosp/vendor/nothing/SuperContra contient proprietary/ et blobs.mk.",
            ),
            Warning(
                "Ces fichiers appartiennent à Qualcomm et à Nothing : ils restent sur votre ordinateur et ne doivent pas être publiés. Le dépôt ne les suit pas.",
            ),
            Source {
                title: "Un port voisin : LineageOS pour le Phone (4a), et ses fichiers propriétaires",
                url: "https://github.com/justjoshin78/frogger-lineage",
            },
        ],
    },
    Step {
        id: "noyau",
        title: "Le noyau",
        goal: "Garder le noyau d'origine pour commencer, puis le recompiler depuis les sources de Nothing.",
        time: "1 heure pour le noyau d'origine",
        verified: Verified::No,
        blocks: &[
            Text(
                "Nothing publie les sources du noyau du Phone (4b) (Linux 6.1), mais sans instructions de compilation. Le plus sûr est de réutiliser d'abord le noyau de Nothing, extrait de boot.img : on sait qu'il fonctionne avec les pilotes d'origine.",
            ),
            Code {
                title: "Extraire le noyau d'origine (outil d'AOSP)",
                code: r#"
cd ~/aosp
source build/envsetup.sh
m unpack_bootimg
unpack_bootimg --boot_img ~/supercontra/stock/boot.img --out ~/supercontra/boot
ls ~/supercontra/boot           # le fichier kernel est le noyau
"#,
            },
            Code {
                title: "Les sources, pour plus tard",
                code: r#"
git clone --depth 1 -b sm6650/b/SuperContra \
  https://github.com/NothingOSS/android_kernel_msm-6.1_nothing_sm6650.git \
  ~/supercontra/kernel
"#,
            },
            Text(
                "L'arbre contient les fichiers de Kleaf, le système de compilation par Bazel des noyaux Android, mais pas la commande exacte pour ce téléphone. Trouver cette commande est une tâche du projet.",
            ),
            Source {
                title: "NothingOSS : sources du noyau du Phone (4b)",
                url: "https://github.com/NothingOSS/android_kernel_msm-6.1_nothing_sm6650",
            },
            Source {
                title: "AOSP : compiler un noyau",
                url: "https://source.android.com/docs/setup/build/building-kernels",
            },
        ],
    },
    Step {
        id: "device-tree",
        title: "Le device tree",
        goal: "Décrire le téléphone à AOSP : son processeur, ses partitions, ses fichiers propriétaires.",
        time: "Plusieurs jours",
        verified: Verified::No,
        blocks: &[
            Text(
                "Le device tree est un dossier de fichiers Make qui dit à AOSP comment construire un système complet pour cet appareil. Le dépôt en contient le squelette, dans device/SuperContra ; il ne compile pas encore.",
            ),
            Code {
                title: "Y verser les faits du téléphone",
                code: r#"
cd ~/aosp/vendor/customfirmware
cf props ~/supercontra/getprop.txt --mk device/SuperContra/facts.mk
cat device/SuperContra/facts.mk
"#,
            },
            File {
                path: "device/SuperContra/BoardConfig.mk",
                code: include_str!("../../device/SuperContra/BoardConfig.mk"),
            },
            List(&[
                "La table des partitions : taille de la partition super et de ses groupes, relevées dans les images d'origine.",
                "Le noyau et l'image de démarrage (étape 10).",
                "La politique SELinux et le manifeste VINTF, qui déclare les couches matérielles.",
                "La liste des fichiers propriétaires (étape 09).",
            ]),
            Source {
                title: "AOSP : ajouter un nouvel appareil",
                url: "https://source.android.com/docs/setup/create/new-device",
            },
        ],
    },
    Step {
        id: "retour",
        title: "Revenir au système d'origine",
        goal: "Réinstaller Nothing OS à partir des images sauvegardées à l'étape 03.",
        time: "20 minutes",
        verified: Verified::Partly,
        blocks: &[
            Text(
                "Nothing Archive fournit un script qui installe toutes les images d'origine dans le bon ordre. Vérifiez sur sa page qu'il prend en charge le Phone (4b) avant de l'utiliser.",
            ),
            Code {
                title: "Avec le script de Nothing Archive",
                code: r#"
git clone https://github.com/spike0en/nothing_flasher.git
# copier le script du Phone (4b) dans ~/supercontra/stock, avec les .img
cd ~/supercontra/stock
chmod +x flash_all.sh
bash flash_all.sh
"#,
            },
            List(&[
                "Effacer les données : oui, pour une installation propre.",
                "Installer sur les deux emplacements (A et B) : oui, recommandé.",
                "Désactiver le démarrage vérifié (AVB) : non. Répondre oui empêcherait de reverrouiller le bootloader.",
            ]),
            Warning(
                "Si le script affiche une erreur, ne redémarrez pas sur le système : redémarrez sur le bootloader et relancez le script.",
            ),
            Code {
                title: "Reverrouiller, seulement sur un système d'origine propre",
                code: r#"
adb reboot bootloader
fastboot flashing lock
"#,
            },
            Source {
                title: "Nothing Archive : installer le firmware d'origine",
                url: "https://nothingarchive.tech/docs/guides",
            },
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_ids_are_unique_anchors() {
        let mut ids: Vec<_> = STEPS.iter().map(|s| s.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), STEPS.len());
        assert!(
            STEPS
                .iter()
                .all(|s| s.id.chars().all(|c| c.is_ascii_lowercase() || c == '-'))
        );
    }

    #[test]
    fn every_step_cites_a_source() {
        for step in STEPS {
            let cited = step
                .blocks
                .iter()
                .any(|b| matches!(b, Block::Source { .. }));
            assert!(cited, "{}", step.id);
        }
    }

    /// Les renvois « étape NN » du texte doivent suivre l'ordre des étapes.
    #[test]
    fn step_numbers_quoted_in_the_text_match() {
        let number = |id: &str| STEPS.iter().position(|s| s.id == id).unwrap();
        assert_eq!(number("informations"), 2);
        assert_eq!(number("sauvegarde"), 3);
        assert_eq!(number("gsi"), 5);
        assert_eq!(number("aosp"), 6);
        assert_eq!(number("compiler"), 8);
        assert_eq!(number("extraire"), 9);
        assert_eq!(number("noyau"), 10);
        assert_eq!(number("device-tree"), 11);
        assert_eq!(number("retour"), 12);
    }
}
