import { CellProps } from "./Block";
import Editor from "react-simple-code-editor";
import { highlight, languages } from "prismjs";
import 'prismjs/components/prism-clike';
import 'prismjs/components/prism-python';
import '../utils/editorTheme.css';

export default function CodeCell(props: CellProps) {
    const { cell, updateCell, evalCell } = props;

    async function handleKeyDown(e: React.KeyboardEvent) {
        if (e.key === 'Enter' && e.shiftKey) {
            await evalCell(cell);
        }
    }

    return (
        <div className="mb-5">
            <Editor
                value={cell.content}
                onValueChange={(v) => updateCell(cell.uuid, v)}
                onKeyDown={handleKeyDown}
                highlight={(code) => highlight(code, languages.python, 'python')}
                padding={10}
                style={{
                    fontFamily: '"Fira code", "Fira Mono", monospace',
                    backgroundColor: '#282c34',
                    fontSize: 12,
                }}
            />
            <div onClick={async () => await evalCell(cell)}>Eval</div>
            <div className="text-gray-500">
                Output
            </div>
        </div>
    )
}
