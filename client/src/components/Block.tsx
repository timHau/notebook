import { CellTypes } from "../types/cellTypes"
import { Cell } from "../types/cell"
import './Block.css'
import MarkdownCell from "./MarkdownCell";
import CodeCell from "./CodeCell";

export interface CellProps {
    cell: Cell
    updateCell: (uuid: string, content: string) => Promise<void>
    addCell(cellType: CellTypes): Promise<void>
}


function Block(props: CellProps) {
    const { addCell } = props;

    function handleNewCell() {
        console.log("add cell")
    }

    return (
        <div className="mb-3 w-full">
            {
                (props.cell.cell_type === CellTypes.Markdown) ? (
                    <MarkdownCell {...props} />) :
                    (<CodeCell {...props} />)
            }
        </div>
    )
}

export default Block