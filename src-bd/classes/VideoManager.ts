import {Utils} from "./Utils";
import {WS} from "./WS";
import {WebRTCStream} from "./WebRTCStream";
import {CaptureEvent} from "../../src-tauri/bindings/CaptureEvent";
import {ICEEvent} from "../../src-tauri/bindings/ICEEvent";
import {AnswerOfferEvent} from "../../src-tauri/bindings/AnswerOfferEvent";
import {SharedUtils} from "../../shared/SharedUtils";

export class VideoManager {
    private videos: Map<number, HTMLVideoElement> = new Map();
    private ws: WS;
    private streams: Map<number, WebRTCStream> = new Map();

    constructor(ws: WS) {
        this.ws = ws;
        this.ws.addEventListener("capture", (e) => this.onRequestCaptureVideoStream(e));
        this.ws.addEventListener("endCapture", (e) => this.onEndCaptureVideoStream(e));
        this.ws.addEventListener("answer", (e) => this.onAnswerEvent(e));
        this.ws.addEventListener("ice", (e) => this.onIceCandidateEvent(e));
    }


    //TODO: add support for picture-in-picture
    /**
     * This function is called when a new RTC_CONNECTION_VIDEO event is dispatched, and it's used to save the video element for future use and send it to the desktop app
     */
    public async onVideoStream(event: any) {
        //If the streamId is null, it means that the video has ended
        if (!event.streamId) {
            Utils.log(`Video ended for ${event.userId}`);
            //We need to wait a bit, because the video element is removed from the DOM after the event is dispatched
            await SharedUtils.delay(500);

            for (let [videoId, video] of this.videos.entries()) {
                if (video.dataset.userId === event.userId && !document.body.contains(video)) {
                    this.ws.sendEvent({
                        type: "remove",
                        detail: {
                            streamId: videoId
                        }
                    })
                    this.videos.delete(videoId);
                    break;
                }
            }
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
            detail: {
                streamId: streamId,
                userId: event.userId
            }
        })
    }

    public async stop() {
        await this.ws.close();
        this.streams.forEach(stream => stream.close());
    }

    private async onRequestCaptureVideoStream(event: CustomEvent<CaptureEvent>) {
        const video = this.videos.get(event.detail.streamId);
        if (!video) {
            Utils.error("Received capture request for unknown stream");
            return
        }

        Utils.log(`Received capture request for stream ${event.detail.streamId}!`)

        const stream = new WebRTCStream(video.captureStream());

        stream.peerConnection.addEventListener("icecandidate", ({candidate}) => {
            if (!candidate) {
                return;
            }
            this.ws.sendEvent({
                type: "ice",
                detail: {
                    streamId: event.detail.streamId,
                    candidate: JSON.stringify(candidate.toJSON())
                }
            })
        });

        const offer = await stream.start();

        this.ws.sendEvent({
            type: "offer",
            detail: {
                sdp: offer.sdp,
                streamId: event.detail.streamId
            }
        })
    }

    private onAnswerEvent(event: CustomEvent<AnswerOfferEvent>) {
        const stream = this.streams.get(event.detail.streamId);
        if (!stream) {
            Utils.error("Received answer for unknown stream");
            return;
        }
        Utils.log("Received answer");
        stream.peerConnection.setRemoteDescription({
            type: "answer",
            sdp: event.detail.sdp
        });
    }

    private onEndCaptureVideoStream(event: CustomEvent<CaptureEvent>) {
        const stream = this.streams.get(event.detail.streamId);
        if (!stream) return;
        stream.close();
        this.streams.delete(event.detail.streamId);
    }

    private onIceCandidateEvent(event: CustomEvent<ICEEvent>) {
        const stream = this.streams.get(event.detail.streamId);
        if (!stream) {
            Utils.error("Received ICE candidate for unknown stream");
            return;
        }
        Utils.log("Received ICE candidate");
        stream.peerConnection.addIceCandidate(new RTCIceCandidate(JSON.parse(event.detail.candidate)));
    }

}