import { CellProps } from "./Block";
import Editor from "react-simple-code-editor";
import { highlight, languages } from "prismjs";
import 'prismjs/components/prism-clike';
import 'prismjs/components/prism-python';
import 'prismjs/themes/prism.css';

export default function CodeCell(props: CellProps) {
    const { cell, updateCell } = props;
    return (
        <Editor
            value={cell.content}
            onValueChange={(v) => updateCell(cell.uuid, v)}
            highlight={(code) => highlight(code, languages.python, 'python')}
            padding={10}
            style={{
                fontFamily: '"Fira code", "Fira Mono", monospace',
                fontSize: 12,
            }}
        />
    )
}
