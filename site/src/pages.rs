//! Les pages du site. Chaque composant est une fonction qui renvoie une vue.

use paquets::prelude::*;

use crate::art;
use crate::content::{
    FACTS, Fact, IN_RUST, NOT_IN_RUST, Part, REFERENCES, RISKS, ROADMAP, Status, Step, TICKER,
    VERIFIED_ON,
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
pub fn link(path: &str) -> String {
    format!("{}{path}", base())
}

/// L'icône et la feuille de style, communes aux deux pages.
pub fn assets(page: Page) -> Page {
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
        .body(div().class("scroll-progress").attr("aria-hidden", "true"))
        .body(site_header())
        .body(
            main()
                .child(intro())
                .child(ticker())
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
                    .class("intro lost")
                    .child(
                        p().class("big-404")
                            .attr("aria-label", "Erreur 404")
                            .children(
                                ["4", "0", "4"]
                                    .into_iter()
                                    .enumerate()
                                    .map(|(i, digit)| span().style(format!("--i:{i}")).text(digit)),
                            ),
                    )
                    .child(h1().style("--i:3").text("Cette page n'existe pas."))
                    .child(
                        a().class("button")
                            .style("--i:4")
                            .href(link("/"))
                            .text("Retour à l'accueil"),
                    ),
            ),
        )
        .body(site_footer())
}

pub fn site_header() -> Element {
    header()
        .child(
            a().class("brand")
                .href(link("/"))
                .child(text("customFirmware"))
                .child(span().class("caret").attr("aria-hidden", "true").text("_")),
        )
        .child(
            nav()
                .attr("aria-label", "Sections")
                .child(a().href(link("/#appareil")).text("Appareil"))
                .child(a().href(link("/#rust")).text("Rust"))
                .child(
                    a().href(link("/#feuille-de-route"))
                        .text("Feuille de route"),
                )
                .child(a().href(link("/#sources")).text("Sources"))
                .child(
                    a().class("nav-guide")
                        .href(crate::guide_page::href())
                        .text("Guide"),
                ),
        )
}

/// Un titre de section, avec son trait animé.
fn heading(title: &'static str) -> Element {
    h2().class("reveal").text(title)
}

fn intro() -> Element {
    let done = ROADMAP.iter().filter(|s| s.status == Status::Done).count();
    section()
        .class("intro")
        .child(
            div()
                .class("intro-text")
                .child(p().class("eyebrow").style("--i:0").text("Projet en préparation"))
                .child(
                    h1().style("--i:1")
                        .child(text("Android libre sur le Nothing Phone (4b), avec le code de l'appareil en "))
                        .child(span().class("shine").text("Rust"))
                        .child(text(".")),
                )
                .child(p().class("lead").style("--i:2").text(
                    "customFirmware porte AOSP, la version libre d'Android, sur le Nothing Phone (4b). Tout ce que le projet écrit lui-même est en Rust : couches matérielles, services et outils.",
                ))
                .child(
                    div()
                        .class("notice")
                        .style("--i:3")
                        .child(art::progress_ring(done, ROADMAP.len()))
                        .child(
                            p().child(strong().text("Rien n'est encore installable. "))
                                .child(text(format!(
                                    "{done} étapes sur {} sont faites ; la suivante est le déverrouillage du bootloader.",
                                    ROADMAP.len()
                                ))),
                        ),
                )
                .child(
                    div()
                        .class("row")
                        .style("--i:4")
                        .child(a().class("button").href(crate::guide_page::href()).text("Lire le guide"))
                        .child(a().class("button ghost").href("#feuille-de-route").text("Feuille de route"))
                        .child(a().class("button ghost").href(REPOSITORY).text("Code source")),
                ),
        )
        .child(div().class("intro-art").child(art::hero()))
}

/// Les mots qui défilent : la liste est écrite deux fois, pour boucler sans saut.
fn ticker() -> Element {
    let words = |hidden: bool| {
        div()
            .class("ticker-group")
            .attr("aria-hidden", if hidden { "true" } else { "false" })
            .children(TICKER.iter().map(|word| span().text(*word)))
    };
    div().class("ticker").child(
        div()
            .class("ticker-track")
            .child(words(false))
            .child(words(true)),
    )
}

fn device() -> Element {
    section()
        .id("appareil")
        .child(heading("L'appareil"))
        .child(
            p().class("muted reveal")
                .text(format!("Faits vérifiés le {VERIFIED_ON}.")),
        )
        .child(
            div()
                .class("table-wrap")
                .child(table().child(tbody().children(FACTS.iter().map(fact_row)))),
        )
}

fn fact_row(fact: &Fact) -> Element {
    tr().class("reveal")
        .child(th().attr("scope", "row").text(fact.label))
        .child(
            td().child(span().text(fact.value))
                .child(fact.note.map(|note| small().text(note))),
        )
}

fn scope() -> Element {
    section()
        .id("rust")
        .child(heading("Ce qui est en Rust, et ce qui ne peut pas l'être"))
        .child(p().class("muted reveal").text(
            "Un téléphone ne peut pas être réécrit entièrement en Rust. Le projet écrit en Rust tout ce qu'il ajoute sous l'interface (services, outils), en Kotlin ses applications, et reprend le reste tel quel.",
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
        .class("card parts reveal")
        .class(kind)
        .child(h3().text(title))
        .child(ul().children(parts.iter().enumerate().map(|(i, part)| {
            li().style(format!("--i:{i}"))
                .child(strong().text(part.title))
                .child(span().text(part.detail))
        })))
}

fn roadmap() -> Element {
    section()
        .id("feuille-de-route")
        .child(heading("Feuille de route"))
        .child(
            ol().class("steps")
                .children(ROADMAP.iter().enumerate().map(step_item)),
        )
}

fn step_item((i, step): (usize, &Step)) -> Element {
    let marker = span().class("marker").attr("aria-hidden", "true");
    let marker = if step.status == Status::Done {
        marker.child(art::check())
    } else {
        marker.text(format!("{:02}", i + 1))
    };
    li().class("reveal")
        .class(match step.status {
            Status::Done => "done",
            Status::Next => "current",
            Status::Todo => "todo",
        })
        .child(marker)
        .child(
            div()
                .class("step-text")
                .child(strong().text(step.title))
                .child(span().text(step.detail)),
        )
        .child(span().class(step.status.class()).text(step.status.label()))
}

fn risks() -> Element {
    section()
        .id("risques")
        .child(heading("Risques"))
        .child(
            ul().class("risks")
                .children(RISKS.iter().enumerate().map(|(i, risk)| {
                    li().class("reveal")
                        .style(format!("--i:{i}"))
                        .child(art::warning())
                        .child(span().text(*risk))
                })),
        )
}

fn references() -> Element {
    section()
        .id("sources")
        .child(heading("Sources"))
        .child(ul().class("sources").children(REFERENCES.iter().map(|r| {
            li().class("reveal")
                .child(a().href(r.url).attr("rel", "noopener").text(r.title))
        })))
}

pub fn site_footer() -> Element {
    footer()
        .child(
            span()
                .child(text("Site écrit en Rust avec "))
                .child(a().href(PAQUETS).text("Paquets"))
                .child(text(".")),
        )
        .child(span().text("Projet indépendant, sans lien avec Nothing, Qualcomm ou Google."))
}
