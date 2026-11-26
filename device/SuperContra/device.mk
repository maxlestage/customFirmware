# Squelette, pas encore compilable : voir README.md.

# Les faits relevés sur le téléphone par `cf props … --mk facts.mk` (variables CF_…).
-include vendor/customfirmware/device/SuperContra/facts.mk

PRODUCT_SHIPPING_API_LEVEL := $(CF_SHIPPING_API_LEVEL)

# Les fichiers propriétaires copiés par `cf blobs` (hors de ce dépôt).
$(call inherit-product-if-exists, vendor/nothing/SuperContra/blobs.mk)
