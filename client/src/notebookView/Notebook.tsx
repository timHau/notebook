import Cell from "./Cell";
import { init } from "../store/cellSlice";
import { CellT } from "../types"
import { useAppDispatch } from "../store/hooks";
import { useEffect } from "react";
import { WsClientT } from "../api/ws";

export type NotebookProps = {
    notebook: any;
    wsClient: WsClientT;
}

function Notebook(props: NotebookProps) {
    const { notebook, wsClient } = props;
    const dispatch = useAppDispatch()

    useEffect(() => {
        let cells: CellT[] = notebook.topology.display_order.map((uuid: string) => notebook.topology.cells[uuid]);
        dispatch(init(cells));
    }, [notebook.topology.display_order]);

    const order = notebook.topology.display_order;
    return (
        <div className="min-w-3/4 max-w-6xl pt-5">
            <div className="flex justify-between mb-5">
                <h5 className="text-5xl">{notebook.title}</h5>
                <div className="text-xs">
                    <span className="mr-0.5">{notebook.language_info.name}</span>
                    <span>{notebook.language_info.version}</span>
                </div>
            </div>
            <div>
                {order.map((cellUuid: string) => <Cell
                    key={cellUuid}
                    cellUuid={cellUuid}
                    notebookUuid={notebook.uuid}
                    wsClient={wsClient}
                />)}
            </div>
        </div>
    )
}

export default Notebook