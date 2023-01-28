import { useState } from "react"
import ReactMarkdown from "react-markdown"
import { CellTypes } from "../types/cellTypes"
import { Cell } from "../types/cell"
import { NotebookData } from "../types/notebook"
import MDEditor from "@uiw/react-md-editor"
import './Block.css'

interface CellProps {
    cell: Cell
    notebook: NotebookData
    updateCell: (uuid: string, content: string) => Promise<void>
    addCell(cellType: CellTypes): Promise<void>
}

function MarkdownCell(props: CellProps) {
    const [editing, setEditing] = useState(false)
    const [content, setContent] = useState(props.cell.content)

    const { cell, notebook, updateCell } = props;
    if (editing) {
        return (
            <div>
                <MDEditor
                    value={content}
                    onChange={(v) => setContent(v || "")}
                    preview="edit"
                />
                <div onClick={async () => {
                    await updateCell(cell.uuid, content)
                    setEditing(!editing)
                }}>Edit</div>
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

function Block(props: CellProps) {
    const { addCell } = props;

    function handleNewCell() {
        console.log("add cell")
    }

    return (
        <div className="flex">
            <div className="relative">
                <div className="border-r-2 h-full border-gray-300 dark:border-slate-600 mr-2"></div>
                <div className="absolute -left-1" onClick={handleNewCell}>+</div>
            </div>
            <div className="mb-1">
                {
                    (props.cell.cell_type === CellTypes.Markdown) ? (
                        <MarkdownCell {...props} />) :
                        (
                            <div>
                                <h3>Code Cell</h3>
                            </div>
                        )
                }
            </div>
            {/* <div className="flex justify-around text-xs">
                <span onClick={() => addCell(CellTypes.Markdown)} className="mr-3">Add Markdown</span>
                <span className="mr-3">Add Non reactive Code</span>
                <span>Add reactive Code</span>
            </div> */}
        </div>
    )
}

export default Block