# hello : le premier service en Rust

Un service Binder minimal : une interface AIDL (`IHello`), son implémentation en Rust, et
son enregistrement sous le nom `customfirmware.hello`. Il suit l'exemple officiel d'AOSP
([Android Rust patterns](https://source.android.com/docs/setup/build/rust/building-rust-modules/android-rust-patterns)).

**État : écrit, pas encore compilé.** Il se compile dans un arbre AOSP, pas avec Cargo.

```sh
m customfirmware-hello
```

## L'essayer sur le téléphone

Le service n'a pas encore de politique SELinux : `init` ne peut donc pas le lancer tout seul.
Pour un premier essai, sur un système `userdebug`, on le lance à la main en mode permissif :

```sh
adb root
adb shell setenforce 0
adb shell /system_ext/bin/customfirmware-hello &
adb shell service check customfirmware.hello        # « found »
adb shell service call customfirmware.hello 1 s16 Max
```

## Ce qui manque pour en faire un vrai service

- une politique SELinux (domaine, contexte du fichier et du service) ;
- un fichier `.rc` pour qu'`init` le lance au démarrage ;
- pour une couche matérielle : une interface stable, la partition vendor et une déclaration
  dans le manifeste VINTF.
