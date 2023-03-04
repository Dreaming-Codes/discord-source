import {WS} from "./classes/WS";
import {Settings} from "./classes/Settings";
import {Utils} from "./classes/Utils";
import {VideoManager} from "./classes/VideoManager";

export default class DiscordSourcePlugin {
    static videoManager: VideoManager;
    private Dispatcher = BdApi.Webpack.getModule(BdApi.Webpack.Filters.byProps("dispatch", "register"));

    async start() {
        //TODO: Remove when the plugin is ready and settings are automatically created by the app
        BdApi.Data.save(DiscordSourcePlugin.name, "wsPort", 8214);
        if (!Settings.getPort()) {
            await Utils.asyncShowConfirmationModal("Discord Source", "Settings not found, please install the plugin from the Discord Source desktop app", {
                danger: true,
                confirmText: "Open Discord Source website",
                cancelText: "Disable plugin",
                onConfirm: () => {
                    window.open("https://github.com/Dreaming-Codes/discord-source");
                },
            })

            BdApi.Plugins.disable(DiscordSourcePlugin.name);
            return;
        }

        Utils.log("Connecting to Discord Source...");
        const ws = new WS(Settings.getPort());
        if (!await ws.connect()) {
            await Utils.asyncShowConfirmationModal("Discord Source", "Failed to connect to Discord Source, please make sure that the app is started", {
                danger: true,
                confirmText: "Open Discord Source website",
                cancelText: "Disable plugin",
                onConfirm: () => {
                    window.open("https://github.com/Dreaming-Codes/discord-source");
                }
            });

            BdApi.Plugins.disable(DiscordSourcePlugin.name);
            return;
        }

        DiscordSourcePlugin.videoManager = new VideoManager(ws);

        this.Dispatcher.subscribe("RTC_CONNECTION_VIDEO", this.onVideoStream);
        Utils.log("Plugin started");
    }

    onVideoStream(event: any) {
        DiscordSourcePlugin.videoManager.onVideoStream(event);
    }

    stop() {
        this.Dispatcher.unsubscribe("RTC_CONNECTION_VIDEO", this.onVideoStream);
        DiscordSourcePlugin.videoManager.stop();
        Utils.log("Plugin stopped");
    }
}