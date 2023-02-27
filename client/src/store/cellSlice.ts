import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { WsMessage } from "../api/ws";
import { CellT, LocalsT } from "../types";

interface cellsState {
    mappings: {
        [key: string]: CellT
    },
    output: {
        [key: string]: WsMessage
    },
}

const initialState: cellsState = {
    mappings: {},
    output: {},
}

export const cellsSlice = createSlice({
    name: "cells",
    initialState,
    reducers: {
        initCell: (state, action: PayloadAction<CellT[]>) => {
            for (let cell of action.payload) {
                state.mappings[cell.uuid] = cell;
            }
        },
        unsyncCell: (state, action: PayloadAction<string>) => {
            state.mappings[action.payload].isSynced = false;
        },
        addOutput: (state, action: PayloadAction<WsMessage>) => {
            let msg = action.payload;
            state.output[msg.cellUuid] = msg;
        },
        updateCellContent: (state, action: PayloadAction<{ uuid: string, content: string }>) => {
            state.mappings[action.payload.uuid].content = action.payload.content;
        }
    },
});

export const { initCell, unsyncCell, updateCellContent, addOutput } = cellsSlice.actions;

export default cellsSlice.reducer;
