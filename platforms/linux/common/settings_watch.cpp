#include "settings_watch.h"

#include <cstring>
#include <filesystem>
#include <system_error>

#include <sys/inotify.h>
#include <unistd.h>

#include "settings.h"

namespace funput {

SettingsWatcher::SettingsWatcher() {
    const std::string p = Settings::path();
    if (p.empty()) return;
    const auto slash = p.rfind('/');
    if (slash == std::string::npos) return;
    dir_ = p.substr(0, slash);
    filename_ = p.substr(slash + 1);

    // Best-effort: ensure the config dir exists so the watch can be added even before
    // the first save (errors are ignored — the watch just won't be set up).
    std::error_code ec;
    std::filesystem::create_directories(dir_, ec);

    fd_ = ::inotify_init1(IN_NONBLOCK | IN_CLOEXEC);
    if (fd_ < 0) return;

    // Watch the directory (not the file): the Settings app and our own save() rewrite
    // the file, and a recreate would invalidate a file-level watch.
    wd_ = ::inotify_add_watch(fd_, dir_.c_str(),
                              IN_CLOSE_WRITE | IN_MOVED_TO | IN_CREATE | IN_MODIFY);
    if (wd_ < 0) {
        ::close(fd_);
        fd_ = -1;
    }
}

SettingsWatcher::~SettingsWatcher() {
    if (fd_ >= 0) ::close(fd_); // closing the fd also removes the watch
}

bool SettingsWatcher::drain() {
    if (fd_ < 0) return false;

    bool hit = false;
    // Buffer big enough for several events; the struct is followed by a variable name.
    alignas(struct inotify_event) char buf[4096];
    for (;;) {
        const ssize_t n = ::read(fd_, buf, sizeof(buf));
        if (n <= 0) break; // EAGAIN (drained) or error → stop
        ssize_t i = 0;
        while (i < n) {
            auto *ev = reinterpret_cast<struct inotify_event *>(buf + i);
            if (ev->len > 0 && filename_ == ev->name) hit = true;
            i += static_cast<ssize_t>(sizeof(struct inotify_event)) + ev->len;
        }
    }
    return hit;
}

} // namespace funput
