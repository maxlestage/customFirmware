# customFirmware

Un système Android (AOSP) pour le Nothing Phone (4b), dont tout le code ajouté par le projet
est écrit en Rust.

**État : en préparation.** Rien n'a encore été installé sur un téléphone ; la prochaine étape
est le déverrouillage du bootloader.

Site et guide : https://maxlestage.github.io/customFirmware/ · https://customfirmware-417dcda3fe66.herokuapp.com/

## Le dépôt

Il se place tel quel dans un arbre AOSP, dans `vendor/customfirmware`
(voir [`manifests/customfirmware.xml`](manifests/customfirmware.xml) et le guide, étape 06).

| Dossier | Contenu | État |
|---|---|---|
| [`tools/`](tools) | `cf`, l'outil du projet, en Rust : vérifie l'ordinateur, lit les faits du téléphone, extrait les fichiers propriétaires | écrit et testé |
| [`services/hello/`](services/hello) | le premier service Binder en Rust (interface AIDL + service) | écrit, pas encore compilé dans AOSP |
| [`products/`](products) | `customfirmware_arm64` : l'image système générique d'AOSP plus les services du projet | écrit, pas encore compilé |
| [`device/SuperContra/`](device/SuperContra) | la configuration propre au Phone (4b), pour l'étape 2 | squelette |
| [`manifests/`](manifests) | le manifeste local pour `repo` | écrit |
| [`site/`](site) | le site et le guide, une application [Paquets](https://github.com/maxlestage/Paquets) | en ligne |

Le projet avance en deux temps :

1. **Notre système sur les pilotes d'origine.** `customfirmware_arm64` est une image système
   générique : elle remplace la partition system et garde le noyau et la partition vendor de
   Nothing. Les APK s'y installent comme sur tout Android.
2. **Nos propres pilotes.** Une partition vendor construite par le projet, avec des couches
   matérielles en Rust (`device/SuperContra`).

## L'outil cf

```sh
cargo install --path tools
cf doctor ~                          # l'ordinateur peut-il compiler AOSP ?
cf props getprop.txt                 # les faits du téléphone, depuis `adb shell getprop`
cf blobs --list … --from … --out …   # copie les fichiers propriétaires de la liste
```

## Automatisation

| Workflow | Rôle |
|---|---|
| [`ci.yml`](.github/workflows/ci.yml) | formatage, clippy, tests et export statique à chaque push ; si tout est vert, la branche est **fusionnée dans `master`** (sauf `wip/...`), puis le site est publié |
| [`pages.yml`](.github/workflows/pages.yml) | exporte `master` en fichiers statiques et les publie sur GitHub Pages (source « GitHub Actions ») |
| [`heroku.yml`](.github/workflows/heroku.yml) | déploie `master` sur l'application Heroku `customfirmware` (secret `HEROKU_API_KEY` requis) |
