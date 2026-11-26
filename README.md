# customFirmware

Un portage d'Android (AOSP) pour le Nothing Phone (4b), dont tout le code propre à l'appareil
est écrit en Rust.

**État : en préparation.** Rien n'est encore installable ; la prochaine étape est le
déverrouillage du bootloader.

Site : https://customfirmware-417dcda3fe66.herokuapp.com/ (Heroku) · https://maxlestage.github.io/customFirmware/ (GitHub Pages)

| Dossier | Contenu |
|---|---|
| [`site/`](site) | le site de présentation, une application [Paquets](https://github.com/maxlestage/Paquets) |

Les faits sur l'appareil, le périmètre du projet et la feuille de route sont dans
[`site/src/content.rs`](site/src/content.rs), et affichés par le site.

## Automatisation

| Workflow | Rôle |
|---|---|
| [`ci.yml`](.github/workflows/ci.yml) | formatage, clippy, tests et export statique à chaque push ; si tout est vert, la branche est **fusionnée dans `master`** (sauf `wip/...`), puis le site est publié |
| [`pages.yml`](.github/workflows/pages.yml) | exporte `master` en fichiers statiques et les publie sur GitHub Pages (source « GitHub Actions ») |
| [`heroku.yml`](.github/workflows/heroku.yml) | déploie `master` sur l'application Heroku `customfirmware` (secret `HEROKU_API_KEY` requis) |
