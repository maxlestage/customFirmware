//! Les illustrations du site, en SVG 2D dessiné par Rust. Les animations sont dans
//! public/site.css : chaque forme porte une classe (`draw`, `glyph`, `float`…) et,
//! pour les décaler dans le temps, une variable CSS `--i`.

use paquets::prelude::*;

/// Le chemin d'un rectangle aux coins arrondis, pour qu'il puisse se « dessiner »
/// (un `<rect>` n'a pas de longueur de tracé fiable partout).
fn rounded_rect(x: f32, y: f32, w: f32, h: f32, r: f32) -> String {
    format!(
        "M{} {y} H{} A{r} {r} 0 0 1 {} {} V{} A{r} {r} 0 0 1 {} {} H{} A{r} {r} 0 0 1 {x} {} V{} A{r} {r} 0 0 1 {} {y} Z",
        x + r,
        x + w - r,
        x + w,
        y + r,
        y + h - r,
        x + w - r,
        y + h,
        x + r,
        y + h - r,
        y + r,
        x + r,
    )
}

/// Le contour d'un engrenage centré sur (0, 0) : `teeth` dents entre les rayons
/// `inner` et `outer`, et un trou au milieu (règle de remplissage evenodd).
fn gear(teeth: usize, inner: f32, outer: f32, hole: f32) -> String {
    let step = std::f32::consts::TAU / teeth as f32;
    let mut d = String::new();
    for i in 0..teeth {
        let a = i as f32 * step;
        // Une dent : le pied, le sommet (un peu plus étroit), puis le pied suivant.
        let points = [
            (inner, a),
            (outer, a + step * 0.12),
            (outer, a + step * 0.42),
            (inner, a + step * 0.54),
        ];
        for (j, (r, angle)) in points.iter().enumerate() {
            let cmd = if i == 0 && j == 0 { 'M' } else { 'L' };
            d.push_str(&format!(
                "{cmd}{:.2} {:.2} ",
                r * angle.cos(),
                r * angle.sin()
            ));
        }
    }
    d.push_str(&format!(
        "Z M{hole} 0 A{hole} {hole} 0 1 0 -{hole} 0 A{hole} {hole} 0 1 0 {hole} 0 Z"
    ));
    d
}

/// Une forme décalée dans le temps de `i` crans.
fn delayed(element: Element, i: usize) -> Element {
    element.style(format!("--i:{i}"))
}

/// L'illustration de la page d'accueil : le dos d'un téléphone qui se dessine, ses
/// bandes lumineuses qui s'allument tour à tour, un engrenage, une orbite et des pixels.
pub fn hero() -> Element {
    let pixels = [
        (28.0, 70.0, 0),
        (262.0, 46.0, 1),
        (40.0, 300.0, 2),
        (286.0, 250.0, 3),
        (62.0, 190.0, 4),
        (252.0, 150.0, 5),
        (20.0, 380.0, 6),
    ];
    svg()
        .class("art")
        .attr("viewBox", "0 0 320 420")
        .attr("role", "img")
        .attr(
            "aria-label",
            "Le dos d'un téléphone dessiné au trait, avec des bandes lumineuses et un engrenage",
        )
        // L'orbite, derrière le téléphone.
        .child(
            circle()
                .class("orbit-path")
                .attr("cx", "160")
                .attr("cy", "200")
                .attr("r", "135"),
        )
        .child(
            g().class("orbit").child(
                circle()
                    .class("satellite")
                    .attr("cx", "295")
                    .attr("cy", "200")
                    .attr("r", "6"),
            ),
        )
        // Les pixels qui flottent.
        .children(pixels.iter().map(|&(x, y, i)| {
            delayed(
                rect()
                    .class("float")
                    .attr("x", format!("{x}"))
                    .attr("y", format!("{y}"))
                    .attr("width", "9")
                    .attr("height", "9")
                    .attr("rx", "2"),
                i,
            )
        }))
        // Le téléphone.
        .child(
            path()
                .class("body")
                .attr("d", rounded_rect(92.0, 45.0, 136.0, 300.0, 28.0)),
        )
        .child(delayed(
            path()
                .class("draw")
                .attr("pathLength", "1")
                .attr("d", rounded_rect(92.0, 45.0, 136.0, 300.0, 28.0)),
            0,
        ))
        .child(delayed(
            path()
                .class("draw")
                .attr("pathLength", "1")
                .attr("d", rounded_rect(108.0, 62.0, 44.0, 74.0, 16.0)),
            2,
        ))
        .child(delayed(
            circle()
                .class("draw")
                .attr("pathLength", "1")
                .attr("cx", "130")
                .attr("cy", "84")
                .attr("r", "11"),
            3,
        ))
        .child(delayed(
            circle()
                .class("draw")
                .attr("pathLength", "1")
                .attr("cx", "130")
                .attr("cy", "114")
                .attr("r", "11"),
            4,
        ))
        // Les bandes lumineuses.
        .child(delayed(
            path()
                .class("glyph")
                .attr("pathLength", "1")
                .attr("d", "M178 70 L206 98"),
            0,
        ))
        .child(delayed(
            path()
                .class("glyph")
                .attr("pathLength", "1")
                .attr("d", "M160 160 A42 42 0 1 1 118 202"),
            1,
        ))
        .child(delayed(
            path()
                .class("glyph")
                .attr("pathLength", "1")
                .attr("d", "M160 268 V306"),
            2,
        ))
        .child(delayed(
            circle()
                .class("glyph dot")
                .attr("cx", "204")
                .attr("cy", "316")
                .attr("r", "4"),
            3,
        ))
        // L'engrenage, qui tourne.
        .child(
            g().attr("transform", "translate(250 352)").child(
                path()
                    .class("gear")
                    .attr("fill-rule", "evenodd")
                    .attr("d", gear(12, 26.0, 34.0, 10.0)),
            ),
        )
        .child(
            g().attr("transform", "translate(206 392)").child(
                path()
                    .class("gear small")
                    .attr("fill-rule", "evenodd")
                    .attr("d", gear(8, 13.0, 19.0, 5.0)),
            ),
        )
        // Un bout de code qui flotte.
        .child(
            g().class("chip").child(
                g().attr("transform", "translate(8 222)")
                    .child(
                        rect()
                            .attr("width", "84")
                            .attr("height", "28")
                            .attr("rx", "8"),
                    )
                    .child(
                        text_svg()
                            .attr("x", "42")
                            .attr("y", "18.5")
                            .attr("text-anchor", "middle")
                            .text("fn main()"),
                    ),
            ),
        )
}

/// Un anneau de progression : `done` sur `total`, qui se remplit au chargement.
pub fn progress_ring(done: usize, total: usize) -> Element {
    let percent = (done * 100).checked_div(total).unwrap_or(0);
    svg()
        .class("ring")
        .attr("viewBox", "0 0 48 48")
        .attr("aria-hidden", "true")
        .child(
            circle()
                .class("track")
                .attr("cx", "24")
                .attr("cy", "24")
                .attr("r", "19"),
        )
        .child(
            circle()
                .class("bar")
                .attr("cx", "24")
                .attr("cy", "24")
                .attr("r", "19")
                .attr("pathLength", "100")
                .attr("transform", "rotate(-90 24 24)")
                .style(format!("--p:{percent}")),
        )
        .child(
            text_svg()
                .attr("x", "24")
                .attr("y", "28")
                .attr("text-anchor", "middle")
                .text(format!("{done}/{total}")),
        )
}

/// Une coche qui se trace, pour les étapes faites.
pub fn check() -> Element {
    svg()
        .class("check")
        .attr("viewBox", "0 0 24 24")
        .attr("aria-hidden", "true")
        .child(
            path()
                .attr("pathLength", "1")
                .attr("d", "M5 12.5 L10 17.5 L19 7"),
        )
}

/// Un triangle d'avertissement qui oscille.
pub fn warning() -> Element {
    svg()
        .class("warn")
        .attr("viewBox", "0 0 24 24")
        .attr("aria-hidden", "true")
        .child(path().attr("d", "M12 3 L22 20 H2 Z"))
        .child(path().attr("d", "M12 10 V14 M12 17 V17.2"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gear_has_one_point_group_per_tooth() {
        let d = gear(8, 10.0, 14.0, 3.0);
        assert_eq!(d.matches('L').count(), 8 * 4 - 1);
        assert!(d.starts_with('M') && d.contains('Z'));
    }

    #[test]
    fn ring_shows_the_fraction() {
        let html = Node::from(progress_ring(2, 9)).render();
        assert!(html.contains("--p:22"));
        assert!(html.contains("2/9"));
    }
}
