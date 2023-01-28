import { CellTypes } from "../types/cellTypes"
import { Cell } from "../types/cell"
import MarkdownCell from "./MarkdownCell";
import CodeCell from "./CodeCell";

export interface CellProps {
    cell: Cell
    updateCell: (uuid: string, content: string) => Promise<void>
    evalCell: (cell: Cell) => Promise<void>
    addCell(cellType: CellTypes): Promise<void>
}


function Block(props: CellProps) {
    const { addCell, cell } = props;

    function handleNewCell() {
        console.log("add cell")
    }

    return (
        <div className="">
            {
                (cell.cell_type === CellTypes.Markdown) ? (
                    <MarkdownCell {...props} />) :
                    (<CodeCell {...props} />)
            }
        </div>
    )
}

export default Block