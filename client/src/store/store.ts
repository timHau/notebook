import { configureStore } from '@reduxjs/toolkit';
import cellsReducer from "./cellSlice";
import wsReducer from "./wsSlice";

const store = configureStore({
    middleware: (getDefaultMiddleware) => getDefaultMiddleware({
        serializableCheck: false,
    }),
    reducer: {
        cells: cellsReducer,
        ws: wsReducer,
    },
});

export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;

export default store;