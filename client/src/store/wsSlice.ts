import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { WsMessage } from "../api/ws";

interface wsState {
    socket: WebSocket | null,
}

const initialState: wsState = {
    socket: null,
}

export const wsSlice = createSlice({
    name: "ws",
    initialState,
    reducers: {
        initWs: (state, action: PayloadAction<WebSocket>) => {
            state.socket = action.payload;
        },
        send(state, action: PayloadAction<WsMessage>) {
            if (state.socket) {
                state.socket.send(JSON.stringify(action.payload));
            }
        }
    },
});

export const { initWs, send } = wsSlice.actions;

export default wsSlice.reducer;