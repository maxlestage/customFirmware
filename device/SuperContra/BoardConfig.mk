# Squelette, pas encore compilable : voir README.md.

# Les faits relevés sur le téléphone par `cf props … --mk facts.mk` (variables CF_…).
-include vendor/customfirmware/device/SuperContra/facts.mk

# Snapdragon 6 Gen 4 (SM6650) : cœurs Cortex-A720 et A520, 64 bits uniquement.
TARGET_ARCH := arm64
TARGET_ARCH_VARIANT := armv8-2a
TARGET_CPU_ABI := arm64-v8a
TARGET_CPU_VARIANT := generic

TARGET_BOOTLOADER_BOARD_NAME := SuperContra
TARGET_BOARD_PLATFORM := $(CF_BOARD_PLATFORM)

# `cf blobs` installe les fichiers propriétaires par PRODUCT_COPY_FILES, bibliothèques
# comprises. AOSP le refuse par défaut ; à retirer quand ils seront des modules prebuilt.
BUILD_BROKEN_ELF_PREBUILT_PRODUCT_COPY_FILES := true

# À faire : partitions (taille de super, groupes), noyau, image de démarrage, SELinux.
