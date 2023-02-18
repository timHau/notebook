// @ts-ignore-next-line
import { highlight, languages } from "prismjs/components/prism-core";
import { CellBindingProps, CellEditorProps, CellProps } from "../types";
import { RxPlay } from "react-icons/rx";
import Api from "../api/api";
import { updateBinding } from "../store/cellSlice";
import { useAppSelector, useAppDispatch } from "../store/hooks";
import Editor from "react-simple-code-editor";
import { KeyboardEvent, useState } from "react";

function Cell(props: CellProps) {
    const { cellUuid, notebookUuid } = props;

    const dispatch = useAppDispatch();
    async function handleEval(content: string) {
        try {
            const { result } = await Api.evalCell(notebookUuid, cellUuid, content);
            dispatch(updateBinding(result));
        } catch (error) {
            console.error(error);
        }
    }

    const cell = useAppSelector((state) => state.cells.mappings[cellUuid]);
    if (!cell) {
        return <div>Loading...</div>
    }

    console.log(languages)

    return (
        <div>
            <CellEditor cell={cell} handleEval={handleEval} />
            <CellBindings cellUuid={cellUuid} />
        </div >
    )
}

function CellEditor(props: CellEditorProps) {
    const { cell } = props;
    const [localCode, setLocalCode] = useState<string>(cell.content);

    function handleKeyDown(event: KeyboardEvent) {
        if (event.key === "Enter" && event.shiftKey) {
            event.preventDefault();
            props.handleEval(localCode);
        }
    }

    return (
        <div className="flex items-end my-2">
            <div>
                <RxPlay
                    className={"text-slate-100 mr-1 w-8 h-8 hover:bg-slate-700 hover:cursor-pointer" + (cell.isSynced ? " bg-green-500" : " bg-red-600")}
                    onClick={() => props.handleEval(localCode)} />
            </div>

            <div className="w-full">
                <Editor
                    value={localCode}
                    onValueChange={(code) => setLocalCode(code)}
                    onKeyDown={handleKeyDown}
                    highlight={(code) => highlight(code, languages.python, 'python')}
                    padding={10}
                    style={{
                        fontFamily: '"Fira code", "Fira Mono", monospace',
                        fontSize: 12,
                        backgroundColor: "#1e1e1e",
                        color: "#d4d4d4",
                    }}
                />
            </div>
        </div>
    )
}

function CellBindings(props: CellBindingProps) {
    const bindings = useAppSelector((state) => state.cells.bindings);
    const binding = bindings[props.cellUuid];
    if (!binding) {
        return <span className="hidden"></span>
    }

    return (
        <div className="flex">
            {Object.keys(binding).map((key: string) => (
                <div key={key} className="mr-2">
                    <span className="text-xs text-slate-100 pr-1">{key === "RETURN" ? "" : key}</span>
                    <span className="text-xs text-slate-100">{binding[key as any]}</span>
                </div>
            ))}
        </div>
    )
}

export default Cell;