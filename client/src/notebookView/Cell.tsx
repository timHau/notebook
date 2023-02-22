import { CellT } from "../types";
import { RxTriangleRight } from "react-icons/rx";
import Api from "../api/api";
import { updateBinding, unsyncCell } from "../store/cellSlice";
import { useAppSelector, useAppDispatch } from "../store/hooks";
import { KeyboardEvent, ReactNode, useState } from "react";
import CodeMirror from "@uiw/react-codemirror";
import { python } from "@codemirror/lang-python";
import { atomone } from "@uiw/codemirror-themes-all";
import { WsClientT } from "../api/ws";
import "./Cell.css"

type CellProps = {
    cellUuid: string;
    notebookUuid: string;
    wsClient: WsClientT;
}

type NotebookError = {
    message: string;
    status: string;
}

function Cell(props: CellProps) {
    const { cellUuid, notebookUuid } = props;

    const [error, setError] = useState<String>("");

    const dispatch = useAppDispatch();
    async function handleEval(content: string) {
        try {
            const res = await Api.evalCell(notebookUuid, cellUuid, content);
            if (res.status === "error") {
                setError(res.message);
                return;
            }

            dispatch(updateBinding(res.result));
        } catch (error: any) {
            console.log(error);
            setError(error.message);
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
            {error && <div className="text-red-500">{error}</div>}
        </div >
    )
}

export type CellEditorProps = {
    cell: CellT;
    handleEval: (content: string) => void;
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

    const dispatch = useAppDispatch();
    function handleKeyChange(code: string) {
        if (code !== localCode) {
            dispatch(unsyncCell(cell.uuid));
        }
        setLocalCode(code);
    }

    return (
        <div className="flex flex-col hover:cursor-pointer"
            onMouseOver={() => setShowCellToolbar(true)}
            onMouseOut={() => setShowCellToolbar(false)}>
            <div className="flex items-end my-1.5 relative">
                <div className="absolute right-0 top-0 z-10">
                    <RxTriangleRight
                        className={"mr-1 w-6 h-6 hover:cursor-pointer" + (cell.isSynced ? " text-green-500" : " text-gray-600")}
                        onClick={() => props.handleEval(localCode)} />
                </div>

                <div className="w-full">
                    <CodeMirror
                        value={localCode}
                        onChange={handleKeyChange}
                        onKeyDown={handleKeyDown}
                        theme={atomone}
                        extensions={[python()]}
                        style={{
                            fontSize: "0.8rem",
                            borderRadius: "0.25rem",
                        }}
                        basicSetup={{
                            lineNumbers: false,
                            highlightActiveLine: false,
                            highlightActiveLineGutter: false,
                        }}
                    />
                </div>
            </div>
            {/* {showCellToolbar &&
                <div className="flex text-xs justify-center mb-1">
                    <div className="flex items-center mr-1 bg-zinc-800 p-1 rounded-md hover:bg-zinc-700">
                        <RxMagicWand className="text-gray-300 mr-1 w-4 h-4 hover:cursor-pointer" />
                        <span className="ml-0.5 mr-1">reactive code</span>
                    </div>
                    <div className="flex items-center mr-1 bg-zinc-800 p-1 rounded-md hover:bg-zinc-700">
                        <RxLinkBreak1 className="text-gray-300 mr-1 w-4 h-4 hover:cursor-pointer" />
                        <span className="ml-0.5 mr-1">non-reactive code</span>
                    </div>
                    <div className="flex items-center mr-1 bg-zinc-800 p-1 rounded-md hover:bg-zinc-700">
                        <RxPencil1 className="text-gray-300 mr-1 w-4 h-4 hover:cursor-pointer" />
                        <span>markdown</span>
                    </div>
                </div>} */}
        </div>
    )
}


export type CellBindingProps = {
    cellUuid: string;
}

function CellBindings(props: CellBindingProps) {
    const bindings = useAppSelector((state) => state.cells.bindings);
    const binding = bindings[props.cellUuid];
    if (!binding || Object.keys(binding).length === 0) {
        return <span className="hidden"></span>
    }

    function formatBinding(key: string): ReactNode {
        let { value, local_type } = binding[key as any];
        if (!value || local_type === "Definition") return
        return (<div key={key} className="text-xs max-h-96 overflow-scroll scrollbar-hide">
            <span className="pr-1">{key === "" ? "" : key + ":"}</span>
            <span className="">{value}</span>
        </div>)
    }
    return (
        <div className="flex border-2 border-zinc-800 px-3 py-1 mb-2.5 rounded-md flex-col">
            {Object.keys(binding).map((key: string) => formatBinding(key))}
        </div>
    )
}

export default Cell;