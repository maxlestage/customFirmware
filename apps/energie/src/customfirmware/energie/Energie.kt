package customfirmware.energie

import java.io.File

/**
 * Le dossier du démon energie (services/energie) : ce que l'application y lit et y écrit.
 *
 * Le démon relit ses fichiers toutes les 2 secondes ; pour lui imposer un profil, il suffit
 * d'écrire son nom dans `force`, comme le fait `energie forcer <profil>`. Rien d'Android
 * ici : cette classe se teste sur un ordinateur.
 */
class Energie(private val dossier: File = File(DOSSIER)) {

    data class Profil(val nom: String, val description: String)

    /** Le profil que le démon a appliqué en dernier, ou `null`. */
    fun actif(): String? = lire("actif")?.takeIf(::nomValide)

    /** Le profil imposé à la main, ou `null` quand le démon suit ses règles. */
    fun force(): String? = lire("force")?.takeIf(::nomValide)

    /** `origine`, puis les profils de `profils/` dans l'ordre alphabétique. Vide si le
     *  dossier est illisible (energie absent, ou pas les droits). */
    fun profils(): List<Profil> {
        val fichiers = File(dossier, "profils")
            .listFiles { f -> f.isFile && f.name.endsWith(".conf") }
            ?: return emptyList()
        val autres = fichiers
            .map { it.name.removeSuffix(".conf") to it }
            .filter { (nom, _) -> nomValide(nom) && nom != ORIGINE }
            .sortedBy { it.first }
            .map { (nom, fichier) ->
                Profil(nom, runCatching { description(fichier.readText()) }.getOrDefault(""))
            }
        return listOf(Profil(ORIGINE, DESCRIPTION_ORIGINE)) + autres
    }

    /** Impose un profil au démon ; `null` lui rend la main. */
    fun forcer(nom: String?) {
        val force = File(dossier, "force")
        if (nom == null) {
            if (force.exists() && !force.delete()) error("impossible d'effacer $force")
            return
        }
        require(nomValide(nom)) { "nom de profil invalide : $nom" }
        // Écrit à côté puis renomme : le démon ne lit jamais un fichier à moitié écrit.
        val provisoire = File(dossier, "force.tmp")
        provisoire.writeText(nom)
        // Le démon tourne en root sans aucune capacité : il lit grâce aux droits « autres ».
        provisoire.setReadable(true, false)
        if (!provisoire.renameTo(force)) error("impossible d'écrire $force")
    }

    /** Le choix suivant pour la tuile : automatique, chaque profil, puis automatique. */
    fun suivant(): String? {
        val noms = profils().map { it.nom }
        val i = force()?.let(noms::indexOf) ?: return noms.firstOrNull()
        return noms.getOrNull(i + 1)
    }

    private fun lire(nom: String): String? =
        runCatching { File(dossier, nom).readText().trim() }.getOrNull()?.ifEmpty { null }

    companion object {
        const val DOSSIER = "/data/misc/energie"
        const val ORIGINE = "origine"
        const val DESCRIPTION_ORIGINE = "Les réglages de Nothing, sans changement."

        /** Les mêmes règles que le démon : minuscules, chiffres, `-` et `_`. */
        fun nomValide(nom: String): Boolean =
            nom.isNotEmpty() && nom.all { it in 'a'..'z' || it in '0'..'9' || it == '_' || it == '-' }

        /** Le premier paragraphe de commentaires en tête du fichier, comme `energie liste`. */
        fun description(texte: String): String {
            val lignes = mutableListOf<String>()
            for (brute in texte.lines()) {
                val ligne = brute.trim()
                if (!ligne.startsWith("#")) {
                    if (ligne.isEmpty() && lignes.isEmpty()) continue
                    break
                }
                val commentaire = ligne.removePrefix("#").trim()
                if (commentaire.isEmpty()) {
                    if (lignes.isEmpty()) continue else break
                }
                lignes += commentaire
            }
            return lignes.joinToString(" ")
        }
    }
}
