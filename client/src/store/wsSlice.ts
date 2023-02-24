import { createSlice, PayloadAction } from "@reduxjs/toolkit";

interface wsStore {
    ws: WebSocket | null
}

const initialState: wsStore = {
    ws: null
}

export const wsSlice = createSlice({
    name: "ws",
    initialState,
    reducers: {
        initWs: (state, action: PayloadAction<string>) => {
            state.ws = new WebSocket(action.payload);
        }
    },
})

export const { initWs } = wsSlice.actions;

export default wsSlice.reducer;