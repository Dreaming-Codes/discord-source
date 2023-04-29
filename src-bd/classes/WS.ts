import { TypedEventTarget } from 'typescript-event-target';
import { MessageType } from "../../src-tauri/bindings/MessageType";
import {Utils} from "./Utils";
import {MessageEventMap} from "../../shared/MappedMessageType";

export class WS extends TypedEventTarget<MessageEventMap> {
    private ws: WebSocket;
    private readonly port: number;
    private isClosed: boolean = false;

    constructor(port: number) {
        super();
        this.port = port;
    }
    /**
     * Wait for the websocket to connect
     */
    public async connect(): Promise<boolean> {
        if(this.isClosed) return false;

        this.ws = new WebSocket(`ws://localhost:${this.port}/discord`);

        const connectionState = await new Promise(resolve => {
            if (this.ws.readyState === WebSocket.OPEN) {
                resolve(true);
            }
            this.ws.addEventListener("error", async () => {
                resolve(false);
            })
            this.ws.addEventListener("open", () => {
                resolve(true);
            });
        }) as boolean;

        if(connectionState) {
            this.ws.addEventListener("message", (e) => this.eventHandler(e));

            this.ws.addEventListener("close", ()=>{
                Utils.log("Connection to Discord Source lost, retrying...");

                this.connect();
            })
        }else{
            return this.connect();
        }

        return connectionState;
    }

    /**
     * Close the websocket connection
     */
    public async close() {
        this.isClosed = true;
        this.ws.close();
    }

    public sendEvent(event: MessageType) {
        this.ws.send(JSON.stringify(event));
    }

    private eventHandler(event: MessageEvent<string>) {
        Utils.log(`Received event from the desktop app`, event.data)
        let data: MessageType = JSON.parse(event.data);
        // @ts-ignore TODO: Remove ts-ignore
        this.dispatchTypedEvent(data.type, new CustomEvent(data.type, {detail: data.detail}) as any);
    }
}