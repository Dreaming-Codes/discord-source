export class WS {
    private ws: WebSocket;

    constructor(private port: number) {
        this.ws = new WebSocket(`ws://localhost:${port}/?role=discord`);
    }

    /**
     * Wait for the websocket to connect
     */
    public async connect(): Promise<boolean> {
        return new Promise(resolve => {
            if (this.ws.readyState === WebSocket.OPEN) {
                resolve(true);
            }
            this.ws.addEventListener("error", async () => {
                resolve(false);
            })
            this.ws.addEventListener("open", () => {
                resolve(true);
            });
        })
    }

    /**
     * Send a new video stream to the desktop app
     */
    public sendNewVideoStream(streamId: number, userId: string) {
        this.ws.send(JSON.stringify({
            operation: "add",
            userId,
            streamId
        }));
    }

    /**
     * Send a remove video stream to the desktop app
     */
    public sendRemoveVideoStream(streamId: number) {
        this.ws.send(JSON.stringify({
            operation: "remove",
            streamId
        }));
    }
}