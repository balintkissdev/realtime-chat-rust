/**
 * Message payload is the same JSON event payload that is passed and serialized
 * by the server backend.
 */
export interface Payload {
    event_type: PayloadEventType,
    username: string,
    message?: string
}

export enum PayloadEventType {
    Connected = 'connected',
    Disconnected = 'disconnected',
    Message = 'message',
}

export const payloadToMessageLine = (payload: Payload) => {
    switch (payload.event_type) {
        case PayloadEventType.Connected:
            return `${payload.username} has joined the chat.`;
        case PayloadEventType.Disconnected:
            return `${payload.username} has left the chat.`;
        case PayloadEventType.Message:
            return `[${payload.username}]: ${payload.message}`;
    }
}
