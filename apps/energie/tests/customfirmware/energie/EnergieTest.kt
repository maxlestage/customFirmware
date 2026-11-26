package customfirmware.energie

import java.io.File
import java.nio.file.Files
import java.nio.file.attribute.PosixFilePermission
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/** La logique de l'application, sur un faux /data/misc/energie. Tourne sur un ordinateur. */
class EnergieTest {

    private val dossier: File = Files.createTempDirectory("energie").toFile()
    private val energie = Energie(dossier)

    private fun fichier(chemin: String, texte: String) =
        File(dossier, chemin).apply { parentFile.mkdirs(); writeText(texte) }

    @After
    fun effacer() {
        dossier.deleteRecursively()
    }

    @Test
    fun litLesProfilsDansLOrdre() {
        fichier("profils/performance.conf", "# Performance : jeux.\n/proc/sys/walt/sched_boost = 1\n")
        fichier("profils/economie.conf", "\n# Économie : écran allumé.\n# Sur deux lignes.\n\n# Détail.\n")
        fichier("profils/notes.txt", "pas un profil")
        fichier("profils/Majuscule.conf", "# nom invalide")
        val profils = energie.profils()
        assertEquals(listOf("origine", "economie", "performance"), profils.map { it.nom })
        assertEquals("Économie : écran allumé. Sur deux lignes.", profils[1].description)
        assertEquals("Performance : jeux.", profils[2].description)
    }

    @Test
    fun sansDossierRienNEstDisponible() {
        assertTrue(Energie(File(dossier, "absent")).profils().isEmpty())
        assertNull(Energie(File(dossier, "absent")).actif())
    }

    @Test
    fun forcerPuisRendreLaMain() {
        fichier("profils/economie.conf", "")
        energie.forcer("economie")
        val force = File(dossier, "force")
        assertEquals("economie", force.readText())
        assertEquals("economie", energie.force())
        // Lisible par le démon, qui n'a que les droits « autres ».
        assertTrue(PosixFilePermission.OTHERS_READ in Files.getPosixFilePermissions(force.toPath()))
        assertFalse(File(dossier, "force.tmp").exists())
        energie.forcer(null)
        assertFalse(force.exists())
        assertNull(energie.force())
        energie.forcer(null) // déjà automatique : rien à faire
    }

    @Test(expected = IllegalArgumentException::class)
    fun refuseUnNomInvalide() {
        energie.forcer("../etc")
    }

    @Test
    fun laTuileFaitLeTourDesChoix() {
        fichier("profils/economie.conf", "")
        fichier("profils/performance.conf", "")
        val vus = mutableListOf<String?>()
        repeat(4) {
            val suivant = energie.suivant()
            energie.forcer(suivant)
            vus += suivant
        }
        assertEquals(listOf("origine", "economie", "performance", null), vus)
        // Un profil forcé qui a disparu : on repart du début.
        fichier("force", "efface")
        assertEquals("origine", energie.suivant())
    }

    @Test
    fun litLeProfilActif() {
        fichier("actif", "economie\n")
        assertEquals("economie", energie.actif())
        fichier("force", "pas valide !")
        assertNull(energie.force())
    }
}
