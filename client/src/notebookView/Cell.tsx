import { CellT, LocalsT } from "../types";
import { RxTriangleRight } from "react-icons/rx";
import { unsyncCell } from "../store/cellSlice";
import { useAppSelector, useAppDispatch } from "../store/hooks";
import { KeyboardEvent, ReactNode, useState } from "react";
import CodeMirror from "@uiw/react-codemirror";
import { python } from "@codemirror/lang-python";
import { atomone } from "@uiw/codemirror-themes-all";
import { WsMessage, WsCmds } from "../api/ws";
import "./Cell.css"

type CellProps = {
    cellUuid: string;
    ws: WebSocket;
}

function Cell(props: CellProps) {
    const { cellUuid, ws } = props;

    async function handleEval(data: string) {
        let wsMessage: WsMessage = {
            cmd: WsCmds.Run,
            cellUuid,
            data: data,
        }
        ws.send(JSON.stringify(wsMessage));
    }

    const cell = useAppSelector((state) => state.cells.mappings[cellUuid]);
    if (!cell) {
        return <div>Loading...</div>
    }

    const out = useAppSelector((state) => state.cells.output[cellUuid]);

    console.log("out", out);
    const hasError = out && out.cmd === WsCmds.Err;
    const hasOutput = out && out.cmd === WsCmds.Res;
    return (
        <div>
            <CellEditor cell={cell} handleEval={handleEval} />
            {hasError && <div className="text-red-500">{out.data}</div>}
            {hasOutput && <CellOutput locals={out.locals} />}
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
                    TEST
                </div>} */}
        </div>
    )
}


export type CellOutputProps = {
    locals: LocalsT;
}

function CellOutput(props: CellOutputProps) {
    const { locals } = props;

    function formatOutput(key: string): ReactNode {
        let { value, local_type } = locals[key as any];
        if (!value || local_type === "Definition") return
        return (<div key={key} className="text-xs max-h-96 overflow-scroll scrollbar-hide">
            <span className="pr-1">{key === "" ? "" : key + ":"}</span>
            <span className="">{value}</span>
        </div>)
    }

    return (
        <div className="flex border-2 border-zinc-800 px-3 py-1 mb-2.5 rounded-md flex-col">
            {Object.keys(locals).map((key: string) => formatOutput(key))}
        </div>
    )
}

export default Cell;