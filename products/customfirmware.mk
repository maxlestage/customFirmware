# Ce que customFirmware ajoute à AOSP, quel que soit le produit.

# Le premier service écrit en Rust (services/hello), et les réglages d'énergie du
# noyau (services/energie).
PRODUCT_PACKAGES += \
    customfirmware-hello \
    customfirmware-energie

# La politique SELinux d'energie (system_ext, privée).
SYSTEM_EXT_PRIVATE_SEPOLICY_DIRS += vendor/customfirmware/services/energie/sepolicy
