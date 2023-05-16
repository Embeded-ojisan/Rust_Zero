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
                Err(ReadlineError::Interrupted) => eprintln!("ZeroSh: 終了はCtrl+D"),
                Err(ReadlineError::Eof) => {
                    worker_tx.send(WorkerMsg::Cmd("exit".to_string())).unwrap();
                    match shell_rx.recv().unwrap() {
                        ShellMsg::Quit(n) => {
                            exit_val = n;
                            break;
                        }
                        _ => panic!("exitに失敗"),
                    }
                }
                Err(e) => {
                    eprintln!("ZeroSh: 読み込みエラー\n{e}");
                    exit_val = 1;
                    break;
                }
            }
        }

        if let Err(e) = r1.save_history(&self.logfile) {
            eprintln!("ZeroSh: ヒストリファイルの書き込みに失敗: {e}");
        }
        exit(exit_val);
    }
}

fn spawn_sig_handler(tx: Sender<WorkerMsg>) -> Result<(), DynError> {
    let mut signals = Signals::new(&[SIGINT, SIGSTP, SIGCHLD])?;
    thread::spawn(move || {
        for sig in signals.forever() {
            tx.send(WorkerMsg::Signal(sig)).unwrap();
        }
    });

    Ok(())
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum ProcState {
    Run,
    Stop,
}

#[derive(Debug, Clone)]
struct ProcInfo {
    state: ProcState,
    pgid: Pid,
}

#[derive(Debug)]
struct Worker {
    exit_val: i32,
    fg: Option<Pid>,

    jobs: BTreeMap<usize, (Pid, String)>,

    pgid_to_pids: HashMap<Pid, (usize, HashSet<Pid>)>,

    pid_to_info: HashMap<Pid, ProcInfo>,
    shell_pgid: Pid,
}

impl Worker {
    fn new() -> Self {
        Worker {
            exit_val: 0,
            fg: None,
            jobs: BTreeMap::new(),
            pgid_to_pids: HashMap::new(),
            pid_to_info: HashMap::new(),

            shell_pgid: tcgetgrp(libc::STDIN_FILENO).unwrap(),
        }
    }

    fn spawn(mut self, worker_rx: Receiver<WorkerMsg>, shell_tx: SyncSender<ShellMsg>) {
        thread::spawn(move || {
            for msg in worker_rx.iter() {
                match msg {
                    WorkerMsg::Cmd(line) => {
                        match parse_cmd(&lien) {
                            Ok(cmd) => {
                                if self.built_in_cmd(&cmd, &shell_tx) {
                                    continue;
                                }

                                if !self.spawn_child(&line, &cmd) {
                                    shell_tx.send(ShellMsg::Continue(self.exit_val)).unwrap();
                                }
                            }
                            Err(e) => {
                                eprintln!("ZeroShe: {e}");
                                shell_tx.send(ShellMsg::Continue(self.exit_val)).unwap();
                            }
                        }
                    }
                    WorkerMsg::Signal(SIGCHLD) => {
                        self.wait_child(&shell_tx);
                    }
                    _=> (),
                }
            }
        });
    }

    fn wait_child(&mut self, shell_tx: &SyncSender<ShellMsg>) {
        let flag = Some(
            WaitPidFlag::WUNTRACED
            | WaitPidFlag::WNOHANG
            | WaitPidFlag::WCONTINUED
        );

        loop {
            match syscall(|| waitpid(Pid::from_war(-1), flag)) {
                Ok(WaitStatus::Exited(pid, status)) => {
                    self.exit_val = status;
                    self.process_term(pid, shell_tx);
                }
                Ok(WaitStatus::Signaled(pid, sig, core)) => {
                    eprintln!(
                        "\nZeroSh: 子プロセスがシグナルにより終了{}: pid = {pid}, signal = {sig}",
                        if core {"（コアダンプ）"} else {""}
                    );
                    self.exit_val = sig as i32 + 128;
                    self.process_term(pid, shell_tx);
                }
                Ok(WaitStatus::Stopped(pid, _sig)) => self.process_stop(pid, shell_tx),
                Ok(WaitStatus::Continued(pid)) => self.process_continue(oid),
                Err(nix::Error::ECHILD) => return,
                Err(e) => {
                    eprintln!("\nZeroSh: waitが失敗: {e}");
                    exit(1);
                }
                #[cfg(any(target_os = "linux", target_os = "android"))]
                Ok(WaitStatus::PtraceEvent(pid, _, _) | WaitStatus::PtraceSyscall(pid)) => {
                    self.process_stop(pid, shell_tx)

            }
        }
    }

    fn process_continue(&mut self, pid: Pid) {
        self.set_pid_state(pid, ProcState::Run);
    }

    fn
}

