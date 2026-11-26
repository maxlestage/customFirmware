//! Les pages du site. Chaque composant est une fonction qui renvoie une vue.

use paquets::prelude::*;

use crate::content::{
    FACTS, Fact, IN_RUST, NOT_IN_RUST, Part, REFERENCES, RISKS, ROADMAP, Status, Step, VERIFIED_ON,
};

const REPOSITORY: &str = "https://github.com/maxlestage/customFirmware";
const PAQUETS: &str = "https://github.com/maxlestage/Paquets";

/// Le préfixe des liens internes : vide sur Heroku, `/customFirmware` sur GitHub Pages,
/// où le site est servi dans un sous-dossier. Lu une fois dans `SITE_BASE`.
fn base() -> &'static str {
    static BASE: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    BASE.get_or_init(|| {
        let base = std::env::var("SITE_BASE").unwrap_or_default();
        base.trim_end_matches('/').to_owned()
    })
}

/// Un lien interne : `link("/site.css")` donne `/customFirmware/site.css` sur GitHub Pages.
fn link(path: &str) -> String {
    format!("{}{path}", base())
}

/// L'icône et la feuille de style, communes aux deux pages.
fn assets(page: Page) -> Page {
    page.head(format!(
        r#"<link rel="icon" href="{}"/>"#,
        link("/favicon.svg")
    ))
    .stylesheet(link("/site.css"))
}

/// La page d'accueil : tout le site tient sur une page.
pub async fn home(_req: Request) -> Page {
    assets(Page::new("customFirmware : AOSP et Rust sur le Nothing Phone (4b)")
        .lang("fr")
        .description(
            "Un portage d'Android (AOSP) pour le Nothing Phone (4b), dont tout le code propre à l'appareil est écrit en Rust.",
        ))
        .body(site_header())
        .body(
            main()
                .child(intro())
                .child(device())
                .child(scope())
                .child(roadmap())
                .child(risks())
                .child(references()),
        )
        .body(site_footer())
}

/// La page des adresses inconnues.
pub async fn not_found(_req: Request) -> Page {
    assets(Page::new("Page introuvable"))
        .lang("fr")
        .status(404)
        .body(site_header())
        .body(
            main().child(
                section()
                    .class("intro")
                    .child(p().class("eyebrow").text("Erreur 404"))
                    .child(h1().text("Cette page n'existe pas."))
                    .child(
                        a().class("button")
                            .href(link("/"))
                            .text("Retour à l'accueil"),
                    ),
            ),
        )
        .body(site_footer())
}

fn site_header() -> Element {
    header()
        .child(a().class("brand").href(link("/")).text("customFirmware"))
        .child(
            nav()
                .attr("aria-label", "Sections")
                .child(a().href(link("/#appareil")).text("Appareil"))
                .child(a().href(link("/#rust")).text("Rust"))
                .child(
                    a().href(link("/#feuille-de-route"))
                        .text("Feuille de route"),
                )
                .child(a().href(link("/#sources")).text("Sources")),
        )
}

fn intro() -> Element {
    let done = ROADMAP.iter().filter(|s| s.status == Status::Done).count();
    section()
        .class("intro")
        .child(p().class("eyebrow").text("Projet en préparation"))
        .child(h1().text("Android libre sur le Nothing Phone (4b), avec le code de l'appareil en Rust."))
        .child(p().class("lead").text(
            "customFirmware porte AOSP, la version libre d'Android, sur le Nothing Phone (4b). Tout ce que le projet écrit lui-même est en Rust : couches matérielles, services et outils.",
        ))
        .child(
            p().class("notice")
                .child(strong().text("Rien n'est encore installable. "))
                .child(text(format!(
                    "{done} étapes sur {} sont faites ; la suivante est le déverrouillage du bootloader.",
                    ROADMAP.len()
                ))),
        )
        .child(
            div()
                .class("row")
                .child(a().class("button").href("#feuille-de-route").text("Voir la feuille de route"))
                .child(a().class("button ghost").href(REPOSITORY).text("Code source")),
        )
}

fn device() -> Element {
    section()
        .id("appareil")
        .child(h2().text("L'appareil"))
        .child(
            p().class("muted")
                .text(format!("Faits vérifiés le {VERIFIED_ON}.")),
        )
        .child(
            div()
                .class("table-wrap")
                .child(table().child(tbody().children(FACTS.iter().map(fact_row)))),
        )
}

fn fact_row(fact: &Fact) -> Element {
    tr().child(th().attr("scope", "row").text(fact.label))
        .child(
            td().child(span().text(fact.value))
                .child(fact.note.map(|note| small().text(note))),
        )
}

fn scope() -> Element {
    section()
        .id("rust")
        .child(h2().text("Ce qui est en Rust, et ce qui ne peut pas l'être"))
        .child(p().class("muted").text(
            "Un téléphone ne peut pas être réécrit entièrement en Rust. Le projet écrit en Rust tout ce qu'il ajoute et reprend le reste tel quel.",
        ))
        .child(
            div()
                .class("grid two")
                .child(part_list("Écrit en Rust", "yes", IN_RUST))
                .child(part_list("Repris tel quel", "no", NOT_IN_RUST)),
        )
}

fn part_list(title: &'static str, kind: &'static str, parts: &'static [Part]) -> Element {
    div()
        .class("card parts")
        .class(kind)
        .child(h3().text(title))
        .child(ul().children(parts.iter().map(|part| {
            li().child(strong().text(part.title))
                .child(span().text(part.detail))
        })))
}

fn roadmap() -> Element {
    section()
        .id("feuille-de-route")
        .child(h2().text("Feuille de route"))
        .child(ol().class("steps").children(ROADMAP.iter().map(step_item)))
}

fn step_item(step: &Step) -> Element {
    li().class(if step.status == Status::Next {
        "current"
    } else {
        ""
    })
    .child(
        div()
            .class("step-text")
            .child(strong().text(step.title))
            .child(span().text(step.detail)),
    )
    .child(span().class(step.status.class()).text(step.status.label()))
}

fn risks() -> Element {
    section().id("risques").child(h2().text("Risques")).child(
        ul().class("risks")
            .children(RISKS.iter().map(|risk| li().text(*risk))),
    )
}

fn references() -> Element {
    section().id("sources").child(h2().text("Sources")).child(
        ul().class("sources").children(
            REFERENCES
                .iter()
                .map(|r| li().child(a().href(r.url).attr("rel", "noopener").text(r.title))),
        ),
    )
}

fn site_footer() -> Element {
    footer()
        .child(
            span()
                .child(text("Site écrit en Rust avec "))
                .child(a().href(PAQUETS).text("Paquets"))
                .child(text(".")),
        )
        .child(span().text("Projet indépendant, sans lien avec Nothing, Qualcomm ou Google."))
}
