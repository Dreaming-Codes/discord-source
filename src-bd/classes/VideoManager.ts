import {Utils} from "./Utils";
import {WS} from "./WS";

export class VideoManager {
    private videos: Map<number, HTMLVideoElement> = new Map();
    private ws: WS;

    constructor(ws: WS) {
        this.ws = ws;
    }


    //TODO: add support for picture-in-picture
    /**
     * This function is called when a new RTC_CONNECTION_VIDEO event is dispatched, and it's used to save the video element for future use and send it to the desktop app
     */
    public async onVideoStream(event: any) {
        //If the streamId is null, it means that the video has ended
        if (!event.streamId) {
            Utils.log(`Video ended for ${event.userId}`);
            this.videos.forEach((video, videoId) => {
                if (video.dataset.userId === event.userId && !document.body.contains(video)) {
                    this.ws.sendRemoveVideoStream(videoId);
                    this.videos.delete(parseInt(video.dataset.streamId));
                }
            });
            return;
        }

        Utils.log(`Video ${event.streamId} started from ${event.userId}, waiting for video element...`);
        const video = await Utils.waitForElm(`[data-selenium-video-tile="${event.userId}"] video`) as HTMLVideoElement;
        Utils.log(`Found video element for ${event.streamId} from ${event.userId}!`);

        //Adding userId to video element, so we can find it later when the stream ends
        video.dataset.userId = event.userId;

        this.videos.set(event.streamId, video);
        this.ws.sendNewVideoStream(event.streamId, event.userId);
    }
}