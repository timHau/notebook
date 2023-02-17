import Cell from "./Cell";
import { CellT as CellT, NotebookProps } from "./types"

function Notebook(props: NotebookProps) {
    const { notebook } = props;
    let cells: CellT[] = notebook.topology.display_order.map((uuid: string) => notebook.topology.cells[uuid]);

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
                {cells.map((cell: CellT, i) => <Cell key={i} cell={cell} notebookUuid={notebook.uuid} />)}
            </div>
        </div>
    )
}

export default Notebook