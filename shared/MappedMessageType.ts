import {MessageType} from '../src-tauri/bindings/MessageType';

type TypeNames = MessageType["type"];
// @ts-ignore TODO: fix this
type DetailType<T extends TypeNames> = Extract<MessageType, { type: T }>["detail"];

// Define the utility type for the intersection type
type CustomEventMapMember<T extends TypeNames> = {
    [K in T]: CustomEvent<DetailType<K>>
};

export type MessageEventMap = {
    [K in TypeNames]: CustomEventMapMember<K>[K];
}