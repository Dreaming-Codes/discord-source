import { MessageType } from "../bindings/MessageType";
import { TypedEventTarget } from 'typescript-event-target';
import {MessageEventMap} from "../../shared/MappedMessageType";



export class WS extends TypedEventTarget<MessageEventMap> {
    private ws: WebSocket;

    constructor(url: string) {
        super();
        this.ws = new WebSocket(url);
        this.ws.addEventListener("message", (event)=>{
            let data: MessageType = JSON.parse(event.data);
            this.dispatchTypedEvent(data.type, new CustomEvent(data.type, {detail: data.detail}) as any);
        });

        this.ws.addEventListener("close", ()=>{
            console.error("Websocket connection closed, reloading");

            window.location.reload();
        });

        this.ws.addEventListener("error", ()=>{
            console.error("Error during websocket connection, closing");
            this.ws.close();
        });
    }

    public sendEvent(event: MessageType) {
        this.ws.send(JSON.stringify(event));
    }
}