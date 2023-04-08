import DiscordSourcePlugin from "../index";

export class Utils {
    /**
     * Waits for an element to be added to the DOM
     * @param selector The query selector of the element
     */
    static waitForElm(selector) {
        return new Promise(resolve => {
            if (document.querySelector(selector)) {
                return resolve(document.querySelector(selector));
            }

            const observer = new MutationObserver(() => {
                if (document.querySelector(selector)) {
                    resolve(document.querySelector(selector));
                    observer.disconnect();
                }
            });

            observer.observe(document.body, {
                childList: true,
                subtree: true
            });
        });
    }

    /**
     * Waits for a confirmation modal to be accepted or rejected
     */
    static async asyncShowConfirmationModal(...args: Parameters<typeof BdApi.UI.showConfirmationModal>): Promise<boolean> {
        return new Promise(resolve => {
            const oldOnConfirm = args[2].onConfirm;
            const oldOnCancel = args[2].onCancel;

            args[2].onConfirm = () => {
                if(oldOnConfirm){
                    oldOnConfirm();
                }
                resolve(true);
            }

            args[2].onCancel = () => {
                if(oldOnCancel){
                    oldOnCancel();
                }
                resolve(false);
            }

            BdApi.UI.showConfirmationModal(...args);
        });
    }

    static log(...msg: any[]) {
        console.log(...Utils.logString(...msg));
    }

    static warn(...msg: any[]) {
        console.warn(...Utils.logString(...msg));
    }

    static error(...msg: any[]) {
        console.error(...Utils.logString(...msg));
    }

    private static logString(...msg: any[]) {
        return [`%c[${DiscordSourcePlugin.name}]`, 'color: #bada55', ...msg];
    }

    public static  addH264Support(sdp) {
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
        const h264Fmtp = `a=fmtp:${h264PayloadType} 	level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=42001f`;
        const h264RtcpFb = [
            `a=rtcp-fb:${h264PayloadType} nack`,
            `a=rtcp-fb:${h264PayloadType} nack pli`,
            `a=rtcp-fb:${h264PayloadType} ccm fir`
        ].join('\r\n');

        const sdpLines = sdp.split('\r\n');
        const videoIndex = sdpLines.findIndex(line => line.startsWith('m=video'));

        if (videoIndex !== -1) {
            // Add H.264 payload type to the video media line
            const videoMediaLine = sdpLines[videoIndex];
            const videoPayloads = videoMediaLine.split(' ');
            videoPayloads.splice(3, 0, h264PayloadType);
            sdpLines[videoIndex] = videoPayloads.join(' ');

            // Add the H.264 codec information
            const codecInfo = [h264RtpMap, h264Fmtp, h264RtcpFb];
            const codecInfoIndex = sdpLines.findIndex(line => line.startsWith('a=rtpmap'));
            sdpLines.splice(codecInfoIndex, 0, ...codecInfo);
        }

        const modifiedSdp = sdpLines.join('\r\n');
        return modifiedSdp;
    }


}