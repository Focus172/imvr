// implementation
// have something that stores the uprocessed args
// and another thing that keeps the state
// where a window is opened the channel is kept until it yeild and when it does
// the coresponding image is opened on it. if none of the channels yeild
// immediatly then just return none

use std::{collections::VecDeque, env, path::PathBuf, time::Duration};

use tokio::sync::oneshot;

use crate::{logic::msg::Msg, prelude::*};

pub struct ArgEventHandler {
    window_opens: Vec<Msg>,
    window_draws: VecDeque<(oneshot::Receiver<u64>, PathBuf)>,
}

impl ArgEventHandler {
    pub fn new() -> Self {
        Self::new_from_list(env::args())
    }

    pub(crate) fn new_from_list(args: impl Iterator<Item = String>) -> Self {
        let mut window_opens = Vec::new();
        let mut window_draws = VecDeque::new();
        for arg in args.skip(1) {
            let (tx, rx) = oneshot::channel();
            window_opens.push(Msg::open(tx));
            window_draws.push_back((rx, arg.into()));
        }

        Self {
            window_opens,
            window_draws,
        }
    }
}

impl Iterator for ArgEventHandler {
    type Item = Msg;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(open) = self.window_opens.pop() {
            return Some(open);
        }

        let Some((mut rx, path)) = self.window_draws.pop_front() else {
            return None;
        };

        log::info!("trying to get id of opened window");

        if let Ok(window) = rx.try_recv() {
            let id = window.into();
            Some(Msg::ShowImage { path, id })
        } else {
            self.window_draws.push_back((rx, path));
            // std::thread::yield_now();
            log::info!("Waiting beacuse nothing is ready yet");
            std::thread::sleep(Duration::from_secs(1));
            self.next()
        }
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

use clap::Parser;

#[derive(Parser)]
pub struct Args {
    #[arg(last = true)]
    pub files: Vec<PathBuf>,
}
