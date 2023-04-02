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
        type: "ice", detail: {
            candidate: JSON.stringify(candidate.toJSON())
        }
    });
})

// TODO: Find a way to avoid using ts-ignore in those event listeners
ws.addEventListener("ice", (event) => {
    peerConnection.addIceCandidate(new RTCIceCandidate(JSON.parse(event.detail.candidate)));
});

ws.addEventListener("offer", async (event) => {
    await peerConnection.setRemoteDescription({
        type: "offer",
        sdp: event.detail.sdp
    });

    const answer = await peerConnection.createAnswer();
    await peerConnection.setLocalDescription(answer);

    ws.sendEvent({
        type: "answer", detail: {
            sdp: answer.sdp
        }
    })
});