package customfirmware.energie

import android.service.quicksettings.Tile
import android.service.quicksettings.TileService
import android.util.Log

/**
 * La tuile « Énergie » des réglages rapides. Un appui passe au choix suivant
 * (automatique, puis chaque profil) ; un appui long ouvre [ReglagesActivity].
 */
class EnergieTile : TileService() {

    private val energie = Energie()

    override fun onStartListening() = afficher()

    override fun onClick() {
        runCatching { energie.forcer(energie.suivant()) }
            .onFailure { Log.w(TAG, "choix du profil impossible", it) }
        afficher()
    }

    private fun afficher() {
        val tuile = qsTile ?: return
        val force = energie.force()
        tuile.label = getString(R.string.tuile)
        when {
            energie.profils().isEmpty() -> {
                tuile.state = Tile.STATE_UNAVAILABLE
                tuile.subtitle = getString(R.string.indisponible)
            }
            force != null -> {
                tuile.state = Tile.STATE_ACTIVE
                tuile.subtitle = force
            }
            else -> {
                tuile.state = Tile.STATE_INACTIVE
                tuile.subtitle = getString(R.string.auto_tuile, energie.actif() ?: "…")
            }
        }
        tuile.updateTile()
    }

    companion object {
        private const val TAG = "energie"
    }
}
