// implementation
// have something that stores the uprocessed args
// and another thing that keeps the state
// where a window is opened the channel is kept until it yeild and when it does
// the coresponding image is opened on it. if none of the channels yeild
// immediatly then just return none

use std::{collections::VecDeque, env, path::PathBuf};


use tokio::sync::oneshot;

use crate::prelude::*;

pub struct ArgEventHandler {
    window_opens: VecDeque<Request>,
    window_draws: VecDeque<(oneshot::Receiver<u64>, PathBuf)>,
}

impl ArgEventHandler {
    pub fn new() -> Self {
        Self::new_from_list(env::args())
    }

    pub(crate) fn new_from_list(args: impl Iterator<Item = String>) -> Self {
        let mut window_opens = VecDeque::new();
        let mut window_draws = VecDeque::new();
        for arg in args.skip(1) {
            let (tx, rx) = oneshot::channel();
            window_opens.push_back(Request::OpenWindow { res: tx });
            window_draws.push_back((rx, arg.into()));
        }

        Self {
            window_opens,
            window_draws,
        }
    }
}

impl Iterator for ArgEventHandler {
    type Item = Result<Request, ()>;

    fn next(&mut self) -> Option<Result<Request, ()>> {
        let Some((rx, _)) = self.window_draws.front_mut() else {
            return Some(Err(()));
        };

        let Ok(window_id) = rx.try_recv() else {
            // TODO: this returning none can cause this event send to return None
            // before when it is expected
            return self.window_opens.pop_front().map(|r| Ok(r));
        };

        // this is a safe unwrap beacuse front returned some above
        let (_, path) = self.window_draws.pop_front().unwrap();

        Some(Ok(Request::ShowImage { path, window_id }))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_args() {
        let path1 = String::from("path1");
        let args = ["prog_name".into(), path1.clone()].into_iter();
        let mut arg_handle = ArgEventHandler::new_from_list(args);

        // assert!(match arg_handle.next() {
        //     Some(Request::OpenWindow { .. }) => true,
        //     _ => false,
        // });

        // assert!(match arg_handle.next() {
        //     Some(Request::ShowImage { path, .. }) => path == PathBuf::from(path1),
        //     _ => false,
        // });
        // beacuse there is no main thread to respond it will always be None
        assert!(arg_handle.next().is_none())
    }
}
