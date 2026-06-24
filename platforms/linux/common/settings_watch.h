// Watches ~/.config/Funput/settings.json for live changes so the engine can apply
// new settings immediately, instead of waiting for the next focus-in mtime check.
//
// Framework-agnostic: it owns a non-blocking inotify fd and a `drain()` that reports
// whether settings.json changed. Each shell wires the fd into its own event loop —
// Fcitx5 via `EventLoop::addIOEvent`, IBus via `g_unix_fd_add` — and calls `drain()`
// when the fd becomes readable. The directory (not the file) is watched so a delete-
// and-recreate still fires.

#ifndef FUNPUT_SETTINGS_WATCH_H
#define FUNPUT_SETTINGS_WATCH_H

#include <string>

namespace funput {

class SettingsWatcher {
public:
    SettingsWatcher();
    ~SettingsWatcher();

    SettingsWatcher(const SettingsWatcher &) = delete;
    SettingsWatcher &operator=(const SettingsWatcher &) = delete;

    // The inotify fd to poll for readability, or -1 if the watch couldn't be set up
    // (in which case the caller falls back to the focus-in mtime check).
    int fd() const { return fd_; }

    // Drain all pending inotify events (non-blocking). Returns true if settings.json
    // was among them, coalescing multiple events into one reload.
    bool drain();

private:
    int fd_ = -1;
    int wd_ = -1;
    std::string dir_;      // ~/.config/Funput
    std::string filename_; // settings.json
};

} // namespace funput

#endif // FUNPUT_SETTINGS_WATCH_H
