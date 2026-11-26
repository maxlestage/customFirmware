# Ce que customFirmware ajoute à AOSP, quel que soit le produit.

# Le premier service écrit en Rust (services/hello), et les réglages d'énergie du
# noyau (services/energie).
PRODUCT_PACKAGES += \
    customfirmware-hello \
    customfirmware-energie
