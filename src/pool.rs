use std::{
    sync::{
        mpsc::{self, SyncSender},
        Arc, Mutex, Weak,
    },
    thread,
};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    tx: Option<SyncSender<Job>>,
    work_buf: Vec<Worker>,
    unused_x: usize,
}

impl ThreadPool {
    pub fn new(n: usize) -> Self {
        assert!(n > 0);

        let (tx, rx) = mpsc::sync_channel(0);
        let rx = Arc::new(Mutex::new(rx));

        let mut work_buf = Vec::with_capacity(n);

        for i in 0..n {
            work_buf.push(Worker::new(i, Arc::downgrade(&rx)));
        }
        work_buf[0].awake(Box::new(|| {}));

        Self {
            tx: Some(tx),
            work_buf,
            unused_x: 1,
        }
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        let sender = self.tx.as_ref().unwrap();

        // all threads are started, add the job to the recv queue.
        if self.unused_x == self.work_buf.capacity() {
            sender.send(job).unwrap();
            return;
        }

        // try sending the job to any thread waiting for work.
        match sender.try_send(job) {
            Ok(_) => (), // a thread recieved it.
            Err(job) => {
                // no thread was ready for work, spin up a worker to handle our job
                // concurrently.
                if let mpsc::TrySendError::Full(job) = job {
                    self.work_buf[self.unused_x].awake(job);
                    self.unused_x += 1;
                }
            }
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.tx.take());

        for worker in self.work_buf.iter_mut() {
            if let Some(handler) = worker.thread.take() {
                handler.join().unwrap();
            }
        }
    }
}

struct Worker {
    recv: Option<Weak<Mutex<mpsc::Receiver<Job>>>>,
    thread: Option<thread::JoinHandle<()>>,
    id: usize,
}

impl Worker {
    fn new(id: usize, recv: Weak<Mutex<mpsc::Receiver<Job>>>) -> Self {
        Self {
            recv: Some(recv),
            id,
            thread: None,
        }
    }

    fn awake(&mut self, f: Job) {
        let rx = match self.recv.take() {
            Some(chan) => chan.upgrade().unwrap(),
            None => panic!("tryied to awoked live thread"),
        };

        let id = self.id;

        let thread = thread::spawn(move || {
            println!("Awakening worker {} and running job", id);
            f();

            loop {
                let work = rx.lock().unwrap().recv();

                match work {
                    Ok(f) => {
                        println!("Worker {} running job", id);

                        f();
                    }
                    Err(_) => {
                        println!("Worker {} exiting", id);
                        break;
                    }
                };
            }
        });

        self.thread = Some(thread);
    }
}
