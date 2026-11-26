//! Le contenu du guide. Chaque étape est une suite de blocs : texte, listes,
//! commandes, fichiers, mises en garde. `guide_page.rs` les met en page.

/// Date de la dernière vérification des commandes.
pub const GUIDE_UPDATED: &str = "26 novembre 2026";

pub const INTRO: &str = "Ce guide décrit, dans l'ordre, tout le chemin entre un Nothing Phone (4b) d'origine et un système AOSP compilé par vous, dont les services propres à l'appareil sont écrits en Rust. Chaque étape dit d'où viennent ses commandes et si elles ont déjà été essayées sur ce téléphone.";

/// Ce que l'on sait d'une étape.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Verified {
    /// Les commandes viennent d'une documentation officielle ou reconnue.
    Yes,
    /// La méthode est connue, mais personne ne l'a encore publiée pour le 4b.
    Partly,
    /// Le projet doit encore écrire cette partie : le code est un point de départ.
    No,
}

impl Verified {
    pub fn label(self) -> &'static str {
        match self {
            Verified::Yes => "Sources officielles",
            Verified::Partly => "À confirmer sur le 4b",
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
            Verified::No => {
                "Le projet doit encore écrire cette partie ; le code montré est un point de départ."
            }
        }
    }

    pub fn class(self) -> &'static str {
        match self {
            Verified::Yes => "badge yes",
            Verified::Partly => "badge partly",
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
    /// Le contenu d'un fichier à créer.
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
                "Un ordinateur Linux 64 bits x86, avec au moins 64 Go de mémoire et 400 Go d'espace libre : 250 Go pour les sources d'AOSP, 150 Go pour la compilation.",
                "Une connexion rapide : les sources d'AOSP pèsent des dizaines de gigaoctets.",
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
        goal: "Installer les outils Android (adb, fastboot, repo), les paquets de compilation et Rust.",
        time: "20 minutes",
        verified: Verified::Yes,
        blocks: &[
            Text(
                "Les commandes ci-dessous sont pour Ubuntu (18.04 ou plus récent), la distribution que la documentation d'AOSP prend pour exemple.",
            ),
            Code {
                title: "Paquets de compilation d'AOSP et repo",
                code: r#"
sudo apt-get update
sudo apt-get install git-core gnupg flex bison build-essential zip curl \
  zlib1g-dev libc6-dev-i386 x11proto-core-dev libx11-dev lib32z1-dev \
  libgl1-mesa-dev libxml2-utils xsltproc unzip fontconfig
sudo apt-get install repo
repo version        # 2.4 ou plus récent
"#,
            },
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
                title: "Rust, pour les outils du projet",
                code: r#"
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustc --version
"#,
            },
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
                title: "Ce que le device tree devra reproduire",
                code: r#"
adb devices                                   # le téléphone doit être listé « device »
mkdir -p ~/supercontra && cd ~/supercontra
adb shell getprop > getprop.txt               # toutes les propriétés, pour plus tard
adb shell getprop ro.build.display.id         # la version de Nothing OS
adb shell getprop ro.build.fingerprint
adb shell getprop ro.board.platform           # le nom de plateforme Qualcomm
adb shell getprop ro.product.first_api_level
adb shell getprop ro.boot.dynamic_partitions  # partitions dynamiques (super)
adb shell getprop ro.virtual_ab.enabled       # mises à jour A/B virtuelles
adb shell ls /vendor/etc | grep -i fstab      # la table des partitions
"#,
            },
            Check(
                "getprop.txt contient plusieurs centaines de lignes, et ro.build.display.id donne une version de la forme SuperContra-B4.1-….",
            ),
            Tip(
                "Copiez le fichier fstab trouvé avec adb pull /vendor/etc/<nom du fichier> . : il décrit les partitions que le système monte au démarrage.",
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
                "Au démarrage, le téléphone affiche un avertissement « Orange State » : c'est normal, il signale un bootloader déverrouillé.",
            ),
            Warning(
                "Pour revenir en arrière, il faudra d'abord réinstaller le système d'origine complet (étape 11). Reverrouiller (fastboot flashing lock) avec une partition modifiée peut rendre le téléphone inutilisable.",
            ),
            Source {
                title: "Nothing Archive : déverrouiller le bootloader",
                url: "https://nothingarchive.tech/docs/guides#unlocking-bootloader",
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
                "Le modem, le GPU, la caméra ou le Wi-Fi ont besoin de pilotes fermés, rangés dans les partitions vendor, odm et vendor_dlkm. AOSP ne les a pas : on les reprend tels quels depuis les images de l'étape 03.",
            ),
            Code {
                title: "Ouvrir la partition vendor en lecture seule",
                code: r#"
cd ~/supercontra/stock
file vendor.img                         # ext4 ou EROFS
sudo mkdir -p /mnt/vendor
sudo mount -o loop,ro vendor.img /mnt/vendor
ls /mnt/vendor/lib64 /mnt/vendor/firmware | head
"#,
            },
            Text(
                "La liste des fichiers à reprendre s'écrit dans proprietary-files.txt, à la racine du device tree, un chemin par ligne. Un outil du projet, écrit en Rust, lira cette liste et copiera les fichiers dans vendor/nothing/SuperContra.",
            ),
            File {
                path: "device/nothing/SuperContra/proprietary-files.txt (extrait)",
                code: r#"
# Chaque ligne est un chemin dans les partitions d'origine.
vendor/lib64/<bibliothèque>.so
vendor/firmware/<micrologiciel>
vendor/etc/init/<service>.rc
"#,
            },
            Tip(
                "Les vrais noms de fichiers se lisent dans /mnt/vendor. Les ports LineageOS d'appareils Qualcomm proches, comme celui du Phone (4a), montrent à quoi ressemble une liste complète.",
            ),
            Source {
                title: "Un port voisin : LineageOS pour le Phone (4a), et ses fichiers propriétaires",
                url: "https://github.com/justjoshin78/frogger-lineage",
            },
            Code {
                title: "Démonter",
                code: "sudo umount /mnt/vendor",
            },
        ],
    },
    Step {
        id: "aosp",
        title: "Télécharger AOSP",
        goal: "Récupérer les sources d'Android sur l'ordinateur.",
        time: "1 à 3 heures",
        verified: Verified::Yes,
        blocks: &[
            Code {
                title: "Initialiser et télécharger",
                code: r#"
mkdir ~/aosp && cd ~/aosp
repo init --partial-clone --no-use-superproject \
  -b android-latest-release \
  -u https://android.googlesource.com/platform/manifest
repo sync -c -j8
"#,
            },
            Text(
                "android-latest-release désigne toujours la dernière version publiée d'Android (Android 17 en novembre 2026). Le système d'origine du 4b est sous Android 16 : la séparation entre système et vendor d'Android (Treble) permet en principe de faire tourner un système plus récent sur les pilotes d'origine.",
            ),
            Check(
                "La commande repo sync se termine sans erreur et ~/aosp contient build/, frameworks/, system/…",
            ),
            Source {
                title: "AOSP : télécharger les sources",
                url: "https://source.android.com/docs/setup/download",
            },
        ],
    },
    Step {
        id: "noyau",
        title: "Le noyau",
        goal: "Démarrer d'abord avec le noyau d'origine, puis le recompiler depuis les sources de Nothing.",
        time: "1 heure pour le noyau d'origine",
        verified: Verified::No,
        blocks: &[
            Text(
                "Nothing publie les sources du noyau du Phone (4b) (Linux 6.1), mais sans instructions de compilation. Pour le premier démarrage, le plus sûr est de réutiliser le noyau de Nothing, extrait de boot.img : on sait qu'il fonctionne avec les pilotes d'origine.",
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
        title: "Écrire le device tree",
        goal: "Décrire le téléphone à AOSP : son processeur, ses partitions, ses fichiers propriétaires.",
        time: "Plusieurs jours",
        verified: Verified::No,
        blocks: &[
            Text(
                "Le device tree est un dossier de fichiers Make qui dit à AOSP comment construire un système pour cet appareil. Voici le squelette ; chaque valeur marquée « à relever » vient de l'étape 02.",
            ),
            File {
                path: "device/nothing/SuperContra/AndroidProducts.mk",
                code: r#"
PRODUCT_MAKEFILES := $(LOCAL_DIR)/aosp_SuperContra.mk
COMMON_LUNCH_CHOICES := aosp_SuperContra-aosp_current-userdebug
"#,
            },
            File {
                path: "device/nothing/SuperContra/aosp_SuperContra.mk",
                code: r#"
$(call inherit-product, $(SRC_TARGET_DIR)/product/core_64_bit_only.mk)
$(call inherit-product, $(SRC_TARGET_DIR)/product/aosp_base_telephony.mk)
$(call inherit-product, device/nothing/SuperContra/device.mk)

PRODUCT_NAME := aosp_SuperContra
PRODUCT_DEVICE := SuperContra
PRODUCT_BRAND := Nothing
PRODUCT_MANUFACTURER := Nothing
PRODUCT_MODEL := A009P
"#,
            },
            File {
                path: "device/nothing/SuperContra/BoardConfig.mk",
                code: r#"
TARGET_ARCH := arm64
TARGET_ARCH_VARIANT := armv8-2a
TARGET_CPU_ABI := arm64-v8a
TARGET_CPU_VARIANT := generic

TARGET_BOOTLOADER_BOARD_NAME := SuperContra
TARGET_BOARD_PLATFORM :=            # à relever : ro.board.platform

# Partitions dynamiques : tailles à relever dans les images d'origine.
BOARD_SUPER_PARTITION_SIZE :=
BOARD_SUPER_PARTITION_GROUPS := nothing_dynamic_partitions
BOARD_NOTHING_DYNAMIC_PARTITIONS_PARTITION_LIST := system system_ext product vendor odm
"#,
            },
            File {
                path: "device/nothing/SuperContra/device.mk",
                code: r#"
# Le noyau d'origine, extrait à l'étape 07.
LOCAL_KERNEL := device/nothing/SuperContra-kernel/kernel
PRODUCT_COPY_FILES += $(LOCAL_KERNEL):kernel

# Le premier service Rust du projet (étape 09).
PRODUCT_PACKAGES += customfirmware-hello
"#,
            },
            Check(
                "Après source build/envsetup.sh, la commande lunch sans argument liste aosp_SuperContra-aosp_current-userdebug.",
            ),
            Source {
                title: "AOSP : compiler (lunch et cibles)",
                url: "https://source.android.com/docs/setup/build/building",
            },
        ],
    },
    Step {
        id: "rust",
        title: "Écrire le premier service en Rust",
        goal: "Ajouter au système un service Binder écrit en Rust, la brique de base de tous les HAL du projet.",
        time: "2 heures",
        verified: Verified::Partly,
        blocks: &[
            Text(
                "Un HAL Android est un service qui parle le langage Binder, décrit par une interface AIDL. AOSP sait générer le code Rust d'une interface AIDL : on écrit l'interface, puis le service en Rust. Le code suit l'exemple officiel d'AOSP.",
            ),
            File {
                path: "vendor/customfirmware/hello/aidl/vendor/customfirmware/hello/IHello.aidl",
                code: r#"
package vendor.customfirmware.hello;

interface IHello {
    String greet(String name);
}
"#,
            },
            File {
                path: "vendor/customfirmware/hello/Android.bp",
                code: r#"
aidl_interface {
    name: "vendor.customfirmware.hello",
    srcs: ["aidl/vendor/customfirmware/hello/*.aidl"],
    local_include_dir: "aidl",
    unstable: true, // tant que l'interface change
    backend: {
        rust: {
            enabled: true, // le Rust n'est pas activé par défaut
        },
    },
}

rust_binary {
    name: "customfirmware-hello",
    srcs: ["src/main.rs"],
    rustlibs: [
        "vendor.customfirmware.hello-rust",
        "libbinder_rs",
    ],
    init_rc: ["customfirmware-hello.rc"],
}
"#,
            },
            File {
                path: "vendor/customfirmware/hello/src/main.rs",
                code: r#"
use binder::{BinderFeatures, Interface, Result as BinderResult};
use vendor_customfirmware_hello::aidl::vendor::customfirmware::hello::IHello::{
    BnHello, IHello,
};

struct Hello;

impl Interface for Hello {}

impl IHello for Hello {
    fn greet(&self, name: &str) -> BinderResult<String> {
        Ok(format!("Bonjour {name}, depuis Rust"))
    }
}

fn main() {
    let service = BnHello::new_binder(Hello, BinderFeatures::default());
    binder::add_service("customfirmware.hello", service.as_binder())
        .expect("impossible d'enregistrer le service");
    binder::ProcessState::join_thread_pool();
}
"#,
            },
            File {
                path: "vendor/customfirmware/hello/customfirmware-hello.rc",
                code: r#"
service customfirmware-hello /system/bin/customfirmware-hello
    class main
    user system
    group system
"#,
            },
            Code {
                title: "Compiler le service seul, puis l'essayer",
                code: r#"
cd ~/aosp && source build/envsetup.sh
lunch aosp_SuperContra-aosp_current-userdebug
m customfirmware-hello
# une fois le système installé (étape 10) :
adb root
adb shell setenforce 0                 # SELinux permissif, pour ce premier essai seulement
adb shell service list | grep customfirmware
"#,
            },
            Warning(
                "SELinux bloque par défaut un service inconnu. Le mode permissif ne sert qu'au premier essai : le vrai HAL aura sa politique SELinux, une interface stable et sa déclaration dans le manifeste VINTF.",
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
        id: "installer",
        title: "Compiler et installer",
        goal: "Construire le système complet et l'installer sur le téléphone.",
        time: "1 à 6 heures de compilation",
        verified: Verified::Partly,
        blocks: &[
            Code {
                title: "Compiler",
                code: r#"
cd ~/aosp
source build/envsetup.sh
lunch aosp_SuperContra-aosp_current-userdebug
m
"#,
            },
            Text(
                "La documentation d'AOSP compte environ 40 minutes sur une machine de 72 cœurs, et 6 heures sur 6 cœurs. Le résultat est dans out/target/product/SuperContra.",
            ),
            Code {
                title: "Installer",
                code: r#"
adb reboot bootloader
fastboot devices
fastboot flashall -w     # -w efface les données : nécessaire au premier essai
"#,
            },
            Check(
                "Le téléphone redémarre sur l'animation d'AOSP. Le premier démarrage peut prendre plusieurs minutes.",
            ),
            Tip(
                "Si le téléphone revient au bootloader ou boucle au démarrage, ne paniquez pas : passez à l'étape 11 pour réinstaller le système d'origine, puis cherchez la cause dans le journal (adb logcat, s'il s'est lancé assez loin).",
            ),
            Source {
                title: "AOSP : installer une version compilée",
                url: "https://source.android.com/docs/setup/test/running",
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
    fn every_step_cites_a_source_or_is_project_work() {
        for step in STEPS {
            let cited = step
                .blocks
                .iter()
                .any(|b| matches!(b, Block::Source { .. }));
            assert!(cited || step.verified == Verified::No, "{}", step.id);
        }
    }
}
