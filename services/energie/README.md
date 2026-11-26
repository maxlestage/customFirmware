# energie : les réglages d'énergie du noyau

**État : écrit et testé sur un faux arbre `/sys` ; pas encore essayé sur le téléphone.**

Le noyau du Phone (4b) (`android_kernel_msm-6.1_nothing_sm6650`) reste celui de Nothing :
environ 40 millions de lignes de C, dont les pilotes de Qualcomm. energie, écrit en Rust,
change ses **réglages** (`/sys`, `/proc/sys`) pour que la batterie dure plus longtemps :

- des **profils** : des fichiers texte, un réglage par ligne, à écrire soi-même ;
- des **règles** : écran éteint, branché, batterie faible → tel profil, appliqué tout seul ;
- un **journal** : la consommation mesurée chaque minute, et un **rapport** qui dit ce que
  chaque profil coûte vraiment, pour l'améliorer d'une semaine sur l'autre.

Avant de changer un réglage, energie note sa valeur d'origine (`origine.etat`) :
`energie restaurer` remet toujours le noyau comme il était.

## Compiler

```sh
rustup target add aarch64-unknown-linux-musl
cargo build --release -p energie --target aarch64-unknown-linux-musl
# → target/aarch64-unknown-linux-musl/release/energie (statique, ~500 Ko)
```

Dans AOSP, `m customfirmware-energie` (voir [Android.bp](Android.bp)).

## L'essayer sur le téléphone

Il faut être root (système `userdebug` et `adb root`, ou téléphone rooté) : le noyau
n'accepte pas les réglages d'un utilisateur ordinaire.

```sh
adb push target/aarch64-unknown-linux-musl/release/energie /data/local/tmp/
adb push services/energie/reglages /data/local/tmp/energie
adb root
adb shell /data/local/tmp/energie etat
adb shell /data/local/tmp/energie verifier economie   # ce qui changerait, sans rien écrire
adb shell /data/local/tmp/energie appliquer economie
adb shell /data/local/tmp/energie restaurer
```

Pour qu'il choisisse les profils tout seul :

```sh
adb shell 'nohup /data/local/tmp/energie demon > /data/local/tmp/energie/demon.log 2>&1 &'
# quelques jours plus tard :
adb shell /data/local/tmp/energie rapport
```

Le démon regarde l'écran et la batterie toutes les 2 s, mais seulement quand le téléphone
est éveillé : il ne l'empêche jamais de dormir. Il ne survit pas encore à un redémarrage
(voir « Ce qui manque »).

## Les commandes

| Commande | Rôle |
|---|---|
| `etat` | batterie, écran, profil actif, fréquences de chaque groupe de cœurs, GPU |
| `lire <motif>…` | explorer les réglages : `energie lire '/sys/devices/system/cpu/cpufreq/policy*/walt/*'` |
| `liste` | les profils, avec leur description |
| `verifier <profil>` | ce que le profil changerait, sans rien écrire |
| `appliquer <profil>` | applique le profil |
| `restaurer` | remet tous les réglages d'origine |
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

Puis `energie verifier lecture`, `energie appliquer lecture`, ou une règle dans
`energie.conf`.

## Optimiser régulièrement

1. Laisser tourner le démon quelques jours avec un profil.
2. `energie rapport` : noter les %/heure écran allumé et écran éteint.
3. Changer **un** réglage dans le profil, recommencer, comparer.

Le rapport ne conclut qu'après 1 h et 2 % perdus dans le même état : en dessous, la jauge
de la batterie est trop grossière.

## Ce qui manque

- une politique SELinux et un `.rc` pour qu'`init` le lance au démarrage, comme pour
  [`hello`](../hello) ;
- un essai sur le vrai téléphone : les valeurs des profils sont des points de départ
  prudents, à ajuster avec le rapport.
