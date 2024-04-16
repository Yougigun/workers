```rust
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
```
