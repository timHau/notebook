import { CellProps } from "./types";
import { RxPlay } from "react-icons/rx";
import Api from "../api";

function Cell(props: CellProps) {
    const { cell, notebookUuid } = props;

    async function handleEval() {
        const res = await Api.evalCell(notebookUuid, cell.uuid);
    }

    return (
        <div className="flex items-center">
            <RxPlay className="text-slate-100 mr-1 bg-slate-600 w-8 h-8 hover:bg-slate-700 hover:cursor-pointer" onClick={handleEval} />
            <div className="w-full bg-slate-600 my-3 px-2 py-1">
                <code>
                    {cell.content}
                </code>
            </div>
        </div>
    )
}

export default Cell;