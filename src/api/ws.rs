use crate::{
    api::{state::State, ws_client::WsClient},
    core::kernel_client::KernelClientMsg,
};
use actix_web::{get, web, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde::Deserialize;
use serde_json::json;
use tracing::info;

#[derive(Deserialize)]
struct WsQuery {
    #[serde(rename = "notebookUuid")]
    notebook_uuid: String,
}

#[get("/")]
async fn ws_route(
    req: HttpRequest,
    state: web::Data<State>,
    query: web::Query<WsQuery>,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    let notebook_uuid = query.notebook_uuid.clone();
    info!("Opening websocket for notebook {}", notebook_uuid);

    let notebooks = state.open_notebooks.lock();
    if notebooks.is_err() {
        return Ok(HttpResponse::InternalServerError()
            .json(json!({ "status": "error", "message": "Could not lock notebooks" })));
    }
    let notebooks = notebooks.unwrap();
    let notebook = match notebooks.get(&notebook_uuid) {
        Some(notebook) => notebook,
        None => return Ok(HttpResponse::NotFound().json(json!({ "status": "Notebook not found" }))),
    };

    let kernel_sender = match state.kernel_sender.lock() {
        Ok(kernel_sender) => kernel_sender,
        Err(_) => {
            return Ok(HttpResponse::InternalServerError()
                .json(json!({ "status": "error", "message": "Could not lock kernel sender" })));
        }
    };
    let kernel_sender = kernel_sender.clone();

    let ws_socket = WsClient::new(notebook);
    let (addr, res) = ws::WsResponseBuilder::new(ws_socket, &req, stream).start_with_addr()?;

    let kernel_init_msg = KernelClientMsg::InitWs(notebook.uuid.clone(), addr.clone());
    kernel_sender.send(kernel_init_msg).unwrap();

    Ok(res)
}
