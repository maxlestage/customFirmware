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

## Déploiement

`heroku.yml` et `app.json`, à la racine du dépôt, font construire `site/Dockerfile` par Heroku.
Cette configuration n'a pas encore été essayée : le site n'est déployé nulle part.
