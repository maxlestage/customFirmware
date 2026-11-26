# customfirmware_arm64 : une image système générique (GSI) d'AOSP pour ARM 64 bits,
# plus les services de customFirmware. Elle s'installe dans la partition system d'un
# téléphone compatible Treble, à côté du noyau et des pilotes d'origine.
#
#   lunch customfirmware_arm64-aosp_current-userdebug && m
#
# aosp_arm64.mk ne règle la diffusion en GSI que pour le produit « aosp_arm64 » :
# gsi_release.mk est donc repris ici.

$(call inherit-product, $(SRC_TARGET_DIR)/product/aosp_arm64.mk)
$(call inherit-product, $(SRC_TARGET_DIR)/product/gsi_release.mk)
$(call inherit-product, vendor/customfirmware/products/customfirmware.mk)

PRODUCT_NAME := customfirmware_arm64
PRODUCT_DEVICE := generic_arm64
PRODUCT_BRAND := customFirmware
PRODUCT_MODEL := customFirmware sur ARM64
