//! The database(s) defined in `zc` are `!Sync` (by design), so this module is
//! responsible for the items necessary to keep the database in its own thread
//! to which updates can be sent.

mod change;

pub use change::Change;

use std::thread::{self, JoinHandle};

use crossbeam::channel::{bounded, select, unbounded, Receiver, Sender};
use salsa::{ParallelDatabase, Snapshot};
use zc::inputs::Inputs;
use zc::ZcDatabase;

/// A handle to the database to which updates can be sent and snapshots
/// requested.
#[derive(Debug)]
pub struct Database {
    handle: Option<JoinHandle<()>>,
    changes: Sender<Change>,

    request_snapshot: Sender<()>,
    get_snapshot: Receiver<Snapshot<ZcDatabase>>,
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        } else {
            unreachable!()
        }
    }
}

fn spawn_zcdb(
    change: Receiver<Change>,
    snapshot_req: Receiver<()>,
    snapshot_send: Sender<Snapshot<ZcDatabase>>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut db = ZcDatabase::default();
        loop {
            select! {
                recv(change) -> msg => if let Ok(change) = msg {
                    match change {
                        Change::NewContent { at, data } => {
                            db.set_input_file(at, data);
                        }

                        Change::SourceData { id, source } => {
                            db.set_source(id, source);
                        }
                    }
                } else {
                    break;
                },

                recv(snapshot_req) -> msg => if let Ok(()) = msg {
                    let snapshot = db.snapshot();
                    snapshot_send.send(snapshot).unwrap();
                } else {
                    break;
                },
            }
        }
    })
}

impl Database {
    /// Create a new, default database.
    pub fn new() -> Self {
        let (send, recv) = unbounded::<Change>();
        let (request_snapshot, rs_recv) = bounded::<()>(0);
        let (gs_send, get_snapshot) = bounded::<Snapshot<ZcDatabase>>(0);

        let handle = Some(spawn_zcdb(recv, rs_recv, gs_send));

        Self {
            handle,
            changes: send,

            request_snapshot,
            get_snapshot,
        }
    }

    /// Update the compiler with the given change.
    pub fn update(&self, change: Change) {
        self.changes
            .send(change)
            .expect("unable to send stuff uh oh");
    }

    /// Get a snapshot of the current database.
    pub fn get_snapshot(&self) -> Snapshot<ZcDatabase> {
        self.request_snapshot.send(()).unwrap();
        self.get_snapshot.recv().unwrap()
    }
}
