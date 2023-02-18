import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { BindingT, CellT } from "../types";

interface cellsState {
    mappings: {
        [key: string]: CellT
    },
    bindings: BindingT
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
        updateBinding: (state, action: PayloadAction<BindingT>) => {
            let cells = action.payload;
            for (let uuid in action.payload) {
                state.bindings[uuid] = cells[uuid];
                state.mappings[uuid].isSynced = true;
            }
        },
    },
});

export const { init, updateBinding } = cellsSlice.actions;

export default cellsSlice.reducer;
