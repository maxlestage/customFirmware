//! `cf`, les outils de customFirmware.
//!
//!     cf doctor [dossier]                 l'ordinateur peut-il compiler AOSP ?
//!     cf props getprop.txt [--mk out.mk]  les faits du téléphone
//!     cf blobs --list … --from … --out …  les fichiers propriétaires

mod blobs;
mod doctor;
mod props;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Écrit sur la sortie standard. Si le lecteur a fermé le tube (`cf props … | head`),
/// l'outil s'arrête sans bruit au lieu de paniquer comme le ferait `println!`.
fn out(text: impl AsRef<str>) {
    let mut stdout = std::io::stdout().lock();
    if writeln!(stdout, "{}", text.as_ref()).is_err() {
        std::process::exit(0);
    }
}

const USAGE: &str = "\
cf, les outils de customFirmware

Usage :
  cf doctor [dossier]
      Vérifie l'ordinateur : système, mémoire, espace libre dans le dossier où
      AOSP sera téléchargé (le dossier courant par défaut), outils installés.

  cf props <getprop.txt> [--mk <fichier.mk>]
      Lit la sortie de `adb shell getprop` et affiche les faits utiles au portage.
      Avec --mk, écrit aussi les variables Make CF_… dans le fichier donné.

  cf blobs --list <proprietary-files.txt> --from <système d'origine> --out <dossier>
           [--tree-path <chemin dans AOSP>]
      Copie les fichiers de la liste depuis le système d'origine (un dossier qui
      contient vendor/, odm/…) vers <dossier>/proprietary, et écrit
      <dossier>/blobs.mk. --tree-path est le chemin de <dossier> dans l'arbre
      d'AOSP (vendor/nothing/SuperContra par défaut).

  cf help
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("doctor") => cmd_doctor(&args[1..]),
        Some("props") => cmd_props(&args[1..]),
        Some("blobs") => cmd_blobs(&args[1..]),
        Some("help" | "--help" | "-h") | None => {
            out(USAGE.trim_end());
            Ok(true)
        }
        Some(other) => Err(format!("commande inconnue : {other}\n\n{USAGE}")),
    };
    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(message) => {
            eprintln!("erreur : {message}");
            ExitCode::from(2)
        }
    }
}

/// Les arguments d'une commande : les valeurs des options `--nom valeur`, et le reste.
struct Parsed {
    options: Vec<(String, String)>,
    positional: Vec<String>,
}

impl Parsed {
    fn new(args: &[String], known: &[&str]) -> Result<Self, String> {
        let mut parsed = Parsed {
            options: Vec::new(),
            positional: Vec::new(),
        };
        let mut iter = args.iter();
        while let Some(arg) = iter.next() {
            if let Some(name) = arg.strip_prefix("--") {
                if !known.contains(&name) {
                    return Err(format!("option inconnue : --{name}"));
                }
                let value = iter.next().ok_or(format!("--{name} attend une valeur"))?;
                parsed.options.push((name.to_owned(), value.clone()));
            } else {
                parsed.positional.push(arg.clone());
            }
        }
        Ok(parsed)
    }

    fn option(&self, name: &str) -> Option<&str> {
        self.options
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }

    fn required(&self, name: &str) -> Result<&str, String> {
        self.option(name).ok_or(format!("--{name} est obligatoire"))
    }
}

fn cmd_doctor(args: &[String]) -> Result<bool, String> {
    let parsed = Parsed::new(args, &[])?;
    let dir = parsed.positional.first().map_or(".", String::as_str);
    let checks = doctor::run(Path::new(dir));
    let mut ok = true;
    for check in &checks {
        let mark = match check.level {
            doctor::Level::Ok => "✓",
            doctor::Level::Warning => "!",
            doctor::Level::Failure => {
                ok = false;
                "✗"
            }
        };
        out(format!("{mark} {}", check.message));
    }
    out(if ok {
        "\nL'ordinateur est prêt."
    } else {
        "\nIl manque des éléments : voir les lignes marquées ✗."
    });
    Ok(ok)
}

fn cmd_props(args: &[String]) -> Result<bool, String> {
    let parsed = Parsed::new(args, &["mk"])?;
    let [file] = parsed.positional.as_slice() else {
        return Err("usage : cf props <getprop.txt> [--mk <fichier.mk>]".into());
    };
    let text = std::fs::read_to_string(file).map_err(|e| format!("{file} : {e}"))?;
    let props = props::Props::parse(&text);
    if props.len() == 0 {
        return Err(format!(
            "{file} ne contient aucune propriété : attendu, la sortie de `adb shell getprop`"
        ));
    }
    out(props::report(&props).trim_end());
    if let Some(mk) = parsed.option("mk") {
        std::fs::write(mk, props::facts_mk(&props)).map_err(|e| format!("{mk} : {e}"))?;
        out(format!("\nVariables Make écrites dans {mk}"));
    }
    Ok(true)
}

fn cmd_blobs(args: &[String]) -> Result<bool, String> {
    let parsed = Parsed::new(args, &["list", "from", "out", "tree-path"])?;
    if !parsed.positional.is_empty() {
        return Err(format!("argument inattendu : {}", parsed.positional[0]));
    }
    let list = parsed.required("list")?;
    let from = PathBuf::from(parsed.required("from")?);
    let out_dir = PathBuf::from(parsed.required("out")?);
    let tree_path = parsed
        .option("tree-path")
        .unwrap_or("vendor/nothing/SuperContra");

    let text = std::fs::read_to_string(list).map_err(|e| format!("{list} : {e}"))?;
    let entries = blobs::parse_list(&text).map_err(|e| format!("{list}, {e}"))?;
    if !from.is_dir() {
        return Err(format!("{} n'est pas un dossier", from.display()));
    }
    let outcome = blobs::extract(&entries, &from, &out_dir).map_err(|e| e.to_string())?;
    let mk = out_dir.join("blobs.mk");
    std::fs::create_dir_all(&out_dir).map_err(|e| format!("{} : {e}", out_dir.display()))?;
    std::fs::write(&mk, blobs::vendor_mk(&outcome.copied, tree_path))
        .map_err(|e| format!("{} : {e}", mk.display()))?;

    out(format!(
        "Copiés dans {} : {}. Introuvables : {}.",
        out_dir.join("proprietary").display(),
        outcome.copied.len(),
        outcome.missing.len()
    ));
    for entry in &outcome.missing {
        out(format!("  introuvable : {}", entry.src));
    }
    out(format!("Fragment Make : {}", mk.display()));
    Ok(outcome.missing.is_empty())
}
