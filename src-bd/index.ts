export default class DiscordSourcePlugin{
    private observer: MutationObserver;

    start() {
        this.log("Plugin started");

        this.observer = new MutationObserver(mutations => {
            if (mutations.filter(mut => mut.addedNodes.length === 0 && mut.target.hasChildNodes()).length == 0) return;

            Array.from(document.getElementsByTagName("video")).forEach((video: HTMLVideoElement) => {
                if (video.dataset.discordSourcePluginEnabled) return;
                video.dataset.discordSourcePluginEnabled = "true";
                const videoAuthour = video.parentElement?.parentElement?.parentElement?.querySelector("[class*='overlayTitleText']")?.textContent;
                if (!videoAuthour) return;
                this.log("Found video from", videoAuthour);
                ((video as any).captureStream() as MediaStream).getVideoTracks().forEach(track => {
                    this.log(track)
                })
            })
        })
        this.observer.observe(document, {childList: true, subtree: true});
    }

    log(...msg: any[]) {
        console.log("DiscordSourcePlugin", ...msg);
    }

    stop() {
        this.observer.disconnect();
        document.querySelectorAll("video").forEach(video => {
            delete video.dataset.discordSourcePluginEnabled;
        });
        this.log("Plugin stopped");
    }
}