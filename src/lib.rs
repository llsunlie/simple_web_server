use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}
impl Worker {
    // 创建一个新的 Worker，id 表示线程编号，receiver 是 Job 的接收者
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        // 创建一个新线程，不断从接收者中获取 Job 并执行
        let thread = thread::spawn(move || loop {
            // 从接收者中获取 Job，如果接收者已经被关闭则退出循环
            let message = receiver.lock().unwrap().recv();
            match message {
                Ok(job) => {
                    println!("Worker {} got a job. executing.", id);
                    job();
                }
                Err(_) => {
                    println!("Worker {} disconnected; shutting down.", id);
                    break;
                }
            }
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}
impl ThreadPool {
    // 创建一个新的线程池，size 表示线程池中的工作线程数目
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        // 创建一个 Job 的发送者和接收者
        let (sender, receiver) = mpsc::channel();
        // 将接收者包装成一个 Arc<Mutex<T>>，这样可以在多个线程间共享
        let receiver = Arc::new(Mutex::new(receiver));

        // 创建一定数量的 Worker，并将它们加入线程池
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    // 提交一个新的 Job 到线程池中执行
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        // 将 Job 封装成 Box，并发送到 Job 的发送者中
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

// 线程池的析构函数
impl Drop for ThreadPool {
    fn drop(&mut self) {
        // 关闭 Job 的发送者
        drop(self.sender.take());

        // 逐个关闭 Worker 中的线程，并等待它们结束
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
