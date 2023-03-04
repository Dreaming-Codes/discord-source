import {TypedEventTarget} from "../../shared/TypedEventTarget";
import { MessageType } from "../../src-tauri/bindings/MessageType";

export class WS extends TypedEventTarget<MessageType> {
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

    public sendEvent(event: MessageType) {
        this.ws.send(JSON.stringify(event));
    }

    private eventHandler(event: MessageEvent<MessageType>) {
        this.dispatch(event.data);
    }
}