import { CellProps } from "./types";
import { RxPlay } from "react-icons/rx";
import Api from "../utils/api";
import Highlight, { defaultProps } from "prism-react-renderer";
import duotoneDark from "prism-react-renderer/themes/duotoneDark";
import { increment, decrement } from "../store/counterSlice";
import { useAppDispatch, useAppSelector } from "../store/hooks";

function Cell(props: CellProps) {
    const { cell, notebookUuid } = props;

    async function handleEval() {
        const res = await Api.evalCell(notebookUuid, cell.uuid);
        console.log(res);
    }

    const count = useAppSelector((state) => state.counter.value)
    const dispatch = useAppDispatch()

    return (
        <div className="flex items-end my-2">
            <RxPlay className="text-slate-100 mr-1 bg-slate-600 w-8 h-8 hover:bg-slate-700 hover:cursor-pointer" onClick={handleEval} />

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
            <div>
                Output: {count}
                <button onClick={() => dispatch(increment())}>+</button>
                <button onClick={() => dispatch(decrement())}>-</button>
            </div>
        </div>
    )
}

export default Cell;