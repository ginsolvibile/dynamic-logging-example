use tonic::{Request, Response, Status};
use tracing::{debug, error, trace, warn};

use crate::dynlog::fibonacci_server::Fibonacci;
use crate::dynlog::*;

#[derive(Debug, Default)]
pub struct ServiceRunner {}

impl ServiceRunner {
    fn fibonacci(val: u32) -> u64 {
        match val {
            0 => 0,
            1 => 1,
            _ => {
                let mut a = 0;
                let mut b = 1;
                for _ in 2..=val {
                    let c = a + b;
                    a = b;
                    b = c;
                    trace!(
                        fun = "fibonacci",
                        val = val,
                        "Fibonacci computation is running: {}",
                        c
                    );
                }
                b
            }
        }
    }
}

#[tonic::async_trait]
impl Fibonacci for ServiceRunner {
    async fn run_service(
        &self,
        request: Request<RunServiceRequest>,
    ) -> Result<Response<RunServiceResponse>, Status> {
        debug!("Got run request: {:?}", request);

        let user = request.get_ref().user_id;
        let val = request.get_ref().value;
        debug!(uid = user, "Running service for user, val is {}", val);

        match val {
            v if v < 0 => {
                error!(uid = user, "User requested a negative value: {}", v);
                return Err(Status::invalid_argument("Negative values not allowed"));
            }
            v if v > 47 && v <= 93 => {
                warn!(
                    uid = user,
                    "User requested a value over 47: {}, which would not fit on a u32", v
                );
            }
            v if v > 93 => {
                error!(
                    uid = user,
                    "User requested a value over 93: {}, not supported on this system", v
                );
                return Err(Status::invalid_argument("Values over 93 are not supported"));
            }
            _ => {}
        }

        let request_span = tracing::trace_span!(
            "Starting computation",
            uid = user,
            val = val,
        );
        let _ = request_span.enter();

        let result = ServiceRunner::fibonacci(val as u32);
        debug!(uid = user, "Result for {} is {}", val, result);

        Ok(Response::new(RunServiceResponse { result }))
    }
}
