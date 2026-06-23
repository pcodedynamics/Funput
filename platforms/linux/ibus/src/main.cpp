// Process entry point for the Funput IBus engine. ibus-daemon launches this
// binary (per the <exec> in the installed component XML) with `--ibus`; it then
// connects to the bus, registers an IBusFactory bound to IBusFunputEngine, and runs
// the GLib main loop. `--xml` prints the component manifest and exits (a smoke
// test that needs no running IBus session).

#include <ibus.h>

#include <cstdio>
#include <cstring>

#include "engine.h"

#ifndef FUNPUT_VERSION
#define FUNPUT_VERSION "0.0.0"
#endif
#ifndef FUNPUT_IBUS_ENGINE_PATH
#define FUNPUT_IBUS_ENGINE_PATH "ibus-engine-funput"
#endif

namespace {

IBusBus *g_bus = nullptr;
IBusFactory *g_factory = nullptr;

// The component descriptor: identity + how ibus-daemon launches us, plus the one
// engine we expose. Used both for standalone registration and `--xml`. Mirrors
// data/funput.xml.in (both derive their exec path/version from the same CMake vars).
IBusComponent *makeComponent() {
    IBusComponent *component = ibus_component_new(
        "org.freedesktop.IBus.Funput",
        "Funput Vietnamese Input Method (Telex & VNI)",
        FUNPUT_VERSION,
        "MIT",
        "Funput",
        "https://github.com/Funput/Funput",
        FUNPUT_IBUS_ENGINE_PATH " --ibus",
        "funput");

    IBusEngineDesc *engine = ibus_engine_desc_new(
        "funput",                  // name
        "Funput",                  // longname
        "Vietnamese Telex & VNI",  // description
        "vi",                      // language
        "MIT",                     // license
        "Funput",                 // author
        "funput",                  // icon
        "us");                     // keyboard layout

    ibus_component_add_engine(component, engine);
    return component;
}

void onDisconnected(IBusBus * /*bus*/, gpointer /*user_data*/) {
    ibus_quit();
}

// Register the factory and either request our well-known name (when started by
// ibus-daemon) or register the component on the fly (standalone debugging).
void startEngine(gboolean execByIbus) {
    ibus_init();

    g_bus = ibus_bus_new();
    g_object_ref_sink(g_bus);
    g_signal_connect(g_bus, "disconnected", G_CALLBACK(onDisconnected), nullptr);

    g_factory = ibus_factory_new(ibus_bus_get_connection(g_bus));
    g_object_ref_sink(g_factory);
    ibus_factory_add_engine(g_factory, "funput", FUNPUT_TYPE_ENGINE);

    if (execByIbus) {
        ibus_bus_request_name(g_bus, "org.freedesktop.IBus.Funput", 0);
    } else {
        ibus_bus_register_component(g_bus, makeComponent());
    }

    ibus_main();
}

} // namespace

int main(int argc, char **argv) {
    gboolean execByIbus = FALSE;

    for (int i = 1; i < argc; ++i) {
        if (std::strcmp(argv[i], "--xml") == 0) {
            IBusComponent *component = makeComponent();
            GString *out = g_string_new(nullptr);
            ibus_component_output(component, out, 4);
            std::printf("%s\n", out->str);
            g_string_free(out, TRUE);
            return 0;
        }
        if (std::strcmp(argv[i], "--ibus") == 0) {
            execByIbus = TRUE;
        }
    }

    startEngine(execByIbus);
    return 0;
}
