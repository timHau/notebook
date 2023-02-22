import Cell from "./Cell";
import { init } from "../store/cellSlice";
import { CellT } from "../types"
import { useAppDispatch } from "../store/hooks";
import { useEffect, useState } from "react";
import { WsClientT } from "../api/ws";
import { DragDropContext, Droppable, Draggable } from "react-beautiful-dnd";
import Api from "../api/api";

export type NotebookProps = {
    notebook: any;
    wsClient: WsClientT;
}

function Notebook(props: NotebookProps) {
    const { notebook, wsClient } = props;
    const [order, setOrder] = useState<string[]>(notebook.topology.display_order);

    const dispatch = useAppDispatch()
    useEffect(() => {
        let cells: CellT[] = notebook.topology.display_order.map((uuid: string) => notebook.topology.cells[uuid]);
        dispatch(init(cells));
    }, [notebook.topology.display_order]);

    async function handleDragEnd(result: any) {
        if (!result.destination) {
            return;
        }

        const newOrder = Array.from(order);
        const [removed] = newOrder.splice(result.source.index, 1);
        newOrder.splice(result.destination.index, 0, removed);

        try {
            const res = await Api.reorderCells(notebook.uuid, newOrder);
            console.log(res);
        } catch (error: any) {
            console.log(error);
        }

        setOrder(newOrder);
    }

    return (
        <div className="min-w-3/4 max-w-6xl pt-5">
            <div className="flex justify-between mb-5">
                <h5 className="text-5xl">{notebook.title}</h5>
                <div className="text-xs">
                    <span className="mr-0.5">{notebook.language_info.name}</span>
                    <span>{notebook.language_info.version}</span>
                </div>
            </div>
            <DragDropContext onDragEnd={handleDragEnd}>
                <Droppable droppableId="cell">
                    {(provided, snapshot) => (
                        <div {...provided.droppableProps} id="cell" ref={provided.innerRef}>
                            {order.map((cellUuid: string, i: number) => (
                                <Draggable key={cellUuid} draggableId={cellUuid} index={i} >
                                    {(provided, snapshot) => (
                                        <div
                                            {...provided.draggableProps}
                                            {...provided.dragHandleProps}
                                            ref={provided.innerRef}
                                        >
                                            < Cell
                                                key={cellUuid}
                                                cellUuid={cellUuid}
                                                notebookUuid={notebook.uuid}
                                                wsClient={wsClient}
                                            />
                                        </div>
                                    )}
                                </Draggable>
                            ))}
                            {provided.placeholder}
                        </div>
                    )}
                </Droppable>
            </DragDropContext>
        </div >
    )
}

export default Notebook