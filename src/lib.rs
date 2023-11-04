use std::{thread::{self, JoinHandle, Thread}, sync::{mpsc, Arc, Mutex}};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct Worker {
    id: usize,
    // 使用vec来存储线程池章所有的thread，这里放着了spawn的实现，
    // 使用了JoinHandle类作为类型参数，并且我们自己的thread不会返回只，所以使用()作为void类型
    thread: Option<JoinHandle<()>>
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();
            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");
                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });
        Worker{id, thread: Some(thread)}
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>
}

impl ThreadPool {
    /// 创建线程池。
    ///
    /// 线程池中线程的数量。
    ///
    /// # Panics
    ///
    /// `new` 函数在 size 为 0 时会 panic。
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        // with_capacity函数是**预先分配**空间!
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            // do something...
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool{workers, sender: Some(sender)}
    }

    pub fn execute<F>(&self, f: F)
    where
        F:FnOnce() + Send + 'static
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap()
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            println!("shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}