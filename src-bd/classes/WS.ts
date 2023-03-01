import {Utils} from "./Utils";
import {TypedEventTarget} from "./TypedEventTarget";
import { SignalType } from "../../src-tauri/bindings/SignalType";

export type StartStreamEvent = {
    readonly type: 'startCaptureStream', readonly streamId: number,
};
export type EndStreamEvent = {
    readonly type: 'endCaptureStream'
    readonly streamId: number,
};

type WSEvent = StartStreamEvent | EndStreamEvent;

export class WS extends TypedEventTarget<WSEvent> {
    private ws: WebSocket;

    constructor(private port: number) {
        super();
        this.ws = new WebSocket(`ws://localhost:${port}/discord`);
        this.ws.addEventListener("message", this.eventHandler);
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
     * Close the websocket connection
     */
    public async close() {
        this.ws.close();
    }

    public sendEvent(event: SignalType) {
        this.ws.send(JSON.stringify(event));
    }

    private eventHandler(event: MessageEvent<any>) {
        switch (event.data.operation) {
            case "startCaptureStream":
                this.dispatch({
                    type: "startCaptureStream", streamId: event.data.streamId
                });
                break;
            case "endCaptureStream":
                this.dispatch({
                    type: "endCaptureStream", streamId: event.data.streamId
                })
                break;
            default:
                Utils.warn("Unknown operation", event.data.operation);
        }
    }
}