// implementation
// have something that stores the uprocessed args
// and another thing that keeps the state
// where a window is opened the channel is kept until it yeild and when it does
// the coresponding image is opened on it. if none of the channels yeild
// immediatly then just return none

use std::{collections::VecDeque, env, path::PathBuf};

use oneshot::Receiver;

use crate::prelude::*;

pub struct ArgEventHandler {
    window_opens: VecDeque<Request>,
    window_draws: VecDeque<(Receiver<u64>, PathBuf)>,
}

impl ArgEventHandler {
    pub fn new() -> Self {
        Self::new_from_list(env::args())
    }

    pub(crate) fn new_from_list(args: impl Iterator<Item = String>) -> Self {
        let mut window_opens = VecDeque::new();
        let mut window_draws = VecDeque::new();
        for arg in args.into_iter().skip(1) {
            let (tx, rx) = oneshot::channel();
            window_opens.push_back(Request::OpenWindow { res: tx });
            window_draws.push_back((rx, arg.into()));
        }

        Self {
            window_opens,
            window_draws,
        }
    }

    pub fn next(&mut self) -> Option<Request> {
        let mut window_id = None;
        let idx = self.window_draws.iter().position(|(rx, _)| {
            if let Ok(id) = rx.try_recv() {
                window_id = Some(id);
                return true;
            }
            return false;
        });

        idx.zip(window_id)
            .and_then(|(idx, id)| {
                self.window_draws.remove(idx).and_then(|(_rx, path)| {
                    Some(Request::ShowImage {
                        path,
                        window_id: id,
                    })
                })
            })
            .or_else(|| self.window_opens.pop_front())
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

        assert!(match arg_handle.next() {
            Some(Request::OpenWindow { .. }) => true,
            _ => false,
        });

        // assert!(match arg_handle.next() {
        //     Some(Request::ShowImage { path, .. }) => path == PathBuf::from(path1),
        //     _ => false,
        // });
        // beacuse there is no main thread to respond it will always be None
        assert!(arg_handle.next().is_none())
    }
}
