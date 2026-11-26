# Ce que customFirmware ajoute à AOSP, quel que soit le produit.

# Le premier service écrit en Rust (services/hello), les réglages d'énergie du
# noyau (services/energie) et l'application qui choisit leur profil (apps/energie).
PRODUCT_PACKAGES += \
    customfirmware-hello \
    customfirmware-energie \
    CustomFirmwareEnergie

# La politique SELinux d'energie (system_ext, privée).
SYSTEM_EXT_PRIVATE_SEPOLICY_DIRS += vendor/customfirmware/services/energie/sepolicy
