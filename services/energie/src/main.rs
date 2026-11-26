//! `energie` : règle le noyau du téléphone pour économiser la batterie.
//!
//! Le noyau garde son code ; energie change ses réglages (/sys, /proc/sys) selon des
//! profils, passe d'un profil à l'autre selon l'écran et la batterie, et mesure ce que
//! chaque profil consomme. Voir README.md.

mod capteurs;
mod config;
mod demon;
mod journal;
mod modeles;
mod profil;
mod systeme;

use capteurs::{Batterie, Ecran};
use profil::Profil;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;
use systeme::Systeme;

const USAGE: &str = "\
energie, les réglages d'énergie du noyau

Usage : energie [--dossier <dossier>] <commande>

  etat                 batterie, écran, profil actif, fréquences des processeurs
  lire <motif>…        les réglages du noyau qui correspondent (`*` accepté), et leur valeur
  liste                les profils disponibles
  verifier <profil>    ce que le profil changerait, sans rien écrire
  appliquer <profil>   applique un profil (il faut être root)
  restaurer            remet tous les réglages d'origine (= appliquer origine)
  forcer <profil>      impose un profil au démon, quoi que disent les règles
  forcer auto          rend la main aux règles
  demon                choisit et applique les profils tout seul, tient le journal
  rapport [journal]    ce que chaque profil consomme, d'après le journal

Le dossier contient energie.conf, profils/ et ce qu'energie écrit (origine.etat,
journal.csv) ; par défaut $ENERGIE_DOSSIER, sinon /data/misc/energie. Les fichiers
qui y manquent sont copiés depuis les modèles du système (--modeles, par défaut
/system_ext/etc/energie). Le démon relit tout dès qu'un fichier change.";

/// Écrit sur la sortie standard ; s'arrête sans bruit si le tube est fermé (`| head`).
fn out(texte: impl AsRef<str>) {
    let mut stdout = std::io::stdout().lock();
    if writeln!(stdout, "{}", texte.as_ref()).is_err() {
        std::process::exit(0);
    }
}

struct Contexte {
    sys: Systeme,
    dossier: PathBuf,
    batterie: String,
    ecran: String,
}

fn main() -> ExitCode {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let mut option = |nom: &str| -> Option<String> {
        let i = args.iter().position(|a| a == nom)?;
        let valeur = args.get(i + 1).cloned();
        args.drain(i..(i + 2).min(args.len()));
        valeur
    };
    let dossier = option("--dossier")
        .or_else(|| std::env::var("ENERGIE_DOSSIER").ok())
        .unwrap_or_else(|| "/data/misc/energie".into());
    let modeles = option("--modeles").unwrap_or_else(|| "/system_ext/etc/energie".into());
    // Pour essayer sur un faux arbre /sys (tests, ordinateur).
    let sys = option("--racine").map_or_else(Systeme::reel, Systeme::avec_racine);
    let dossier = PathBuf::from(dossier);
    let resultat = modeles::installer(PathBuf::from(modeles).as_path(), &dossier)
        .and_then(|_| demon::Regles::charger(&dossier))
        .and_then(|regles| {
            let ctx = Contexte {
                sys,
                dossier,
                batterie: regles.batterie,
                ecran: regles.ecran,
            };
            executer(&ctx, &args)
        });
    match resultat {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("energie : {e}");
            ExitCode::FAILURE
        }
    }
}

fn executer(ctx: &Contexte, args: &[String]) -> Result<bool, String> {
    let argument = |quoi: &str| {
        args.get(1)
            .map(String::as_str)
            .ok_or_else(|| format!("il manque {quoi}\n\n{USAGE}"))
    };
    match args.first().map(String::as_str) {
        Some("etat") => etat(ctx),
        Some("lire") if args.len() > 1 => lire(ctx, &args[1..]),
        Some("liste") => liste(ctx),
        Some("verifier") => verifier(ctx, argument("le nom du profil")?),
        Some("appliquer") => appliquer(ctx, argument("le nom du profil")?),
        Some("restaurer") => appliquer(ctx, profil::ORIGINE),
        Some("forcer") => forcer(ctx, argument("un nom de profil, ou auto")?),
        Some("demon") => demon::lancer(&ctx.sys, &ctx.dossier).map(|()| true),
        Some("rapport") => {
            let fichier = args
                .get(1)
                .map(PathBuf::from)
                .unwrap_or_else(|| ctx.dossier.join("journal.csv"));
            let texte = std::fs::read_to_string(&fichier)
                .map_err(|e| format!("{} : {e}", fichier.display()))?;
            out(journal::rapport(&texte));
            Ok(true)
        }
        Some("help" | "--help" | "-h") | None => {
            out(USAGE);
            Ok(true)
        }
        Some(autre) => Err(format!(
            "commande inconnue ou incomplète : {autre}\n\n{USAGE}"
        )),
    }
}

fn mhz(khz: Option<i64>) -> String {
    khz.map(|k| format!("{} MHz", k / 1000))
        .unwrap_or_else(|| "?".into())
}

fn etat(ctx: &Contexte) -> Result<bool, String> {
    let sys = &ctx.sys;
    let b = Batterie::lire(sys, &ctx.batterie);
    let mut ligne = format!(
        "Batterie : {} · {}",
        b.capacite.map(|c| format!("{c} %")).unwrap_or("?".into()),
        if b.statut.is_empty() { "?" } else { &b.statut }
    );
    if let Some(p) = b.puissance_mw() {
        ligne.push_str(&format!(" · {p} mW"));
    }
    if let Some(t) = b.temperature {
        ligne.push_str(&format!(" · {:.1} °C", t as f64 / 10.0));
    }
    out(ligne);
    out(format!("Écran : {}", Ecran::lire(sys, &ctx.ecran).nom()));
    out(format!(
        "Profil actif : {}{}",
        profil::actif(&ctx.dossier).unwrap_or_else(|| "aucun (réglages d'origine)".into()),
        demon::force(&ctx.dossier)
            .map(|f| format!(" (forcé : {f})"))
            .unwrap_or_default()
    ));
    out("Processeurs :");
    for policy in sys.developper("/sys/devices/system/cpu/cpufreq/policy*") {
        let f = |nom: &str| sys.nombre(&format!("{policy}/{nom}"));
        out(format!(
            "  {:<8} cœurs {:<8} {:<10} {} – {} (max matériel {}, actuel {})",
            policy.rsplit('/').next().unwrap_or(""),
            sys.lire(&format!("{policy}/related_cpus"))
                .unwrap_or_default(),
            sys.lire(&format!("{policy}/scaling_governor"))
                .unwrap_or_default(),
            mhz(f("scaling_min_freq")),
            mhz(f("scaling_max_freq")),
            mhz(f("cpuinfo_max_freq")),
            mhz(f("scaling_cur_freq")),
        ));
    }
    let gpu = "/sys/class/kgsl/kgsl-3d0";
    if sys.existe(gpu) {
        out(format!(
            "GPU : {} (niveaux de puissance {} à {})",
            sys.nombre(&format!("{gpu}/gpuclk"))
                .map(|hz| format!("{} MHz", hz / 1_000_000))
                .unwrap_or("?".into()),
            sys.lire(&format!("{gpu}/max_pwrlevel"))
                .unwrap_or("?".into()),
            sys.lire(&format!("{gpu}/min_pwrlevel"))
                .unwrap_or("?".into()),
        ));
    }
    Ok(true)
}

fn lire(ctx: &Contexte, motifs: &[String]) -> Result<bool, String> {
    let mut trouve = false;
    for motif in motifs {
        for chemin in ctx.sys.developper(motif) {
            trouve = true;
            let valeur = match ctx.sys.lire(&chemin) {
                Ok(v) if v.contains('\n') => format!("{} …", v.lines().next().unwrap_or("")),
                Ok(v) => v,
                Err(e) => format!("(illisible : {e})"),
            };
            out(format!("{chemin} = {valeur}"));
        }
    }
    if !trouve {
        eprintln!("energie : aucun fichier ne correspond");
    }
    Ok(trouve)
}

fn liste(ctx: &Contexte) -> Result<bool, String> {
    let actif = profil::actif(&ctx.dossier);
    for nom in profil::liste(&ctx.dossier) {
        let marque = if actif.as_deref() == Some(&nom) {
            "*"
        } else {
            " "
        };
        let description = Profil::charger(&ctx.dossier, &nom)
            .map(|p| p.description)
            .unwrap_or_else(|e| format!("(illisible : {e})"));
        out(format!("{marque} {nom:<16} {description}"));
    }
    Ok(true)
}

fn verifier(ctx: &Contexte, nom: &str) -> Result<bool, String> {
    let p = Profil::charger(&ctx.dossier, nom)?;
    out(format!("{nom} : {}", p.description));
    let (cibles, avertissements) = p.resoudre(&ctx.sys);
    for (chemin, valeur) in &cibles {
        let actuelle = ctx.sys.lire(chemin).unwrap_or_else(|_| "?".into());
        let fleche = if actuelle == *valeur {
            "  (déjà)"
        } else {
            ""
        };
        out(format!("  {chemin} : {actuelle} → {valeur}{fleche}"));
    }
    for a in &avertissements {
        out(format!("  attention, {a}"));
    }
    out(format!(
        "{} réglage(s), {} avertissement(s). Rien n'a été écrit.",
        cibles.len(),
        avertissements.len()
    ));
    Ok(true)
}

fn appliquer(ctx: &Contexte, nom: &str) -> Result<bool, String> {
    let p = Profil::charger(&ctx.dossier, nom)?;
    std::fs::create_dir_all(&ctx.dossier)
        .map_err(|e| format!("{} : {e}", ctx.dossier.display()))?;
    let bilan = profil::appliquer(&ctx.sys, &ctx.dossier, &p)?;
    for (chemin, avant, apres) in &bilan.changes {
        out(format!("  {chemin} : {avant} → {apres}"));
    }
    for a in &bilan.avertissements {
        out(format!("  attention, {a}"));
    }
    for e in &bilan.erreurs {
        out(format!("  erreur : {e}"));
    }
    out(format!(
        "Profil {nom} : {} changé(s), {} déjà bon(s), {} erreur(s).",
        bilan.changes.len(),
        bilan.inchanges,
        bilan.erreurs.len()
    ));
    Ok(bilan.erreurs.is_empty())
}

fn forcer(ctx: &Contexte, nom: &str) -> Result<bool, String> {
    let fichier = ctx.dossier.join("force");
    if nom == "auto" {
        match std::fs::remove_file(&fichier) {
            Ok(()) => out("Le démon suit de nouveau les règles."),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                out("Aucun profil n'était forcé.")
            }
            Err(e) => return Err(format!("{} : {e}", fichier.display())),
        }
        return Ok(true);
    }
    Profil::charger(&ctx.dossier, nom)?;
    profil::ecrire_partage(&fichier, nom).map_err(|e| format!("{} : {e}", fichier.display()))?;
    out(format!(
        "Le démon applique {nom} (dans les 2 s s'il tourne) ; `energie forcer auto` pour revenir aux règles."
    ));
    Ok(true)
}
