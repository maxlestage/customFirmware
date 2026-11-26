//! La page du guide : un sommaire, puis chaque étape avec ses commandes, ses
//! vérifications et ses mises en garde. Le contenu est dans `guide.rs`.

use paquets::prelude::*;

use crate::guide::{Block, GUIDE_UPDATED, INTRO, Os, STEPS, Step, Verified};
use crate::pages::{assets, link, site_footer, site_header};

/// `/guide/` : la page d'Ubuntu.
pub async fn guide(_req: Request) -> Page {
    page(Os::Ubuntu)
}

/// `/guide/<système>/` : la page d'un autre système, ou 404.
pub async fn guide_for(req: Request) -> Page {
    match req.param("systeme").and_then(Os::from_slug) {
        Some(os) => page(os),
        None => crate::pages::not_found(req).await,
    }
}

/// L'adresse de la page d'un système.
pub fn path(os: Os) -> String {
    match os {
        Os::Ubuntu => "/guide/".into(),
        os => format!("/guide/{}/", os.slug()),
    }
}

fn page(os: Os) -> Page {
    let title = match os {
        Os::Ubuntu => "Guide : AOSP et Rust sur le Nothing Phone (4b) · customFirmware".into(),
        os => format!(
            "Guide pour {} : AOSP et Rust sur le Nothing Phone (4b) · customFirmware",
            os.label()
        ),
    };
    assets(Page::new(title))
        .lang("fr")
        .description(
            "Le guide pas à pas du projet customFirmware : sauvegarder, déverrouiller, extraire, compiler AOSP, écrire un HAL en Rust et installer sur le Nothing Phone (4b).",
        )
        .body(div().class("scroll-progress").attr("aria-hidden", "true"))
        .body(site_header())
        .body(
            main().class("guide").child(guide_intro(os)).child(
                div()
                    .class("guide-layout")
                    .child(toc_mobile())
                    .child(toc(os))
                    .child(
                        div()
                            .class("guide-steps")
                            .children(STEPS.iter().enumerate().map(|s| step(s, os))),
                    ),
            ),
        )
        .body(site_footer())
}

fn guide_intro(os: Os) -> Element {
    section()
        .class("guide-intro")
        .child(p().class("eyebrow").style("--i:0").text("Guide pas à pas"))
        .child(
            h1().style("--i:1").text(
                "Installer AOSP sur le Nothing Phone (4b), avec le code de l'appareil en Rust",
            ),
        )
        .child(p().class("lead").style("--i:2").text(INTRO))
        .child(os_picker(os))
        .child(
            ul().class("legend")
                .style("--i:4")
                .children(Verified::ALL.map(|v| {
                    li().child(span().class(v.class()).text(v.label()))
                        .child(span().text(v.meaning()))
                })),
        )
        .child(
            p().class("muted")
                .style("--i:5")
                .text(format!("Mis à jour le {GUIDE_UPDATED}.")),
        )
}

/// Le choix du système : un lien par page, sans JavaScript. La page choisie se garde
/// dans les favoris.
fn os_picker(current: Os) -> Element {
    nav()
        .class("os-picker")
        .id("systeme")
        .style("--i:3")
        .attr("aria-label", "Système de votre ordinateur")
        .child(p().class("os-picker-title").text("Votre ordinateur"))
        .child(ul().children(Os::ALL.map(|os| {
            let lien = a()
                // L'ancre garde le lecteur au sélecteur après le changement de page.
                .href(format!("{}#systeme", link(&path(os))))
                .child(strong().text(os.label()))
                .child(span().text(os.family()));
            li().child(if os == current {
                lien.attr("aria-current", "page")
            } else {
                lien
            })
        })))
        .child(p().class("muted os-picker-note").text(
            "Une autre distribution : partez de celle dont elle dérive ; seuls les noms des paquets changent. Windows : installez Ubuntu dans WSL 2, puis suivez la page d'Ubuntu.",
        ))
}

/// La liste des étapes, avec leur numéro.
fn toc_list() -> Element {
    ol().class("toc-list")
        .children(STEPS.iter().enumerate().map(|(i, s)| {
            li().child(
                a().href(format!("#{}", s.id))
                    .child(span().class("toc-num").text(format!("{:02}", i)))
                    .child(text(s.title)),
            )
        }))
}

/// Le sommaire sur téléphone : un menu qui se déplie, sans JavaScript.
fn toc_mobile() -> Element {
    details()
        .class("toc-mobile")
        .child(summary().text(format!("Les {} étapes", STEPS.len())))
        .child(toc_list())
}

/// Le sommaire sur grand écran : une colonne qui reste visible.
fn toc(os: Os) -> Element {
    nav()
        .class("toc")
        .attr("aria-label", "Étapes du guide")
        .child(
            a().class("toc-os")
                .href("#systeme")
                .child(span().class("toc-title").text("Système"))
                .child(span().text(os.label())),
        )
        .child(p().class("toc-title").text("Étapes"))
        .child(toc_list())
}

fn step((i, s): (usize, &Step), os: Os) -> Element {
    section()
        .class("guide-step")
        .id(s.id)
        .child(
            div()
                .class("step-head reveal")
                .child(
                    span()
                        .class("step-num")
                        .attr("aria-hidden", "true")
                        .text(format!("{:02}", i)),
                )
                .child(
                    div().child(h2().text(s.title)).child(
                        p().class("step-meta")
                            .child(span().class(s.verified.class()).text(s.verified.label()))
                            .child(span().class("muted").text(s.time)),
                    ),
                ),
        )
        .child(p().class("step-goal").text(s.goal))
        .children(blocks(s.blocks, os))
}

/// Les blocs à afficher pour ce système : ceux des `For` qui ne le visent pas sont omis.
fn blocks(list: &[Block], os: Os) -> Vec<Node> {
    list.iter()
        .flat_map(|b| match b {
            Block::For(systems, inner) if systems.contains(&os) => blocks(inner, os),
            Block::For(..) => Vec::new(),
            b => vec![block(b)],
        })
        .collect()
}

fn block(b: &Block) -> Node {
    match b {
        // Déjà déplié par `blocks`.
        Block::For(..) => text(""),
        Block::Text(t) => p().text(*t).into(),
        Block::List(items) => ul()
            .class("guide-list")
            .children(items.iter().map(|t| li().text(*t)))
            .into(),
        Block::Steps(items) => ol()
            .class("guide-list")
            .children(items.iter().map(|t| li().text(*t)))
            .into(),
        Block::Code { title, code } => figure()
            .class("terminal")
            .child(
                figcaption()
                    .child(span().class("dots").attr("aria-hidden", "true"))
                    .child(span().text(*title)),
            )
            .child(pre().child(code_el(code)))
            .into(),
        Block::File { path, code } => figure()
            .class("terminal file")
            .child(
                figcaption()
                    .child(span().class("dots").attr("aria-hidden", "true"))
                    .child(span().text(*path)),
            )
            .child(pre().child(code_el(code)))
            .into(),
        Block::Warning(t) => aside()
            .class("callout warning")
            .child(strong().text("Attention. "))
            .child(text(*t))
            .into(),
        Block::Tip(t) => aside()
            .class("callout tip")
            .child(strong().text("Astuce. "))
            .child(text(*t))
            .into(),
        Block::Check(t) => aside()
            .class("callout verify")
            .child(strong().text("Vérifier. "))
            .child(text(*t))
            .into(),
        Block::Source { title, url } => p()
            .class("step-source")
            .child(text("Source : "))
            .child(a().href(*url).attr("rel", "noopener").text(*title))
            .into(),
    }
}

/// Un bloc de code, tel quel : le texte est échappé par Active.
fn code_el(source: &str) -> Element {
    code().text(source.trim_matches('\n').to_owned())
}

/// Le lien vers le guide, pour l'en-tête et l'accueil.
pub fn href() -> String {
    link("/guide/")
}
