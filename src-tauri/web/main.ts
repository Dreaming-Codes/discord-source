import {WS} from "./WS";

const video = document.getElementById('video') as HTMLVideoElement;

// @ts-ignore
const ws = new WS(`ws://127.0.0.1:${window.ws_port}/${window.location.pathname.substring(1)}`);

let peerConnection = new RTCPeerConnection();

peerConnection.addEventListener("track", (event) => {
    console.log("Received track!");
    video.srcObject = new MediaStream([event.track]);
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

ws.addEventListener("ice", (event) => {
    console.log("Received ice!");
    peerConnection.addIceCandidate(new RTCIceCandidate(JSON.parse(event.detail.candidate)));
});

ws.addEventListener("offer", async (event) => {
    console.log("Received offer!");
    await peerConnection.setRemoteDescription({
        type: "offer",
        sdp: event.detail.sdp
    });

    const answer = await peerConnection.createAnswer();
    //answer.sdp = SharedUtils.forceH264Support(answer.sdp);
    //answer.sdp = SharedUtils.forceVideoBandwidth(answer.sdp, 90000);
    await peerConnection.setLocalDescription(answer);

    ws.sendEvent({
        type: "answer", detail: {
            sdp: answer.sdp
        }
    })
});

ws.addEventListener("unlink", async () =>{
    peerConnection.close();
    peerConnection = new RTCPeerConnection();

    video.srcObject = null;
})