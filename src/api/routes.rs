use super::{index::index, reorder::reorder_cells, ws::ws_route};
use actix_web::web;

pub fn notebook_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(index);
    cfg.service(reorder_cells);
}

pub fn ws_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(ws_route);
}
