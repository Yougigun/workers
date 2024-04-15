use std::sync::{
    mpsc::{Receiver, Sender, TryRecvError},
    Arc, Mutex,
};
pub struct Workers {
    pub n_workers: u32,
    pub pool: Vec<Arc<Mutex<Worker>>>,
    // queue for consumers
    pub task_queue: Arc<Mutex<Receiver<Box<dyn Task>>>>,
    pub task_queue_tx: Sender<Box<dyn Task>>,
    // pub task_queue: ,
}

impl Workers {
    pub fn new(n_workers: u32) -> Workers {
        let mut pool = Vec::<Arc<Mutex<Worker>>>::new();
        let (tx, rx) = std::sync::mpsc::channel::<Box<dyn Task>>();
        for i in 0..n_workers {
            let worker = Arc::new(Mutex::new(Worker::new(i)));
            pool.push(Clone::clone(&worker));
        }
        Workers {
            n_workers,
            pool,
            task_queue: Arc::new(Mutex::new(rx)),
            task_queue_tx: tx,
        }
    }

    pub fn run(&mut self) {
        for worker in &self.pool {
            let worker = Arc::clone(worker);
            let queue = self.task_queue.clone();
            std::thread::spawn(move || {
                worker.lock().unwrap().run(queue);
            });
        }
    }

    pub fn add_task(&self, task: Box<dyn Task>) {
        self.task_queue_tx.send(task).unwrap();
    }
}

pub struct Worker {
    pub id: u32,
    pub status: WorkerStatus,
}

impl Worker {
    pub fn new(id: u32) -> Worker {
        Worker {
            id,
            status: WorkerStatus::Idle,
        }
    }
    pub fn run(&mut self, queue: Arc<Mutex<Receiver<Box<dyn Task>>>>) {
        println!("Worker {} is running", self.id);
        loop {
            let task = {
                let lock = queue.lock().unwrap();
                lock.recv()
            };
            match task {
                Ok(task) => {
                    println!("Worker {} execute the task {}", self.id, task.id());
                    self.status = WorkerStatus::Busy;
                    task.execute();
                    self.status = WorkerStatus::Idle;
                }
                // Err(TryRecvError::Empty) => {
                //     // No task in the queue, continue to next iteration
                //     continue;
                // }
                // Err(TryRecvError::Disconnected) => {
                //     // The sender has disconnected, break the loop
                //     break;
                // }
                Err(_) => {
                    break;
                }
            }
        }
        println!("Worker {} closed", self.id);
    }
}

pub enum WorkerStatus {
    Idle,
    Busy,
}

pub trait Task: Send {
    fn execute(&self);
    fn id(&self) -> u32;
}

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use super::*;
    #[test]
    fn test_workers() {
        struct TaskImpl {
            id: u32,
            one_shot_rx: Receiver<i32>,
            one_shot_tx: Sender<i32>,
        }
        impl Task for TaskImpl {
            fn execute(&self) {
                println!("Task {} is running", self.id);
                self.one_shot_tx.send(100).unwrap();
                println!("Task {} is done", self.id);
            }
            fn id(&self) -> u32 {
                self.id
            }
        }
        impl TaskImpl {
            fn get_result(&self) -> i32 {
                self.one_shot_rx.recv().unwrap()
            }
        }
        let mut workers = Workers::new(4);
        workers.run();
        for i in 0..10 {
            let (tx, rx) = std::sync::mpsc::channel::<i32>();
            let task = TaskImpl {
                id: i,
                one_shot_rx: rx,
                one_shot_tx: tx,
            };
            workers.add_task(Box::new(task));
        }
        sleep(Duration::from_secs(5));
    }
}
