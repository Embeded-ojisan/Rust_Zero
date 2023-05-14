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

