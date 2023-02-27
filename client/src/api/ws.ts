export type WsClientT = {
    ws: WebSocket;
    send: (data: WsMessage) => void;
};

export enum WsCmds {
    Run = 'Run',
    Res = 'Res',
    Err = 'Err',
    Ping = 'Ping',
    Pong = 'Pong',
}

export type WsMessage = {
    cmd: WsCmds;
    data?: string;
    content?: string;
    cellUuid: string;
    locals?: any;
    bindings?: string[];
};

export type WsMessageEvent = {
    data: string;
};

export type WsCloseEvent = {
    wasClean: boolean;
    code: number;
    reason: string;
};

export type WsErrorEvent = {
    type: string;
    message: string;
};