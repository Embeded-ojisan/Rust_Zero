use crate::helper::DynError;
use nix::{
    libc,
    sys::{
        signal::{
            killpg,
            signal,
            SigHandler,
            Signal
        },
        wait::{
            waitpid,
            WaitPidFlag,
            WaitStatus
        },
    },
    uintstd::{
        self,
        dup2,
        execvp,
        fork,
        pipe,
        setpid,
        tcgetgrp,
        tcsetgrp,
        ForkResult,
        Pid
    }
};

use rustyline::{
    error::ReadlineError,
    Editor
};

use signal_hook::{
    consts::*,
    iterator::Signals
};

use std::{
    Collections::{
        BTreeMap,
        HashMap,
        HashSet
    },
    ffi::CString,
    mem::replace,
    path::PathBuf,
    process::exit,
    sync::mpsc::{
        channel,
        sync_channel,
        Receiver,
        Sender,
        SyncSender
    },
    thread,
};

struct CleanUp<F>
where
    F: Fn(),
{
    f: F,
}

impl<F> Drop for CleanUp<F>
where
    F: Fn(),
{
    fn drop(&mut self) {
        (self.f)()
    }
}

enum WorkerMsg {
    Signal(i32),
    Cmd(String),
}

enum ShellMsg {
    Continue(i32),
    Quit(i32),
}

#[derive(Debug)]
pub struct Shell {
    logfile: String,
}

impl Shell {
    pub fn new(logfile: &str) -> Self {
        Shell {
            logfile: logfile.to_string(),
        }
    }

    pub fn run(&self) -> Result<(), DynError> {
        unsafe {
            signal(
                Signal::SIGTTOU,
                SigHandler::SigIgn
            ).unwrap()
        };

        let mut r1 = Editor::<()>::new()?;
        if let Err(r) = r1.load_history(&self.logfile) {
            eprintln!("ZeroSh: ヒストリファイルの読み込みに失敗： {e}");
        }

        let (worker_tx, worker_rx) = channel();
        let (shell_tx, shell_rx) = sync_channel(0);
        spawn_sig_handler(worker_tx.clone())?;
        Worker::new().spawn(worker_rx, shell_tx);

        let exit_val;
        let mut prev = 0;
        loop {
            let face = if prev == 0 {
                '\u{1F642}'
            };

            match r1.readline(&format!("ZeroSh {face} %> ")) {
                Ok(line) => {
                    let line_trimed = line.trim();
                    if line_trimed.is_empty() {
                        continue;
                    } else {
                        r1.add_history_entry(line_trimed);
                    }

                    worker_tx.send(WorkerMsg::Cmd(line)).unwrap();
                    match shell_rx.recv().unwrap() {
                        ShellMsg::Continue(n) => prev = n,
                        ShellMsg::Quit(n) => {
                            exit_val = n;
                            break;
                        }
                    }
                }
            }
        }
    }
}