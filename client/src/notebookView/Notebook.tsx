import Cell from "./Cell";
import { init } from "../store/cellSlice";
import { CellT, NotebookProps } from "../types"
import { useAppDispatch } from "../store/hooks";
import { useEffect } from "react";

function Notebook(props: NotebookProps) {
    const { notebook } = props;
    const dispatch = useAppDispatch()

    useEffect(() => {
        let cells: CellT[] = notebook.topology.display_order.map((uuid: string) => notebook.topology.cells[uuid]);
        dispatch(init(cells));
    }, [notebook.topology.display_order]);

    const order = notebook.topology.display_order;
    return (
        <div className="min-w-3/4 pt-5">
            <div className="flex justify-between mb-5">
                <h5 className="text-5xl">{notebook.title}</h5>
                <div className="text-xs">
                    <span className="mr-0.5">{notebook.language_info.name}</span>
                    <span>{notebook.language_info.version}</span>
                </div>
            </div>
            <div>
                {order.map((cellUuid: string) => <Cell key={cellUuid} cellUuid={cellUuid} notebookUuid={notebook.uuid} />)}
            </div>
        </div>
    )
}

export default Notebook