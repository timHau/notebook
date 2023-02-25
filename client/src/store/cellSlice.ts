import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { WsMessage } from "../api/ws";
import { CellT, LocalsT } from "../types";

interface cellsState {
    mappings: {
        [key: string]: CellT
    },
    bindings: LocalsT
    output: {
        [key: string]: WsMessage
    },
}

const initialState: cellsState = {
    mappings: {},
    bindings: {},
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
        updateBinding: (state, action: PayloadAction<LocalsT>) => {
            let cells = action.payload;
            for (let uuid in action.payload) {
                let locals = cells[uuid];
                state.bindings[uuid] = locals;
                state.mappings[uuid].isSynced = true;
            }
        },
        unsyncCell: (state, action: PayloadAction<string>) => {
            state.mappings[action.payload].isSynced = false;
        },
        addOutput: (state, action: PayloadAction<WsMessage>) => {
            let msg = action.payload;
            state.output[msg.cellUuid] = msg;
        },
    },
});

export const { initCell, updateBinding, unsyncCell, addOutput } = cellsSlice.actions;

export default cellsSlice.reducer;
