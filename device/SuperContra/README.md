# device/SuperContra : la configuration du Phone (4b)

**État : squelette, pas encore compilable.** C'est l'étape 2 du projet : un système complet
pour le Nothing Phone (4b), avec sa propre partition vendor. L'étape 1, l'image système
générique de [`products/`](../../products), n'en a pas besoin.

| Fichier | Rôle | État |
|---|---|---|
| `aosp_SuperContra.mk` | le produit | écrit, jamais compilé |
| `device.mk` | ce qui est installé | à compléter |
| `BoardConfig.mk` | processeur, partitions, noyau | processeur seulement |
| `proprietary-files.txt` | les fichiers à reprendre du système d'origine | vide |
| `facts.mk` | les faits du téléphone, écrits par `cf props` | à générer (non suivi par git) |

```sh
# Les faits du téléphone, depuis la sortie de `adb shell getprop` :
cf props ~/supercontra/getprop.txt --mk device/SuperContra/facts.mk

# Les fichiers propriétaires, depuis les partitions d'origine montées dans /mnt/stock :
cf blobs --list device/SuperContra/proprietary-files.txt \
         --from /mnt/stock --out ~/aosp/vendor/nothing/SuperContra
```

Reste à écrire : la table des partitions, le noyau et l'image de démarrage, la politique
SELinux, le manifeste VINTF et la liste des fichiers propriétaires.
