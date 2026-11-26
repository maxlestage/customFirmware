package customfirmware.energie

import android.app.Activity
import android.os.Bundle
import android.util.TypedValue
import android.view.View
import android.view.WindowInsets
import android.widget.LinearLayout
import android.widget.RadioButton
import android.widget.RadioGroup
import android.widget.ScrollView
import android.widget.TextView
import android.widget.Toast

/**
 * Le choix du profil : automatique (les règles de `energie.conf`) ou un profil imposé.
 * S'ouvre depuis le tiroir d'applications, ou par un appui long sur la tuile.
 */
class ReglagesActivity : Activity() {

    private val energie = Energie()
    private lateinit var etat: TextView
    private lateinit var choix: RadioGroup

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val marge = dp(20)
        val colonne = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(marge, marge, marge, marge)
        }
        colonne.addView(TextView(this).apply {
            text = getString(R.string.app_name)
            setTextSize(TypedValue.COMPLEX_UNIT_SP, 28f)
        })
        etat = TextView(this).apply { setPadding(0, dp(8), 0, dp(16)) }
        colonne.addView(etat)
        choix = RadioGroup(this)
        colonne.addView(choix)
        colonne.addView(TextView(this).apply {
            text = getString(R.string.aide)
            alpha = 0.7f
            setPadding(0, dp(24), 0, 0)
        })
        val defilement = ScrollView(this).apply { addView(colonne) }
        // Sous Android 15 et plus, l'écran va sous les barres système : on s'en écarte.
        defilement.setOnApplyWindowInsetsListener { vue, insets ->
            val barres = insets.getInsets(WindowInsets.Type.systemBars())
            vue.setPadding(barres.left, barres.top, barres.right, barres.bottom)
            insets
        }
        setContentView(defilement)
    }

    override fun onResume() {
        super.onResume()
        remplir()
    }

    private fun remplir() {
        val profils = energie.profils()
        choix.setOnCheckedChangeListener(null)
        choix.removeAllViews()
        if (profils.isEmpty()) {
            etat.text = getString(R.string.indisponible_long)
            return
        }
        val force = energie.force()
        etat.text = getString(R.string.applique, energie.actif() ?: "…")
        val noms = listOf<String?>(null) + profils.map { it.nom }
        val textes = listOf(getString(R.string.auto_choix)) +
            profils.map { if (it.description.isEmpty()) it.nom else "${it.nom}\n${it.description}" }
        noms.zip(textes).forEach { (nom, texte) ->
            choix.addView(RadioButton(this).apply {
                id = View.generateViewId()
                tag = nom
                text = texte
                setPadding(dp(8), dp(12), 0, dp(12))
            })
        }
        // indexOf(null) vaut 0 : « automatique » quand rien n'est forcé.
        val coche = noms.indexOf(force).coerceAtLeast(0)
        choix.check(choix.getChildAt(coche).id)
        choix.setOnCheckedChangeListener { groupe, id ->
            val nom = groupe.findViewById<RadioButton>(id)?.tag as String?
            runCatching { energie.forcer(nom) }
                .onSuccess {
                    etat.text = getString(R.string.dans_deux_secondes, nom ?: getString(R.string.auto_court))
                }
                .onFailure {
                    Toast.makeText(this, getString(R.string.erreur, it.message), Toast.LENGTH_LONG).show()
                    remplir()
                }
        }
    }

    private fun dp(valeur: Int): Int = (valeur * resources.displayMetrics.density).toInt()
}
