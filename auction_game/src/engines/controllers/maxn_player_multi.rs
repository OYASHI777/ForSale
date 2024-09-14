use crossbeam::scope;
use crossbeam_deque::{Injector, Stealer, Worker};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Relaxed, SeqCst};
use std::sync::{mpsc, Arc};
use std::thread;

enum Task<T, P> {
    Traverse(T),
    Propagate(P),
}

struct WorkStealingMaxN<T, P> {
    num_threads: usize,
    traverser_vec: Vec<Worker<T>>,
    propagator_vec: Vec<Worker<P>>,
    stealers: Vec<Vec<Stealer<T>>>,
    abort_flag: Arc<AtomicBool>,
}

impl<T: Send + 'static, P: Send + 'static> WorkStealingMaxN<T, P> {
    pub fn new(num_threads: usize) -> Self {
        let mut traverser_vec = Vec::with_capacity(num_threads);
        let mut propagator_vec = Vec::with_capacity(num_threads);
        let mut stealers = Vec::with_capacity(num_threads);

        WorkStealingMaxN {
            num_threads,
            traverser_vec,
            propagator_vec,
            stealers,
            abort_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn spawn_workers<F>(&mut self, work_fn: F)
    where
        F: Fn(Worker<T>, Worker<P>, &[Stealer<T>], Arc<AtomicBool>) + Send + Sync + Clone + 'static,
    {
        for _ in 0..self.num_threads {
            let traverser = Worker::new_lifo();
            let propagator = Worker::new_lifo();
            self.traverser_vec.push(traverser);
            self.propagator_vec.push(propagator);
        }

        for i in 0..self.num_threads {
            let mut thread_stealers = Vec::with_capacity(self.num_threads - 1);
            for j in 0..self.num_threads {
                if i != j {
                    let stealer = self.traverser_vec[j].stealer();
                    thread_stealers.push(stealer);
                }
            }
            thread_stealers.shuffle(&mut thread_rng());
            self.stealers.push(thread_stealers);
        }

        let abort_flag = Arc::clone(&self.abort_flag);

        scope(|s| {
            for _ in 0..self.traverser_vec.len() {
                let work_fn = work_fn.clone();
                let traverser = self.traverser_vec.pop().unwrap();
                let propagator = self.propagator_vec.pop().unwrap();
                let stealers = self.stealers.pop().unwrap();
                let abort_flag = Arc::clone(&abort_flag);

                s.spawn(move |_| {
                    work_fn(traverser, propagator, &stealers, abort_flag);
                });
            }
        })
        .unwrap();
    }
}

fn find_task<T, P>(
    traverser: &Worker<T>,
    propagator: &Worker<P>,
    stealers: &[Stealer<T>],
) -> Option<Task<T, P>> {
    if let Some(task) = propagator.pop() {
        return Some(Task::Propagate(task));
    }
    if let Some(task) = traverser.pop() {
        return Some(Task::Traverse(task));
    }
    match stealers.iter().find_map(|s| s.steal().success()) {
        Some(task) => Some(Task::Traverse(task)),
        None => None,
    }
}

fn worker_fn<T, P>(
    traverser: Worker<T>,
    propagator: Worker<P>,
    stealers: &[Stealer<T>],
    abort_flag: Arc<AtomicBool>,
) {
    while abort_flag.load(Relaxed) {
        match find_task(&traverser, &propagator, stealers) {
            Some(Task::Propagate(task)) => {
                todo!();
            }
            Some(Task::Traverse(task)) => {
                todo!();
            }
            None => {}
        }
    }
    end_slavery(&traverser, &propagator)
}
pub fn end_slavery<T, P>(traverser: &Worker<T>, propagator: &Worker<P>) {
    while propagator.pop().is_some() {}
    while traverser.pop().is_some() {}
}
// fn mpsc_thread(rx: mpsc::Receiver<Task1Result>, injector: Injector<Work2>) {
//     for result in rx {
//         let new_tasks = compute_new_tasks(result);
//         for task in new_tasks {
//             injector.push(task);
//         }
//     }
// }
