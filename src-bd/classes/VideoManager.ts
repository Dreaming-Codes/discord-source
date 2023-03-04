import {Utils} from "./Utils";
import {WS} from "./WS";
import {WebRTCStream} from "./WebRTCStream";
import {CaptureEvent} from "../../src-tauri/bindings/CaptureEvent";

export class VideoManager {
    private videos: Map<number, HTMLVideoElement> = new Map();
    private ws: WS;
    private streams: Map<number, WebRTCStream> = new Map();

    constructor(ws: WS) {
        this.ws = ws;
        // TODO: Find a way to avoid using ts-ignore in those event listeners
        // @ts-ignore
        this.ws.addEventListener("capture", this.onRequestCaptureVideoStream);
        // @ts-ignore
        this.ws.addEventListener("endCapture", this.onEndCaptureVideoStream);
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
                    this.ws.sendEvent({
                        type: "remove",
                        data: {
                            streamId: videoId
                        }
                    })
                    this.videos.delete(parseInt(video.dataset.streamId));
                }
            });
            return;
        }

        Utils.log(`Video ${event.streamId} started from ${event.userId}, waiting for video element...`);
        const video = await Utils.waitForElm(`[data-selenium-video-tile="${event.userId}"] video`) as HTMLVideoElement;
        Utils.log(`Found video element for ${event.streamId} from ${event.userId}!`);

        //Adding userId to the video element, so we can find it later when the stream ends
        video.dataset.userId = event.userId;

        let streamId = parseInt(event.streamId);

        this.videos.set(streamId, video);
        this.ws.sendEvent({
            type: "add",
            data: {
                streamId: streamId,
                userId: event.userId
            }
        })
    }

    public async stop() {
        await this.ws.close();
        this.streams.forEach(stream => stream.close());
    }

    private onRequestCaptureVideoStream(event: Event & { data: CaptureEvent }) {
        const video = this.videos.get(event.data.streamId);
        if (!video) return;

        const stream = new WebRTCStream(video.captureStream());

        stream.peerConnection.addEventListener("icecandidate", ({candidate}) => {
            if (!candidate) {
                return;
            }
            this.ws.sendEvent({
                type: "ice",
                data: {
                    streamId: event.data.streamId,
                    candidate: String(candidate)
                }
            })
        })

        stream.start();
    }

    private onEndCaptureVideoStream(event: Event & { data: CaptureEvent }) {
        const stream = this.streams.get(event.data.streamId);
        if (!stream) return;
        stream.close();
        this.streams.delete(event.data.streamId);
    }

}