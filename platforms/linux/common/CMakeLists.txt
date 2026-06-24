# Shared Linux code across input-method shells (Fcitx5, IBus): the settings.json
# reader, the RAII wrapper around funput-ffi's C ABI, the word-boundary test, and
# the build of the Rust cdylib itself. Framework-agnostic.
#
# Each shell `add_subdirectory()`s this and links `funput_linux_common`, inheriting
# (PUBLIC) the include dir for "settings.h"/"ffi_handle.h"/"boundary.h", the FFI
# header path for "funput.h", and the built libfunput_ffi.so. The shells only add
# their own framework deps, the $ORIGIN rpath, and install the .so next to their
# own binary (paths exposed below as cache vars).
find_package(nlohmann_json 3 REQUIRED)
find_program(CARGO_EXECUTABLE cargo REQUIRED)

# --- Rust core (funput-ffi) -> libfunput_ffi.so ----------------------------
# The pure typing logic lives in Rust; the shells are thin C++ wrappers calling
# the cbindgen-generated C ABI. Built once here, consumed by every shell.
# Exposed as CACHE INTERNAL so the parent (shell) scope can install() the .so and
# locate the app root for the icon.
get_filename_component(_funput_root "${CMAKE_CURRENT_SOURCE_DIR}/../../.." ABSOLUTE)
set(FUNPUT_ROOT "${_funput_root}" CACHE INTERNAL "Funput app root")
set(FUNPUT_FFI_INCLUDE "${FUNPUT_ROOT}/crates/funput-ffi/include" CACHE INTERNAL "FFI header dir")
set(FUNPUT_FFI_SO "${FUNPUT_ROOT}/target/release/libfunput_ffi.so" CACHE INTERNAL "Built FFI cdylib")

add_custom_command(
    OUTPUT "${FUNPUT_FFI_SO}"
    COMMAND ${CARGO_EXECUTABLE} build --release -p funput-ffi
    WORKING_DIRECTORY "${FUNPUT_ROOT}"
    COMMENT "Building funput-ffi (Rust cdylib)"
    VERBATIM
)
add_custom_target(funput_ffi DEPENDS "${FUNPUT_FFI_SO}")

# --- Shared static lib ------------------------------------------------------
add_library(funput_linux_common STATIC settings.cpp settings_watch.cpp)
target_compile_features(funput_linux_common PUBLIC cxx_std_17)
# This static lib is linked into shared objects (the Fcitx5 addon MODULE) and PIE
# executables (the IBus engine), so its objects must be position-independent.
set_target_properties(funput_linux_common PROPERTIES POSITION_INDEPENDENT_CODE ON)
add_dependencies(funput_linux_common funput_ffi)

# PUBLIC so consumers inherit: the common headers, funput.h, and the .so to link.
target_include_directories(funput_linux_common PUBLIC
    "${CMAKE_CURRENT_SOURCE_DIR}"
    "${FUNPUT_FFI_INCLUDE}")
target_link_libraries(funput_linux_common PUBLIC
    nlohmann_json::nlohmann_json
    "${FUNPUT_FFI_SO}")
