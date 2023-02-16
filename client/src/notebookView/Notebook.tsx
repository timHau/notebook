import Cell from "./Cell";
import { Cell as CellT, NotebookProps } from "./types"

function Notebook(props: NotebookProps) {
    const { notebook } = props;
    let cells: CellT[] = notebook.topology.display_order.map((uuid: string) => notebook.topology.cells[uuid]);
    return (
        <div>
            <h5>Notebook {notebook.uuid}</h5>
            <div>
                {cells.map((cell: CellT, i) => <Cell key={i} cell={cell} />)}
            </div>
        </div>
    )
}

export default Notebook