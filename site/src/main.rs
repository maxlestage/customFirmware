//! Le site de présentation de customFirmware, une application Paquets.
//!
//!     paquets dev                        # le serveur, http://localhost:3000
//!     cargo run -- export dist           # le site en fichiers statiques (GitHub Pages)

mod content;
mod pages;

use std::io;
use std::path::Path;

use paquets::prelude::*;

fn app() -> App {
    let mut app = App::new();
    app.get("/", pages::home);
    app.health("/health"); // pour les sondes de disponibilité
    app.get("/*", pages::not_found);
    app
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [] => app().run(), // $PORT sur Heroku, 3000 sinon
        [cmd, out] if cmd == "export" => {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            let written = runtime.block_on(export(Path::new(out)))?;
            println!("{written} fichiers écrits dans {out}");
            Ok(())
        }
        _ => Err(io::Error::other(
            "usage : custom-firmware-site [export <dossier>]",
        )),
    }
}

/// Écrit le site dans `out` : `index.html`, `404.html` et les fichiers de `public/`.
/// Les pages viennent du serveur lui-même, par le client de test de Paquets : le site
/// statique et le site sur Heroku sont identiques. `SITE_BASE` préfixe les liens.
async fn export(out: &Path) -> io::Result<usize> {
    let mut app = app();
    // Des fichiers statiques n'ont pas de serveur à protéger.
    app.security(None);
    let client = app.test().await;
    std::fs::create_dir_all(out)?;

    let mut written = 0;
    for (path, file, status) in [("/", "index.html", 200), ("/404", "404.html", 404)] {
        let res = client.get(path).await;
        if res.status().as_u16() != status {
            return Err(io::Error::other(format!(
                "GET {path} : {} au lieu de {status}",
                res.status()
            )));
        }
        std::fs::write(out.join(file), res.text())?;
        written += 1;
    }
    for entry in std::fs::read_dir("public")? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            std::fs::copy(entry.path(), out.join(entry.file_name()))?;
            written += 1;
        }
    }
    // GitHub Pages sert les fichiers tels quels, sans passer par Jekyll.
    std::fs::write(out.join(".nojekyll"), "")?;
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn home_page_presents_the_project() {
        let client = app().test().await;
        let res = client.get("/").await;
        assert_eq!(res.status(), 200);
        let html = res.text();
        assert!(html.contains(r#"<html lang="fr""#));
        assert!(html.contains("Nothing Phone (4b)"));
        assert!(html.contains("SuperContra"));
        assert!(html.contains("Feuille de route"));
    }

    #[tokio::test]
    async fn every_roadmap_step_is_on_the_page() {
        let html = app().test().await.get("/").await.text();
        for step in content::ROADMAP {
            // Le texte est échappé par Active : l'apostrophe devient une entité.
            let title = step.title.replace('\'', "&#x27;");
            assert!(
                html.contains(&title) || html.contains(step.title),
                "{}",
                step.title
            );
        }
    }

    #[tokio::test]
    async fn exactly_one_step_is_next() {
        let next = content::ROADMAP
            .iter()
            .filter(|s| s.status == content::Status::Next)
            .count();
        assert_eq!(next, 1);
    }

    #[tokio::test]
    async fn export_writes_the_static_site() {
        let out = std::env::temp_dir().join(format!("cf-export-{}", std::process::id()));
        let written = export(&out).await.unwrap();
        assert!(written >= 4);
        let index = std::fs::read_to_string(out.join("index.html")).unwrap();
        assert!(index.contains("Feuille de route"));
        assert!(
            std::fs::read_to_string(out.join("404.html"))
                .unwrap()
                .contains("404")
        );
        assert!(out.join("site.css").is_file());
        std::fs::remove_dir_all(out).unwrap();
    }

    #[tokio::test]
    async fn health_and_unknown_pages() {
        let client = app().test().await;
        assert_eq!(client.get("/health").await.status(), 200);
        assert_eq!(client.get("/nope").await.status(), 404);
    }
}
