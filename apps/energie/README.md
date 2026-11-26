# Énergie : choisir le profil depuis le téléphone

Une application Android en Kotlin qui pilote le démon [energie](../../services/energie) :

- **une tuile « Énergie »** dans les réglages rapides : un appui passe au choix suivant
  (automatique, puis chaque profil) ; elle affiche le profil en cours ;
- **un écran** (tiroir d'applications, ou appui long sur la tuile) : la liste des profils
  avec leur description, et « Automatique » pour revenir aux règles de `energie.conf`.

**État : compilée et testée sur un ordinateur ; pas encore compilée dans AOSP ni essayée
sur le téléphone.**

## Comment elle parle au démon

Exactement comme `energie forcer <profil>` : elle écrit le nom du profil dans
`/data/misc/energie/force` (et l'efface pour revenir à « Automatique »). Le démon relit ce
fichier toutes les 2 secondes. Elle lit `profils/` pour la liste et `actif` pour le profil
appliqué. Pas de service Binder à maintenir : le dossier est l'interface.

Pour y avoir accès, l'application tourne avec l'uid `system` (`android:sharedUserId`),
signée avec la clé de la plateforme ; le dossier appartient à `root:system` (0770), et la
politique SELinux du démon ([energie.te](../../services/energie/sepolicy/energie.te))
autorise le domaine `system_app` à y écrire. Elle ne fonctionne donc qu'intégrée au système
customFirmware, pas installée à part.

## Les fichiers

| Fichier | Rôle |
|---|---|
| [`Energie.kt`](src/customfirmware/energie/Energie.kt) | lire les profils, forcer un profil, le choix suivant : sans Android, testé sur un ordinateur |
| [`EnergieTile.kt`](src/customfirmware/energie/EnergieTile.kt) | la tuile des réglages rapides |
| [`ReglagesActivity.kt`](src/customfirmware/energie/ReglagesActivity.kt) | l'écran de choix, construit en code, sans bibliothèque |
| [`EnergieTest.kt`](tests/customfirmware/energie/EnergieTest.kt) | les tests JUnit de `Energie.kt` |
| [`Android.bp`](Android.bp), [`AndroidManifest.xml`](AndroidManifest.xml) | la compilation par AOSP |

## Compiler

Dans AOSP : `m CustomFirmwareEnergie` (le produit `customfirmware_arm64` l'inclut déjà).

Sur un ordinateur, pour vérifier le code sans AOSP ni SDK Android, avec le compilateur
Kotlin, JUnit, et `android-all` (les classes d'Android construites depuis AOSP par
Robolectric, sur Maven Central) :

```sh
# les tests de la logique
kotlinc apps/energie/src/customfirmware/energie/Energie.kt apps/energie/tests \
  -cp junit-4.13.2.jar:hamcrest-core-1.3.jar -d out-test
java -cp out-test:junit-4.13.2.jar:hamcrest-core-1.3.jar:$KOTLIN_HOME/lib/kotlin-stdlib.jar \
  org.junit.runner.JUnitCore customfirmware.energie.EnergieTest

# toute l'application, contre les classes d'Android 16 (R.kt : un faux R, généré
# depuis res/values/strings.xml, car c'est aapt qui produit le vrai)
kotlinc apps/energie/src R.kt -cp android-all-16-robolectric-13921718.jar -d out-app -Werror
```

## Sur le téléphone

1. Ouvrir les réglages rapides, appuyer sur le crayon (modifier), glisser la tuile
   **Énergie** parmi les tuiles actives.
2. Un appui : automatique → origine → economie → … → automatique.
3. Un appui long : l'écran de choix.

Ajouter ou modifier un profil dans `/data/misc/energie/profils/` (voir le README du démon)
le fait apparaître dans la tuile et l'écran.
