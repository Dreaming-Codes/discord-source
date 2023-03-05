import {ICEEvent} from "../bindings/ICEEvent";
import {AnswerOfferEvent} from "../bindings/AnswerOfferEvent";

import {WS} from "./WS";


const video = document.getElementById('video') as HTMLVideoElement;

// @ts-ignore
const ws = new WS(`ws://127.0.0.1:${window.ws_port}/${window.location.pathname.substring(1)}`);

const peerConnection = new RTCPeerConnection();

peerConnection.addEventListener("track", (event) => {
    video.srcObject = event.streams[0];
})

peerConnection.addEventListener("icecandidate", ({candidate}) => {
    if (!candidate) {
        return;
    }

    ws.sendEvent({
        type: "ice",
        data: {
            candidate: String(candidate)
        }
    });
})

// TODO: Find a way to avoid using ts-ignore in those event listeners
// @ts-ignore
ws.addEventListener("ice", (event: Event & { data: ICEEvent }) => {
    peerConnection.addIceCandidate(new RTCIceCandidate(event.data.candidate as RTCIceCandidateInit));
});

// @ts-ignore
ws.addEventListener("offer", async (event: Event & { data: AnswerOfferEvent }) => {
    await peerConnection.setRemoteDescription(new RTCSessionDescription(event.data.sdp as unknown as RTCSessionDescriptionInit));

    const answer = await peerConnection.createAnswer();
    await peerConnection.setLocalDescription(answer);

    ws.sendEvent({
        type: "answer",
        data: {
            sdp: String(answer)
        }
    })
});

// @ts-ignore
ws.addEventListener("answer", async (event: Event & { data: AnswerOfferEvent }) => {
    await peerConnection.setRemoteDescription(new RTCSessionDescription(event.data.sdp as unknown as RTCSessionDescriptionInit));
});