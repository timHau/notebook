import { Cell } from "./types/cell"
import ReactMarkdown from "react-markdown"
import { CellTypes } from "./types/cellTypes"
import './Block.css'

interface CellProps {
    cell: Cell
}

function BlockCell(props: CellProps) {
    const { cell } = props
    if (cell.cellType === CellTypes.Markdown) {
        return (
            <div>
                <ReactMarkdown className="markdown" >{cell.content}</ReactMarkdown>
            </div>
        )
    } else {
        return (
            <div>
                <h3>Code Cell</h3>
            </div>
        )
    }
}

interface BlockProps {
    cell: Cell
}

function Block(props: BlockProps) {
    return (
        <div>
            <BlockCell cell={props.cell} />
            <div>
                <span>Add Markdown</span>
                <span>Add Code</span>
            </div>
        </div>
    )
}

export default Block