//! Les réglages livrés avec le système (`/system_ext/etc/energie`, en lecture seule) sont
//! copiés dans le dossier d'energie (`/data/misc/energie`) où on peut les modifier.
//! Seuls les fichiers absents sont copiés : une modification n'est jamais écrasée, et un
//! nouveau profil livré par une mise à jour apparaît tout seul.

use std::fs;
use std::path::Path;

/// Copie les fichiers manquants ; renvoie combien ont été copiés.
pub fn installer(modeles: &Path, dossier: &Path) -> Result<usize, String> {
    if !modeles.is_dir() {
        return Ok(0);
    }
    let mut copies = 0;
    for sous_dossier in ["", "profils"] {
        let (source, cible) = (modeles.join(sous_dossier), dossier.join(sous_dossier));
        let Ok(entrees) = fs::read_dir(&source) else {
            continue;
        };
        fs::create_dir_all(&cible).map_err(|e| format!("{} : {e}", cible.display()))?;
        for entree in entrees.flatten() {
            let nom = entree.file_name();
            let est_conf = Path::new(&nom).extension().is_some_and(|e| e == "conf");
            if !est_conf || !entree.path().is_file() || cible.join(&nom).exists() {
                continue;
            }
            fs::copy(entree.path(), cible.join(&nom))
                .map_err(|e| format!("copie de {} : {e}", entree.path().display()))?;
            copies += 1;
        }
    }
    Ok(copies)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systeme::tests::FauxArbre;

    #[test]
    fn copie_les_manquants_sans_ecraser() {
        let arbre = FauxArbre::nouveau("modeles");
        arbre
            .fichier("/etc/energie.conf", "[regles]\ndefaut = eco\n")
            .fichier("/etc/profils/eco.conf", "# livré\n")
            .fichier("/etc/profils/nuit.conf", "# livré\n")
            .fichier("/etc/profils/LISEZMOI", "pas un profil")
            .fichier("/data/profils/eco.conf", "# modifié\n");
        let (modeles, dossier) = (arbre.racine.join("etc"), arbre.racine.join("data"));
        assert_eq!(installer(&modeles, &dossier).unwrap(), 2);
        let lire = |c: &str| fs::read_to_string(dossier.join(c)).unwrap();
        assert_eq!(lire("profils/eco.conf"), "# modifié\n");
        assert_eq!(lire("profils/nuit.conf"), "# livré\n");
        assert!(lire("energie.conf").contains("defaut = eco"));
        assert!(!dossier.join("profils/LISEZMOI").exists());
        assert_eq!(installer(&modeles, &dossier).unwrap(), 0);
        assert_eq!(
            installer(&arbre.racine.join("absent"), &dossier).unwrap(),
            0
        );
    }
}
