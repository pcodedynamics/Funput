import AppKit
import Combine
import Observation
import Sparkle

/// Bridges Sparkle into the SwiftUI app for the "Kiểm tra cập nhật…" feature.
///
/// Funput is an input method that the system loads into other processes, so a
/// freshly installed build can't take effect with an in-place relaunch — apps
/// that already mapped the old binary keep using it. We therefore let Sparkle
/// download and install the update, but replace its silent relaunch with an
/// explicit "log out / back in" prompt (see `UpdaterDelegate`).
///
/// Checks are manual-only: `SUEnableAutomaticChecks` is `false` in Info.plist,
/// so the updater never polls in the background.
@MainActor
@Observable
final class UpdaterManager {
    /// Drives the enabled state of the "Kiểm tra cập nhật…" buttons. Sparkle
    /// flips this off briefly while a check is already running.
    private(set) var canCheckForUpdates = false

    @ObservationIgnored private let controller: SPUStandardUpdaterController
    @ObservationIgnored private let delegate = UpdaterDelegate()
    @ObservationIgnored private var cancellable: AnyCancellable?

    init() {
        // startingUpdater: true → start the updater now. With automatic checks
        // disabled this just wires up the manual check path (no network I/O).
        controller = SPUStandardUpdaterController(
            startingUpdater: true,
            updaterDelegate: delegate,
            userDriverDelegate: nil
        )

        cancellable = controller.updater
            .publisher(for: \.canCheckForUpdates)
            .receive(on: RunLoop.main)
            .sink { [weak self] value in
                self?.canCheckForUpdates = value
            }
    }

    /// Shows Sparkle's update UI. The menu bar agent has no Dock icon, so pull
    /// the app forward first or the update window opens behind everything.
    func checkForUpdates() {
        NSApp.activate(ignoringOtherApps: true)
        controller.updater.checkForUpdates()
    }
}

/// Swaps Sparkle's silent in-place relaunch for an explicit re-login prompt.
private final class UpdaterDelegate: NSObject, SPUUpdaterDelegate {
    /// Postpone the relaunch until the user acknowledges that an input method
    /// needs a fresh login session to load. Invoking `installHandler` lets
    /// Sparkle finish installing the new bundle.
    func updater(
        _ updater: SPUUpdater,
        shouldPostponeRelaunchForUpdate item: SUAppcastItem,
        untilInvokingBlock installHandler: @escaping () -> Void
    ) -> Bool {
        let alert = NSAlert()
        alert.messageText = "Đã tải bản cập nhật Funput"
        alert.informativeText = """
        Funput sẽ được cập nhật ngay bây giờ. Vì Funput là bộ gõ do hệ thống nạp, \
        bạn cần đăng xuất rồi đăng nhập lại để bản mới có hiệu lực.
        """
        alert.addButton(withTitle: "Đồng ý")
        NSApp.activate(ignoringOtherApps: true)
        alert.runModal()
        installHandler()
        return true
    }
}
