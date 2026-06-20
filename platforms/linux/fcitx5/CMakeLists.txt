cmake_minimum_required(VERSION 3.16)
project(funput-fcitx5 VERSION 1.2026.1 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Release)
endif()

find_package(Fcitx5Core REQUIRED)
find_package(Fcitx5Utils REQUIRED)
find_package(nlohmann_json 3 REQUIRED)

include(GNUInstallDirs)

# --- Rust core (funput-ffi) -> libfunput_ffi.so ----------------------------
# The pure typing logic lives in Rust; this addon is a thin C++ shell that calls
# the cbindgen-generated C ABI. We build the cdylib via cargo and link it in.
get_filename_component(FUNPUT_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../../.." ABSOLUTE)
set(FUNPUT_FFI_INCLUDE "${FUNPUT_ROOT}/crates/funput-ffi/include")
set(FUNPUT_FFI_SO "${FUNPUT_ROOT}/target/release/libfunput_ffi.so")

find_program(CARGO_EXECUTABLE cargo REQUIRED)

add_custom_command(
    OUTPUT "${FUNPUT_FFI_SO}"
    COMMAND ${CARGO_EXECUTABLE} build --release -p funput-ffi
    WORKING_DIRECTORY "${FUNPUT_ROOT}"
    COMMENT "Building funput-ffi (Rust cdylib)"
    VERBATIM
)
add_custom_target(funput_ffi DEPENDS "${FUNPUT_FFI_SO}")

# --- Fcitx5 addon -> libfunput.so ------------------------------------------
add_library(funput MODULE
    src/funput_engine.cpp
    src/settings.cpp
)
add_dependencies(funput funput_ffi)

target_include_directories(funput PRIVATE "${FUNPUT_FFI_INCLUDE}")
target_link_libraries(funput PRIVATE
    Fcitx5::Core
    Fcitx5::Utils
    nlohmann_json::nlohmann_json
    "${FUNPUT_FFI_SO}"
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

# Tray / input-method icon (referenced as Icon=funput).
if(EXISTS "${FUNPUT_ROOT}/assets/logo.png")
    install(FILES "${FUNPUT_ROOT}/assets/logo.png"
            RENAME funput.png
            DESTINATION "${CMAKE_INSTALL_DATADIR}/icons/hicolor/128x128/apps")
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
set(CPACK_PACKAGE_NAME "funput")
set(CPACK_PACKAGE_VERSION "${FUNPUT_VERSION}")
set(CPACK_PACKAGE_DESCRIPTION_SUMMARY "Vietnamese input method (Telex & VNI) — Fcitx5 addon")
set(CPACK_PACKAGE_CONTACT "PulseFu <hello@pulsefu.com>")
set(CPACK_DEBIAN_PACKAGE_MAINTAINER "PulseFu <hello@pulsefu.com>")
set(CPACK_DEBIAN_PACKAGE_SECTION "utils")
set(CPACK_DEBIAN_PACKAGE_HOMEPAGE "https://github.com/pcodedynamics/Funput")
# fcitx5 must be present; the rest are picked up automatically.
set(CPACK_DEBIAN_PACKAGE_DEPENDS "fcitx5")
set(CPACK_DEBIAN_PACKAGE_SHLIBDEPS ON)
set(CPACK_PACKAGE_FILE_NAME "funput_${FUNPUT_VERSION}_amd64")
include(CPack)
