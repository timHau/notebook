import { CellProps } from "./types";

function Cell(props: CellProps) {
    const { cell } = props;
    return (
        <div>
            <code>
                {cell.content}
            </code>
        </div>
    )
}

export default Cell;