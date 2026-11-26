//! Le site de présentation de customFirmware, une application Paquets.
//!
//!     paquets dev      # http://localhost:3000

mod content;
mod pages;

use paquets::prelude::*;

fn app() -> App {
    let mut app = App::new();
    app.get("/", pages::home);
    app.health("/health"); // pour les sondes de disponibilité
    app.get("/*", pages::not_found);
    app
}

fn main() -> std::io::Result<()> {
    app().run() // $PORT sur Heroku, 3000 sinon
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
    async fn health_and_unknown_pages() {
        let client = app().test().await;
        assert_eq!(client.get("/health").await.status(), 200);
        assert_eq!(client.get("/nope").await.status(), 404);
    }
}
