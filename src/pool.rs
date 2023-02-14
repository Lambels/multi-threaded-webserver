use std::{
    sync::mpsc::{self, TrySendError},
    thread,
};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    work_buf: Vec<Worker>,
    unused_x: usize,
}

impl ThreadPool {
    pub fn new(n: usize) -> Self {
        assert!(n > 0);

        let mut work_buf = Vec::with_capacity(n);

        for i in 0..n {
            work_buf.push(Worker::new(i));
        }

        Self {
            work_buf,
            unused_x: 0,
        }
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        
    }

    fn try_active<F>(&self, f: F) -> bool {}
    
    fn first_sleeping<F>(&mut self, f: F) {}
}

enum ThreadState {
    Sleeping,
    Active(mpsc::SyncSender<Work>, thread::JoinHandle<()>),
    Closed,
}

enum Work {
    Run(Job),
    Done,
}

struct Worker {
    thread: ThreadState,
    id: usize,
}

impl Worker {
    pub fn new(id: usize) -> Self {
        Self {
            thread: ThreadState::Sleeping,
            id,
        }
    }

    pub fn bind<F>(&mut self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        match &mut self.thread {
            ThreadState::Active(tx, _) => tx.send(Work::Run(Box::new(f))).unwrap(),
            _ => {
                let (tx, rx) = mpsc::sync_channel(0);

                let id = self.id;
                let thread_handle = thread::spawn(move || loop {
                    let work: Work = rx.recv().unwrap();

                    println!("Worker: {} got work", id);

                    if let Work::Run(f) = work {
                        f();
                    } else {
                        println!("Worker: {} exiting", id);
                        return;
                    }
                });

                tx.send(Work::Run(Box::new(f))).unwrap();

                self.thread = ThreadState::Active(tx, thread_handle);
            }
        }
    }

    pub fn bind_soft<F>(&self, f: F) -> Result<(), TrySendError<Work>>
    where
        F: FnOnce() + Send + 'static,
    {
        match &self.thread {
            ThreadState::Active(tx, _) => tx.try_send(Work::Run(Box::new(f))),
            ThreadState::Sleeping => Err(TrySendError::Full(Work::Run(Box::new(f)))),
            ThreadState::Closed => Err(TrySendError::Disconnected(Work::Run(Box::new(f)))),
        }
    }

    pub fn close(&mut self) {
        let state = std::mem::replace(&mut self.thread, ThreadState::Closed);

        if let ThreadState::Active(tx, handle) = state {
            tx.send(Work::Done).unwrap();
            handle.join().unwrap();
        }

        self.thread = ThreadState::Closed
    }
}
