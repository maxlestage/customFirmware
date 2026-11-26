# Squelette, pas encore compilable : voir README.md.

$(call inherit-product, $(SRC_TARGET_DIR)/product/core_64_bit_only.mk)
$(call inherit-product, $(SRC_TARGET_DIR)/product/aosp_base_telephony.mk)
$(call inherit-product, vendor/customfirmware/device/SuperContra/device.mk)
$(call inherit-product, vendor/customfirmware/products/customfirmware.mk)

PRODUCT_NAME := aosp_SuperContra
PRODUCT_DEVICE := SuperContra
PRODUCT_BRAND := Nothing
PRODUCT_MANUFACTURER := Nothing
PRODUCT_MODEL := A009P
