use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct Work {
    id: usize,
    handler: Option<thread::JoinHandle<()>>,
}

impl Work {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let handler = thread::spawn(move || loop {
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

        Work {
            id,
            handler: Some(handler),
        }
    }
}

pub struct ThreadPool {
    threads: Vec<Work>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        let (sender, receiver) = mpsc::channel::<Job>();
        let mut threads = Vec::with_capacity(size);

        let receiver = Arc::new(Mutex::new(receiver));
        for id in 0..size {
            threads.push(Work::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            threads,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        let sender_ref = self.sender.as_ref();
        sender_ref.unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for work in &mut self.threads {
            println!("Shutting down thread {}", work.id);
            if let Some(handler) = work.handler.take() {
                handler.join().unwrap();
            }
        }
    }
}
