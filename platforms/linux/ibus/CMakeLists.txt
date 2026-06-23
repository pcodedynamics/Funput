cmake_minimum_required(VERSION 3.16)
project(funput-ibus VERSION 1.2026.1 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Release)
endif()

# Install into /usr, not /usr/local: ibus-daemon scans the system component dir
# (/usr/share/ibus/component) and the /usr prefix makes GNUInstallDirs resolve the
# Debian multiarch dirs. Must run before include(GNUInstallDirs).
if(CMAKE_INSTALL_PREFIX_INITIALIZED_TO_DEFAULT)
    set(CMAKE_INSTALL_PREFIX "/usr" CACHE PATH "Install prefix" FORCE)
endif()

find_package(PkgConfig REQUIRED)
pkg_check_modules(IBUS REQUIRED IMPORTED_TARGET ibus-1.0)

include(GNUInstallDirs)

# --- Shared Linux code (settings reader + FFI wrapper + Rust cdylib build) --
# Lives in platforms/linux/common, shared with the Fcitx5 shell. Owns building
# libfunput_ffi.so and exposes (PUBLIC) the common headers, funput.h, and the .so
# to link — plus the FUNPUT_ROOT / FUNPUT_FFI_SO cache vars used below.
add_subdirectory("${CMAKE_CURRENT_SOURCE_DIR}/../common"
                 "${CMAKE_BINARY_DIR}/common")

# Package version: project() by default; CI overrides with -DFUNPUT_VERSION=<tag>.
# Flows into both the component manifest and the binary's --xml output.
set(FUNPUT_VERSION "${PROJECT_VERSION}" CACHE STRING "Package version")

# --- Engine install path (the component <exec> must match this) ------------
# Debian installs IBus engines into /usr/lib/ibus; query ibus for the exact dir.
pkg_get_variable(IBUS_LIBEXECDIR ibus-1.0 libexecdir)
if(NOT IBUS_LIBEXECDIR)
    set(IBUS_LIBEXECDIR "${CMAKE_INSTALL_FULL_LIBEXECDIR}/ibus")
endif()
set(IBUS_ENGINE_PATH "${IBUS_LIBEXECDIR}/ibus-engine-funput")

# --- IBus engine executable -> ibus-engine-funput --------------------------
add_executable(ibus-engine-funput src/main.cpp src/engine.cpp)
add_dependencies(ibus-engine-funput funput_ffi)

# funput_linux_common carries the FFI include dir + the .so transitively.
target_compile_definitions(ibus-engine-funput PRIVATE
    "FUNPUT_VERSION=\"${FUNPUT_VERSION}\""
    "FUNPUT_IBUS_ENGINE_PATH=\"${IBUS_ENGINE_PATH}\"")
target_link_libraries(ibus-engine-funput PRIVATE
    PkgConfig::IBUS
    funput_linux_common)

# libfunput_ffi.so is installed next to the engine binary; resolve it via $ORIGIN.
set_target_properties(ibus-engine-funput PROPERTIES
    INSTALL_RPATH "$ORIGIN"
    BUILD_WITH_INSTALL_RPATH ON)

# --- Install ---------------------------------------------------------------
install(TARGETS ibus-engine-funput DESTINATION "${IBUS_LIBEXECDIR}")
install(FILES "${FUNPUT_FFI_SO}" DESTINATION "${IBUS_LIBEXECDIR}")

# Component manifest (exec path + version filled in from the vars above).
configure_file(data/funput.xml.in funput.xml @ONLY)
install(FILES "${CMAKE_CURRENT_BINARY_DIR}/funput.xml"
        DESTINATION "${CMAKE_INSTALL_DATADIR}/ibus/component")

# Engine icon (referenced as Icon=funput). Ship the scalable SVG so ibus-setup
# and the GNOME panel render it crisp at any requested size; the 512px PNG goes in
# its real size bucket. (The old 512px-logo-in-128x128/ made loaders draw it
# oversized — it overflowed the ibus-setup window.)
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
# Self-contained packaging: an IBus-only user still gets the Settings GUI. The
# shared funput-common package that would de-dup this is deferred to a later PR.
set(FUNPUT_SETTINGS_BIN
    "${CMAKE_CURRENT_SOURCE_DIR}/../src-tauri/target/release/funput-settings"
    CACHE FILEPATH "Path to the built Tauri Settings binary")

if(EXISTS "${FUNPUT_SETTINGS_BIN}")
    install(PROGRAMS "${FUNPUT_SETTINGS_BIN}" DESTINATION "${CMAKE_INSTALL_BINDIR}")
    install(FILES "${CMAKE_CURRENT_SOURCE_DIR}/../packaging/funput-settings.desktop"
            DESTINATION "${CMAKE_INSTALL_DATADIR}/applications")
else()
    message(WARNING "funput-settings binary not found at ${FUNPUT_SETTINGS_BIN}; "
                    "packaging the IBus engine only (no Settings GUI).")
endif()

# --- .deb packaging (CPack) ------------------------------------------------
# FUNPUT_VERSION is defined near the top (project() default; CI overrides with
# -DFUNPUT_VERSION=<tag>).
set(CPACK_GENERATOR "DEB")
set(CPACK_PACKAGING_INSTALL_PREFIX "/usr")
set(CPACK_PACKAGE_NAME "funput-ibus")
set(CPACK_PACKAGE_VERSION "${FUNPUT_VERSION}")
set(CPACK_PACKAGE_DESCRIPTION_SUMMARY "Vietnamese input method (Telex & VNI) — IBus engine")
set(CPACK_PACKAGE_CONTACT "Funput <hello@funput.app>")
set(CPACK_DEBIAN_PACKAGE_MAINTAINER "Funput <hello@funput.app>")
set(CPACK_DEBIAN_PACKAGE_SECTION "utils")
set(CPACK_DEBIAN_PACKAGE_HOMEPAGE "https://github.com/Funput/Funput")
# ibus must be present; the rest are picked up automatically via shlibdeps.
set(CPACK_DEBIAN_PACKAGE_DEPENDS "ibus")
set(CPACK_DEBIAN_PACKAGE_SHLIBDEPS ON)
# Architecture from the build host (amd64 / arm64 / …); mirror it in the filename
# so an arm64 build isn't mislabelled "amd64".
find_program(DPKG_CMD dpkg)
if(DPKG_CMD)
    execute_process(COMMAND ${DPKG_CMD} --print-architecture
        OUTPUT_VARIABLE FUNPUT_DEB_ARCH OUTPUT_STRIP_TRAILING_WHITESPACE)
else()
    set(FUNPUT_DEB_ARCH "amd64")
endif()
set(CPACK_DEBIAN_PACKAGE_ARCHITECTURE "${FUNPUT_DEB_ARCH}")
set(CPACK_PACKAGE_FILE_NAME "funput-ibus_${FUNPUT_VERSION}_${FUNPUT_DEB_ARCH}")
include(CPack)
