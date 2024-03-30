pub use crate::logic::msg::WindowMsg;
pub use crate::render::ctx::GlobalContext;

pub use futures::executor::block_on;
pub use std::fmt;
// pub use tokio::sync::{mpsc, oneshot};

pub use ext::{
    error::{Context, Report, Result, ResultExt},
    log,
    parse::{SomeFrom, SomeInto},
};

#[macro_export]
macro_rules! non_fatal {
    ($err:expr) => {
        $crate::non_fatal!($err, |e| e)
    };

    ($err:expr, $bad:expr) => {{
        let res: std::result::Result<_, _> = $err;
        let fnc: fn(_) -> _ = $bad;
        match res {
            Ok(v) => v,
            Err(e) => {
                let e = fnc(e);
                log::warn!("non-fatal error: \n\r{:#?}", e);
                return Ok(());
            }
        }
    }};
}
pub(crate) use crate::non_fatal;
