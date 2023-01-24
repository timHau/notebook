import { Cell } from "./types/cell"
import { useState } from "react"
import ReactMarkdown from "react-markdown"
import { CellTypes } from "./types/cellTypes"
import MDEditor from "@uiw/react-md-editor"
import './Block.css'

interface CellProps {
    cell: Cell
}

function MarkdownCell(props: CellProps) {
    const [editing, setEditing] = useState(false)
    const [content, setContent] = useState(props.cell.content)

    if (editing) {
        return (
            <div>
                <MDEditor
                    value={content}
                    onChange={(v) => setContent(v || "")}
                    preview="edit"
                />
                <div onClick={() => setEditing(!editing)}>Edit</div>
            </div>
        )
    }

    return (
        <div>
            <ReactMarkdown className="markdown" >{content}</ReactMarkdown>
            <div onClick={() => setEditing(!editing)}>Edit</div>
        </div>
    )
}


function BlockCell(props: CellProps) {
    if (props.cell.cellType === CellTypes.Markdown) {
        return <MarkdownCell cell={props.cell} />
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
            <div className="mb-1">
                <BlockCell cell={props.cell} />
            </div>
            <div className="">
                <span className="mr-3">Add Markdown</span>
                <span>Add Code</span>
            </div>
        </div>
    )
}

export default Block