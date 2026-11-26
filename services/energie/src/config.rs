//! Le format des fichiers de réglages, le même partout : des lignes `clé = valeur`,
//! des sections `[nom]`, et des commentaires qui commencent par `#`.

/// Une ligne `clé = valeur`, avec la section où elle se trouve.
#[derive(Debug, PartialEq)]
pub struct Ligne {
    pub section: String,
    pub cle: String,
    pub valeur: String,
    /// Le numéro de la ligne dans le fichier, pour les messages d'erreur.
    pub numero: usize,
}

/// Lit un fichier de réglages. Les commentaires du début (avant la première ligne utile)
/// forment la description, renvoyée à part.
pub fn lire(texte: &str) -> Result<(String, Vec<Ligne>), String> {
    let mut section = String::new();
    let mut lignes = Vec::new();
    let mut description = Vec::new();
    for (i, brute) in texte.lines().enumerate() {
        let numero = i + 1;
        let ligne = brute.trim();
        if let Some(commentaire) = ligne.strip_prefix('#') {
            if lignes.is_empty() && section.is_empty() {
                description.push(commentaire.trim().to_string());
            }
            continue;
        }
        if ligne.is_empty() {
            // Une ligne vide termine aussi la description.
            if lignes.is_empty() && section.is_empty() {
                description.push(String::new());
            }
            continue;
        }
        if let Some(nom) = ligne.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            section = nom.trim().to_string();
            continue;
        }
        let Some((cle, valeur)) = ligne.split_once('=') else {
            return Err(format!(
                "ligne {numero} : « clé = valeur » attendu, lu « {ligne} »"
            ));
        };
        let (cle, valeur) = (cle.trim(), valeur.trim());
        if cle.is_empty() || valeur.is_empty() {
            return Err(format!("ligne {numero} : clé ou valeur vide"));
        }
        lignes.push(Ligne {
            section: section.clone(),
            cle: cle.to_string(),
            valeur: valeur.to_string(),
            numero,
        });
    }
    // La description est le premier paragraphe de commentaires.
    let debut = description
        .iter()
        .position(|l| !l.is_empty())
        .unwrap_or(description.len());
    let fin = description[debut..]
        .iter()
        .position(String::is_empty)
        .map_or(description.len(), |n| debut + n);
    Ok((description[debut..fin].join(" "), lignes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sections_commentaires_et_description() {
        let texte = "\n# Un profil.\n# Sur deux lignes.\n\n# Détail.\n\n[a]\nx = 1\n# rien\n[b]\n/y/z = 80% w\n";
        let (description, lignes) = lire(texte).unwrap();
        assert_eq!(description, "Un profil. Sur deux lignes.");
        assert_eq!(lignes.len(), 2);
        assert_eq!(lignes[0].section, "a");
        assert_eq!(lignes[1].cle, "/y/z");
        assert_eq!(lignes[1].valeur, "80% w");
        assert_eq!(lignes[1].numero, 11);
    }

    #[test]
    fn ligne_invalide() {
        assert!(lire("x\n").unwrap_err().contains("ligne 1"));
        assert!(lire("x =\n").is_err());
    }
}
