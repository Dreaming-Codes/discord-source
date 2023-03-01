const video = document.getElementById('video');

// @ts-ignore
const ws = new WebSocket(`ws://localhost:${window.ws_port}/${window.location.pathname.substring(1)}`);

const peerConnection = new RTCPeerConnection();

peerConnection.addEventListener("track", (event) => {
    //video.srcObject = event.streams[0];
})

peerConnection.addEventListener("icecandidate", ({candidate}) => {
    if (!candidate) {
        return;
    }

    ws.send(JSON.stringify({
        type: "iceCandidate",
        candidate
    }));
})

ws.addEventListener("message", async (event) => {
    switch (event.type){
        case "offer":
            // @ts-ignore
            await peerConnection.setRemoteDescription(new RTCSessionDescription(event.offer));

            const answer = await peerConnection.createAnswer();
            await peerConnection.setLocalDescription(answer);

            ws.send(JSON.stringify({
                type: "answer",
                data: {
                    sdp: answer
                }
            }));
            break;
        case "answer":
            // @ts-ignore
            await peerConnection.setRemoteDescription(new RTCSessionDescription(event.answer));
            break;
        case "iceCandidate":
            // @ts-ignore
            await peerConnection.addIceCandidate(new RTCIceCandidate(event.candidate));
            break;
        default:
            console.warn("Unknown message type", event.type);
    }
})