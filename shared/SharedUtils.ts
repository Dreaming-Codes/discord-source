export class SharedUtils {
    static delay(ms: number) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }

    public static forceH264Support(sdp) {
        let h264PayloadType;

        // Find the next available payload type for H.264
        for (let i = 96; i <= 127; i++) {
            const regex = new RegExp(`a=rtpmap:${i} .*/90000`);
            if (!sdp.match(regex)) {
                h264PayloadType = i;
                break;
            }
        }

        if (!h264PayloadType) {
            console.error('Failed to find an available payload type for H.264');
            return sdp;
        }

        const h264RtpMap = `a=rtpmap:${h264PayloadType} H264/90000`;
        const h264Fmtp = `a=fmtp:${h264PayloadType} level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=42001f`;
        const h264RtcpFb = [
            `a=rtcp-fb:${h264PayloadType} nack`,
            `a=rtcp-fb:${h264PayloadType} nack pli`,
            `a=rtcp-fb:${h264PayloadType} ccm fir`
        ].join('\r\n');

        const sdpLines = sdp.split('\r\n');
        const videoIndex = sdpLines.findIndex(line => line.startsWith('m=video'));

        if (videoIndex !== -1) {
            // Remove all lines that start with "a=rtpmap" and do not contain "H264" in them.
            sdpLines.filter(line => line.startsWith('a=rtpmap') && !line.includes('H264')).forEach(line => {
                const payloadType = line.split(':')[1].split(' ')[0];
                sdpLines.splice(sdpLines.indexOf(line), 1);

                // Remove all lines that start with "a=fmtp" and "a=rtcp-fb" for the same payload type.
                sdpLines.filter(innerLine => innerLine.startsWith('a=fmtp') || innerLine.startsWith('a=rtcp-fb')).forEach(innerLine => {
                    if (innerLine.includes(payloadType)) {
                        sdpLines.splice(sdpLines.indexOf(innerLine), 1);
                    }
                });
            });

            // Add H.264 payload type to the video media line
            const videoMediaLine = sdpLines[videoIndex];
            const videoPayloads = videoMediaLine.split(' ');
            videoPayloads.splice(3, 0, h264PayloadType);
            sdpLines[videoIndex] = videoPayloads.join(' ');

            // Add the H.264 codec information
            const codecInfo = [h264RtpMap, h264Fmtp, h264RtcpFb];
            const codecInfoIndex = sdpLines.findIndex(line => line.startsWith('a=rtpmap') && line.includes('H264'));
            sdpLines.splice(codecInfoIndex + 1, 0, ...codecInfo);
        }

        return sdpLines.join('\r\n');
    }

    public static forceVideoBandwidth(sdp, bandwidth) {
        sdp = sdp.replace(/(m=video.*\r\n)/g, `$1b=AS:${bandwidth}\r\n`);
        return sdp;
    }
}