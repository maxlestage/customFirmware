//! La page du guide : un sommaire, puis chaque étape avec ses commandes, ses
//! vérifications et ses mises en garde. Le contenu est dans `guide.rs`.

use paquets::prelude::*;

use crate::guide::{Block, GUIDE_UPDATED, INTRO, STEPS, Step, Verified};
use crate::pages::{assets, link, site_footer, site_header};

pub async fn guide(_req: Request) -> Page {
    assets(Page::new("Guide : AOSP et Rust sur le Nothing Phone (4b) · customFirmware"))
        .lang("fr")
        .description(
            "Le guide pas à pas du projet customFirmware : sauvegarder, déverrouiller, extraire, compiler AOSP, écrire un HAL en Rust et installer sur le Nothing Phone (4b).",
        )
        .body(div().class("scroll-progress").attr("aria-hidden", "true"))
        .body(site_header())
        .body(
            main().class("guide").child(guide_intro()).child(
                div()
                    .class("guide-layout")
                    .child(toc_mobile())
                    .child(toc())
                    .child(div().class("guide-steps").children(STEPS.iter().enumerate().map(step))),
            ),
        )
        .body(site_footer())
}

fn guide_intro() -> Element {
    section()
        .class("guide-intro")
        .child(p().class("eyebrow").style("--i:0").text("Guide pas à pas"))
        .child(
            h1().style("--i:1").text(
                "Installer AOSP sur le Nothing Phone (4b), avec le code de l'appareil en Rust",
            ),
        )
        .child(p().class("lead").style("--i:2").text(INTRO))
        .child(ul().class("legend").style("--i:3").children(
            [Verified::Yes, Verified::Partly, Verified::No].map(|v| {
                li().child(span().class(v.class()).text(v.label()))
                    .child(span().text(v.meaning()))
            }),
        ))
        .child(
            p().class("muted")
                .style("--i:4")
                .text(format!("Mis à jour le {GUIDE_UPDATED}.")),
        )
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
fn toc() -> Element {
    nav()
        .class("toc")
        .attr("aria-label", "Étapes du guide")
        .child(p().class("toc-title").text("Étapes"))
        .child(toc_list())
}

fn step((i, s): (usize, &Step)) -> Element {
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
        .children(s.blocks.iter().map(block))
}

fn block(b: &Block) -> Node {
    match b {
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
