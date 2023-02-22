import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { BindingT, CellT, LocalsT } from "../types";

interface cellsState {
    mappings: {
        [key: string]: CellT
    },
    bindings: LocalsT
}

const initialState: cellsState = {
    mappings: {},
    bindings: {}
}

export const cellsSlice = createSlice({
    name: "cells",
    initialState,
    reducers: {
        init: (state, action: PayloadAction<CellT[]>) => {
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
        }
    },
});

export const { init, updateBinding, unsyncCell } = cellsSlice.actions;

export default cellsSlice.reducer;
