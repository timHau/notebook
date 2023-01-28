import MDEditor from "@uiw/react-md-editor";
import { useEffect, useRef, useState } from "react";
import ReactMarkdown from "react-markdown";
import { CellProps } from "./Block";


export default function MarkdownCell(props: CellProps) {
    const editorRef = useRef(null as any);
    const { cell, updateCell } = props;
    const [editing, setEditing] = useState(false)
    const [content, setContent] = useState(cell.content)

    useEffect(() => {
        async function handleClickOutside(event: MouseEvent) {
            if (editorRef.current && !editorRef.current.contains(event.target)) {
                await updateCell(cell.uuid, content)
                setEditing(false)
            }
        }
        document.addEventListener("mousedown", handleClickOutside);
        return () => {
            document.removeEventListener("mousedown", handleClickOutside);
        }
    }, [editorRef, content]);


    if (editing) {
        return (
            <div ref={editorRef}>
                <MDEditor
                    value={content}
                    onChange={(v) => setContent(v || "")}
                    preview="edit"
                    className="md-editor"
                />
            </div>
        )
    }

    return (
        <div>
            <ReactMarkdown className="markdown">{content}</ReactMarkdown>
            <div onClick={() => setEditing(!editing)}>Edit</div>
        </div>
    )
}