# Le site de présentation

Une application [Paquets](https://github.com/maxlestage/Paquets), écrite en Rust : une page
rendue par le serveur, sans base de données ni JavaScript.

```sh
cd site
paquets dev          # http://localhost:3000 (ou `cargo run`)
cargo test
```

| Fichier | Contenu |
|---|---|
| `src/content.rs` | les textes et les données : l'appareil, le périmètre Rust, la feuille de route, les sources |
| `src/pages.rs` | les vues : la page d'accueil et la page 404 |
| `src/main.rs` | les routes et les tests |
| `public/site.css` | les styles, par-dessus le thème de Paquets |

Pour mettre le site à jour (une étape terminée, un fait vérifié), modifiez `src/content.rs` :
le statut d'une étape est `Status::Done`, `Status::Next` ou `Status::Todo`.

Pour GitHub Pages, le site est exporté en fichiers statiques, avec les liens préfixés par
`SITE_BASE` :

```sh
cd site
SITE_BASE=/customFirmware cargo run -- export ../dist
```

## Déploiement

- **Heroku** : buildpack emk/rust sur la pile heroku-26 (`RustConfig`, `Procfile` et `app.json`
  à la racine du dépôt). `cargo build --release` à la racine compile le site.
- **GitHub Pages** : `.github/workflows/pages.yml` pousse l'export sur la branche `gh-pages`.

Les deux sont lancés après chaque fusion dans `master` (voir le README du dépôt).
