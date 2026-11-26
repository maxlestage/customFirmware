# energie : les réglages d'énergie du noyau

**État : écrit et testé sur un faux arbre `/sys` ; démarrage par init et SELinux écrits ; pas encore compilé dans AOSP ni essayé sur le téléphone.**

Le noyau du Phone (4b) (`android_kernel_msm-6.1_nothing_sm6650`) reste celui de Nothing :
environ 40 millions de lignes de C, dont les pilotes de Qualcomm. energie, écrit en Rust,
change ses **réglages** (`/sys`, `/proc/sys`) pour que la batterie dure plus longtemps :

- des **profils** : des fichiers texte, un réglage par ligne, à écrire soi-même ;
- des **règles** : écran éteint, branché, batterie faible → tel profil, appliqué tout seul ;
- un **journal** : la consommation mesurée chaque minute, et un **rapport** qui dit ce que
  chaque profil coûte vraiment, pour l'améliorer d'une semaine sur l'autre.

Avant de changer un réglage, energie note sa valeur d'origine (`origine.etat`) :
`energie restaurer` remet toujours le noyau comme il était.

## Dans le système

Avec le produit `customfirmware_arm64`, energie fait partie du système :

| Fichier | Rôle |
|---|---|
| `/system_ext/bin/customfirmware-energie` | le programme |
| `/system_ext/etc/init/energie.rc` | [energie.rc](energie.rc) : init le lance au démarrage, en root sans aucune capacité |
| `/system_ext/etc/energie/` | les réglages livrés ([reglages/](reglages)), en lecture seule |
| `/data/misc/energie/` | les réglages **à modifier**, le journal, les valeurs d'origine |
| [sepolicy/](sepolicy) | son domaine SELinux, `energie` |

Au premier lancement, les réglages livrés sont copiés dans `/data/misc/energie`. Ensuite, ce
sont ceux-là qui comptent : un fichier modifié n'est jamais écrasé, et le démon relit tout
dans les 2 s qui suivent une modification. Un fichier faux est refusé (le message est dans
`dmesg`) et les réglages précédents restent.

```sh
adb root
adb shell customfirmware-energie etat
adb pull /data/misc/energie/profils/economie.conf      # modifier, puis :
adb push economie.conf /data/misc/energie/profils/      # pris en compte tout de suite
adb shell customfirmware-energie forcer performance     # pour un jeu
adb shell customfirmware-energie forcer auto            # retour aux règles
adb shell customfirmware-energie rapport
adb shell dmesg | grep energie                          # ce que fait le démon
```

## L'essayer avant d'avoir le système

Sur n'importe quel Android rooté, avec un binaire statique compilé par Cargo :

```sh
rustup target add aarch64-unknown-linux-musl
cargo build --release -p energie --target aarch64-unknown-linux-musl
adb push target/aarch64-unknown-linux-musl/release/energie /data/local/tmp/
adb push services/energie/reglages /data/local/tmp/energie-modeles
adb shell su -c '/data/local/tmp/energie --modeles /data/local/tmp/energie-modeles verifier economie'
```

Sans `--dossier`, il travaille dans `/data/misc/energie`, comme dans le système.

## Les commandes

| Commande | Rôle |
|---|---|
| `etat` | batterie, écran, profil actif, fréquences de chaque groupe de cœurs, GPU |
| `lire <motif>…` | explorer les réglages : `energie lire '/sys/devices/system/cpu/cpufreq/policy*/walt/*'` |
| `liste` | les profils, avec leur description |
| `verifier <profil>` | ce que le profil changerait, sans rien écrire |
| `appliquer <profil>` | applique le profil |
| `restaurer` | remet tous les réglages d'origine |
| `forcer <profil>` / `forcer auto` | impose un profil au démon / lui rend la main |
| `demon` | applique les règles de `energie.conf` et tient `journal.csv` |
| `rapport` | par profil et par état de l'écran : puissance moyenne, % perdus par heure, autonomie |

## Les profils livrés

| Profil | Quand | Ce qu'il règle |
|---|---|---|
| `economie` | écran allumé (défaut) | processeurs plafonnés à 85 % de leur fréquence maximale |
| `ecran_eteint` | écran éteint | 50 % ; plus de boost au toucher ; disque vidé toutes les 30 s |
| `economie_forte` | batterie ≤ 20 % | 65 % ; WALT monte moins vite ; GPU plafonnable (commenté) |
| `performance` | à la main, pour les jeux | boost du planificateur WALT |
| `origine` | branché | les réglages de Nothing |

Les chemins viennent du noyau de ce téléphone (gouverneur `walt`,
`kernel/sched/walt/` ; sysctl `/proc/sys/walt/`). Un réglage absent est signalé et sauté.

## Écrire son profil

Un fichier `profils/<nom>.conf` ; la première ligne de commentaire est sa description.

```ini
# Lecture : écran allumé, rien de lourd.
/sys/devices/system/cpu/cpufreq/policy*/scaling_max_freq = 70% cpuinfo_max_freq
/proc/sys/walt/sched_boost = 0
```

- `*` remplace n'importe quel morceau d'un nom ;
- `70% cpuinfo_max_freq` : 70 % du fichier voisin, arrondi à une fréquence disponible ;
- seuls `/sys/…` et `/proc/sys/…` sont acceptés.

Le déposer dans `/data/misc/energie/profils/`, puis `energie verifier lecture`,
`energie forcer lecture`, ou une règle dans `energie.conf`.

## Optimiser régulièrement

1. Laisser tourner le démon quelques jours avec un profil.
2. `energie rapport` : noter les %/heure écran allumé et écran éteint.
3. Changer **un** réglage dans le profil, recommencer, comparer.

Le rapport ne conclut qu'après 1 h et 2 % perdus dans le même état : en dessous, la jauge
de la batterie est trop grossière.

## SELinux

Le domaine `energie` a le droit d'écrire les réglages des processeurs
(`sysfs_devices_system_cpu`), `/proc/sys/walt` et `/proc/sys/vm` (`proc`, `proc_dirty`), et
de lire la batterie et l'écran. La partition vendor de Nothing peut donner à certains
fichiers ses propres étiquettes, qu'une politique système ne peut pas nommer : sur un
système `userdebug`, le domaine est donc permissif, et ce qui lui manquerait est noté :

```sh
adb shell dmesg | grep avc | grep energie
```

Sur un système `user`, ces lignes disent quelles règles ajouter à
[energie.te](sepolicy/energie.te).

## Ce qui reste à faire

- compiler le système dans AOSP (`m`) : `energie.rc`, la politique SELinux et
  `Android.bp` sont écrits, mais pas encore passés par la compilation d'AOSP ;
- l'essayer sur le vrai téléphone, puis régler les profils avec le rapport ;
- choisir le profil depuis le téléphone, sans ordinateur (une tuile des réglages rapides).
