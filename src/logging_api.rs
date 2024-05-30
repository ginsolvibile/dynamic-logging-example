use std::fs::File;
use std::sync::{Arc, Mutex};
use tonic::{Request, Response, Status};
use tracing::{debug, info, trace, warn};
use tracing_subscriber::{filter::Directive, fmt, prelude::*, reload::Handle, EnvFilter, Registry};

use crate::dynlog::logging_server::Logging;
use crate::dynlog::*;

#[derive(Debug)]
pub struct DynamicLogHandler {
    reload_handle: Handle<EnvFilter, Registry>,
    directives: Arc<Mutex<Vec<Directive>>>,
}

impl DynamicLogHandler {
    pub fn new() -> Self {
        let reload_handle = DynamicLogHandler::init_logging();
        DynamicLogHandler {
            reload_handle,
            directives: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Initialize the logging system with a default filter and JSON formatting
    fn init_logging() -> Handle<EnvFilter, Registry> {
        let filter = DynamicLogHandler::default_filter();
        let file = File::create("app-structured.log").expect("Couldn't create log file");
        let json_layer = fmt::layer().json().with_writer(file);
        let stdout_layer = fmt::layer().with_writer(std::io::stdout);

        let (filter, reload_handle) = tracing_subscriber::reload::Layer::new(filter);
        tracing_subscriber::registry()
            .with(filter)
            .with(json_layer)
            .with(stdout_layer)
            .init();
        reload_handle
    }

    /// Create a default filter with INFO level
    fn default_filter() -> EnvFilter {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
    }

    /// Reload the logging system with the current directives
    fn reload(&self) -> std::result::Result<(), String> {
        info!("Reloading log filters...");
        let mut filter = DynamicLogHandler::default_filter();
        let directives = self.directives.lock().unwrap();
        for dir in directives.iter() {
            filter = filter.add_directive(dir.clone());
        }
        self.reload_handle.reload(filter).map_err(|e| e.to_string())?;
        trace!("Effective directives: {:#?}", directives);
        Ok(())
    }
}

#[tonic::async_trait]
impl Logging for DynamicLogHandler {
    async fn list_directives(&self, request: Request<ListRequest>) -> Result<Response<ListResponse>, Status> {
        debug!("Got list request: {:?}", request);

        let directives = self.directives.lock().unwrap();
        Ok(Response::new(ListResponse {
            directives: directives.iter().map(|d| d.to_string()).collect(),
        }))
    }

    async fn add_directive(
        &self,
        request: Request<AddDirectiveRequest>,
    ) -> Result<Response<AddDirectiveResponse>, Status> {
        debug!("Got add request: {:?}", request);

        let position = request.get_ref().position as usize;
        let position = if request.get_ref().before_pos {
            position
        } else {
            position + 1
        };
        let new_directive = request
            .into_inner()
            .directive
            .parse::<Directive>()
            .map_err(|e| Status::internal(e.to_string()))?;
        {
            let mut directives = self.directives.lock().unwrap();
            if position <= directives.len() {
                debug!("Adding directive at position {}: {}", position, new_directive);
                directives.insert(position, new_directive);
            } else {
                warn!(
                    "Position out of range: {} (we have {} entries)",
                    position,
                    directives.len()
                );
                return Err(Status::invalid_argument("Position out of range"));
            }
        }
        self.reload().map_err(|e| Status::internal(e))?;

        Ok(Response::new(AddDirectiveResponse {}))
    }

    async fn change_directive(
        &self,
        request: Request<ChangeDirectiveRequest>,
    ) -> Result<Response<ChangeDirectiveResponse>, Status> {
        debug!("Got change request: {:?}", request);
        {
            let mut directives = self.directives.lock().unwrap();
            let position = request.get_ref().position as usize;
            if position >= directives.len() {
                warn!(
                    "Position out of range: {} (we have {} entries)",
                    position,
                    directives.len()
                );
                return Err(Status::invalid_argument("Position out of range"));
            }
            let new_directive = request
                .into_inner()
                .directive
                .parse::<Directive>()
                .map_err(|e| Status::internal(e.to_string()))?;
            debug!("Setting directive at position {}: {}", position, new_directive);
            directives[position] = new_directive;
        }
        self.reload().map_err(|e| Status::internal(e))?;

        Ok(Response::new(ChangeDirectiveResponse {}))
    }

    async fn delete_directive(
        &self,
        request: Request<DeleteDirectiveRequest>,
    ) -> Result<Response<DeleteDirectiveResponse>, Status> {
        debug!("Got remove request: {:?}", request);

        let position = request.get_ref().position as usize;
        {
            let mut directives = self.directives.lock().unwrap();
            if position >= directives.len() {
                warn!(
                    "Position out of range: {} (we have {} entries)",
                    position,
                    directives.len()
                );
                return Err(Status::invalid_argument("Position out of range"));
            }
            debug!("Removing directive at position {}", position);
            directives.remove(position);
        }
        self.reload().map_err(|e| Status::internal(e))?;

        Ok(Response::new(DeleteDirectiveResponse {}))
    }
}
