// @ts-ignore-next-line
import { highlight, languages } from "prismjs/components/prism-core";
import { CellBindingProps, CellEditorProps, CellProps } from "../types";
import { RxPlay, RxMagicWand, RxLinkBreak1, RxPencil1 } from "react-icons/rx";
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
    const [showCellToolbar, setShowCellToolbar] = useState<boolean>(false);

    function handleKeyDown(event: KeyboardEvent) {
        if (event.key === "Enter" && event.shiftKey) {
            event.preventDefault();
            props.handleEval(localCode);
        }
    }

    return (
        <div className="flex flex-col hover:cursor-pointer"
            onMouseOver={() => setShowCellToolbar(true)}
            onMouseOut={() => setShowCellToolbar(false)}>
            <div className="flex items-end my-2 relative">
                {showCellToolbar &&
                    <div className="absolute right-0 top-0 z-10">
                        <RxPlay
                            className={"mr-1 w-8 h-8 hover:cursor-pointer" + (cell.isSynced ? " text-green-500" : " text-red-600")}
                            onClick={() => props.handleEval(localCode)} />
                    </div>
                }

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
                            backgroundColor: "#27272a",
                            color: "#d4d4d4",
                            borderRadius: "0.375rem",
                        }}
                    />
                </div>
            </div>
            {showCellToolbar &&
                <div className="flex text-xs justify-center">
                    <div className="flex mr-2">
                        <RxMagicWand className="text-slate-100 mr-1 w-4 h-4 hover:bg-slate-700 hover:cursor-pointer" />
                        <span>Reactive code</span>
                    </div>
                    <div className="flex mr-2">
                        <RxLinkBreak1 className="text-slate-100 mr-1 w-4 h-4 hover:bg-slate-700 hover:cursor-pointer" />
                        <span>Non-reactive code</span>
                    </div>
                    <div className="flex">
                        <RxPencil1 className="text-slate-100 mr-1 w-4 h-4 hover:bg-slate-700 hover:cursor-pointer" />
                        <span>Markdown</span>
                    </div>
                </div>}
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