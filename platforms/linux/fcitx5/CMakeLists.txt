cmake_minimum_required(VERSION 3.16)
project(funput-fcitx5 VERSION 1.2026.1 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Release)
endif()

# Install into /usr, not the /usr/local default. Fcitx5 only scans the system
# addon dirs (/usr/lib/<triplet>/fcitx5, /usr/share/fcitx5) — it ignores
# /usr/local. The /usr prefix also makes GNUInstallDirs use the Debian multiarch
# libdir, so FCITX_INSTALL_ADDONDIR resolves correctly. Must run before
# find_package(Fcitx5Utils), which computes those install dirs.
if(CMAKE_INSTALL_PREFIX_INITIALIZED_TO_DEFAULT)
    set(CMAKE_INSTALL_PREFIX "/usr" CACHE PATH "Install prefix" FORCE)
endif()

find_package(Fcitx5Core REQUIRED)
find_package(Fcitx5Utils REQUIRED)

include(GNUInstallDirs)

# --- Shared Linux code (settings reader + FFI wrapper + Rust cdylib build) --
# Lives in platforms/linux/common, shared with the IBus shell. Owns building
# libfunput_ffi.so and exposes (PUBLIC) the common headers, funput.h, and the .so
# to link — plus the FUNPUT_ROOT / FUNPUT_FFI_SO cache vars used below.
add_subdirectory("${CMAKE_CURRENT_SOURCE_DIR}/../common"
                 "${CMAKE_BINARY_DIR}/common")

# --- Fcitx5 addon -> libfunput.so ------------------------------------------
add_library(funput MODULE
    src/funput_engine.cpp
)
add_dependencies(funput funput_ffi)

# funput_linux_common carries the FFI include dir + the .so transitively.
target_link_libraries(funput PRIVATE
    Fcitx5::Core
    Fcitx5::Utils
    funput_linux_common
)

# libfunput_ffi.so is installed next to libfunput.so; resolve it via $ORIGIN so
# no system-wide ldconfig entry is needed.
set_target_properties(funput PROPERTIES
    INSTALL_RPATH "$ORIGIN"
    BUILD_WITH_INSTALL_RPATH ON
)

# --- Install ---------------------------------------------------------------
# Addon .so + the Rust core, side by side in the fcitx5 addon dir.
install(TARGETS funput DESTINATION "${FCITX_INSTALL_ADDONDIR}")
install(FILES "${FUNPUT_FFI_SO}" DESTINATION "${FCITX_INSTALL_ADDONDIR}")

# Addon metadata + input-method registration.
configure_file(data/funput-addon.conf.in funput-addon.conf @ONLY)
configure_file(data/funput.conf.in funput.conf @ONLY)
install(FILES "${CMAKE_CURRENT_BINARY_DIR}/funput-addon.conf"
        RENAME funput.conf
        DESTINATION "${FCITX_INSTALL_PKGDATADIR}/addon")
install(FILES "${CMAKE_CURRENT_BINARY_DIR}/funput.conf"
        DESTINATION "${FCITX_INSTALL_PKGDATADIR}/inputmethod")

# Tray / input-method icon (referenced as Icon=funput). Ship the scalable SVG so
# it renders crisp at any size; the 512px PNG goes in its real size bucket.
if(EXISTS "${FUNPUT_ROOT}/assets/svg/funput_logo.svg")
    install(FILES "${FUNPUT_ROOT}/assets/svg/funput_logo.svg"
            RENAME funput.svg
            DESTINATION "${CMAKE_INSTALL_DATADIR}/icons/hicolor/scalable/apps")
endif()
if(EXISTS "${FUNPUT_ROOT}/assets/logo.png")
    install(FILES "${FUNPUT_ROOT}/assets/logo.png"
            RENAME funput.png
            DESTINATION "${CMAKE_INSTALL_DATADIR}/icons/hicolor/512x512/apps")
endif()

# --- Tauri Settings app (built separately by cargo) ------------------------
# The GUI binary + its launcher. Built before this CMake runs (see build.sh);
# if it isn't present we still package the addon alone (typing works without it).
set(FUNPUT_SETTINGS_BIN
    "${CMAKE_CURRENT_SOURCE_DIR}/../src-tauri/target/release/funput-settings"
    CACHE FILEPATH "Path to the built Tauri Settings binary")

if(EXISTS "${FUNPUT_SETTINGS_BIN}")
    install(PROGRAMS "${FUNPUT_SETTINGS_BIN}" DESTINATION "${CMAKE_INSTALL_BINDIR}")
    install(FILES "${CMAKE_CURRENT_SOURCE_DIR}/../packaging/funput-settings.desktop"
            DESTINATION "${CMAKE_INSTALL_DATADIR}/applications")
else()
    message(WARNING "funput-settings binary not found at ${FUNPUT_SETTINGS_BIN}; "
                    "packaging the Fcitx5 addon only (no Settings GUI).")
endif()

# --- .deb packaging (CPack) ------------------------------------------------
# CI overrides this from the git tag (-DFUNPUT_VERSION=...); otherwise project().
set(FUNPUT_VERSION "${PROJECT_VERSION}" CACHE STRING "Package version")
set(CPACK_GENERATOR "DEB")
# Lay the relative installs (bin, share/…) under /usr to match the absolute
# Fcitx5 addon dirs — everything in the .deb lives under /usr.
set(CPACK_PACKAGING_INSTALL_PREFIX "/usr")
set(CPACK_PACKAGE_NAME "funput")
set(CPACK_PACKAGE_VERSION "${FUNPUT_VERSION}")
set(CPACK_PACKAGE_DESCRIPTION_SUMMARY "Vietnamese input method (Telex & VNI) — Fcitx5 addon")
set(CPACK_PACKAGE_CONTACT "Funput <hello@funput.app>")
set(CPACK_DEBIAN_PACKAGE_MAINTAINER "Funput <hello@funput.app>")
set(CPACK_DEBIAN_PACKAGE_SECTION "utils")
set(CPACK_DEBIAN_PACKAGE_HOMEPAGE "https://github.com/Funput/Funput")
# fcitx5 must be present; the rest are picked up automatically.
set(CPACK_DEBIAN_PACKAGE_DEPENDS "fcitx5")
set(CPACK_DEBIAN_PACKAGE_SHLIBDEPS ON)
# Architecture from the build host (amd64 / arm64 / …). CPack sets the control
# field from this; mirror it in the filename so an arm64 build isn't mislabelled
# "amd64". Apple Silicon VMs (Apple Virtualization) are arm64, so a host-arch deb
# matters there.
find_program(DPKG_CMD dpkg)
if(DPKG_CMD)
    execute_process(COMMAND ${DPKG_CMD} --print-architecture
        OUTPUT_VARIABLE FUNPUT_DEB_ARCH OUTPUT_STRIP_TRAILING_WHITESPACE)
else()
    set(FUNPUT_DEB_ARCH "amd64")
endif()
set(CPACK_DEBIAN_PACKAGE_ARCHITECTURE "${FUNPUT_DEB_ARCH}")
set(CPACK_PACKAGE_FILE_NAME "funput_${FUNPUT_VERSION}_${FUNPUT_DEB_ARCH}")
include(CPack)
