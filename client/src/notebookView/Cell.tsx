import { CellBindingProps, CellProps } from "../types";
import { RxPlay } from "react-icons/rx";
import Api from "../api/api";
import Highlight, { defaultProps } from "prism-react-renderer";
import duotoneDark from "prism-react-renderer/themes/duotoneDark";
import { updateBinding } from "../store/cellSlice";
import { useAppSelector, useAppDispatch } from "../store/hooks";

function Cell(props: CellProps) {
    const { cellUuid, notebookUuid } = props;

    const dispatch = useAppDispatch();
    async function handleEval() {
        try {
            const { result } = await Api.evalCell(notebookUuid, cellUuid);
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
            <div className="flex items-end my-2">
                <div>
                    <RxPlay className={"text-slate-100 mr-1 w-8 h-8 hover:bg-slate-700 hover:cursor-pointer" + (cell.isSynced ? " bg-green-500" : " bg-red-600")} onClick={handleEval} />
                </div>

                <div className="w-full">
                    <Highlight {...defaultProps} code={cell.content} language="python" theme={duotoneDark}>
                        {({ className, style, tokens, getLineProps, getTokenProps }) => (
                            <pre className={className + " py-2 px-3 text-xs"} style={style}>
                                {tokens.map((line, i) => (
                                    <div {...getLineProps({ line, key: i })}>
                                        {line.map((token, key) => (
                                            <span {...getTokenProps({ token, key })} />
                                        ))}
                                    </div>
                                ))}
                            </pre>
                        )}
                    </Highlight>
                </div>
            </div>
            <CellBindings cellUuid={cellUuid} />
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