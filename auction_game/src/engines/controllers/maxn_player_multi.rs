use crossbeam::scope;
use crossbeam_deque::{Injector, Stealer, Worker};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Relaxed, SeqCst};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};
use std::thread;

// TODO: Make a score updater trait for the P
enum Task<T, P> {
    Traverse(T),
    Propagate(P),
}
enum CompletedJob<L, M> {
    Traverse(L),
    Propagate(M),
}

struct WorkStealingMaxN<L, M> {
    num_threads: usize,
    abort_flag: Arc<AtomicBool>,
    tx: Sender<CompletedJob<L, M>>,
    rx: Receiver<CompletedJob<L, M>>,
}

impl<L: Send + 'static, M: Send + 'static> WorkStealingMaxN<L, M> {
    pub fn new(num_threads: usize) -> Self {
        let (tx, rx) = mpsc::channel();
        WorkStealingMaxN {
            num_threads,
            abort_flag: Arc::new(AtomicBool::new(false)),
            tx,
            rx,
        }
    }

    pub fn spawn_workers<F, T, P>(&mut self, work_fn: F)
    where
        F: Fn(Worker<T>, Worker<P>, &[Stealer<T>], Arc<AtomicBool>, Sender<CompletedJob<L, M>>)
            + Send
            + Sync
            + Clone
            + 'static,
        T: Send + 'static,
        P: Send + 'static,
    {
        let mut traverser_vec = Vec::with_capacity(self.num_threads);
        let mut propagator_vec = Vec::with_capacity(self.num_threads);
        let mut stealers = Vec::with_capacity(self.num_threads);
        for _ in 0..self.num_threads {
            let traverser = Worker::new_lifo();
            let propagator = Worker::new_lifo();
            traverser_vec.push(traverser);
            propagator_vec.push(propagator);
        }

        for i in 0..self.num_threads {
            let mut thread_stealers = Vec::with_capacity(self.num_threads - 1);
            for j in 0..self.num_threads {
                if i != j {
                    let stealer = traverser_vec[j].stealer();
                    thread_stealers.push(stealer);
                }
            }
            thread_stealers.shuffle(&mut thread_rng());
            stealers.push(thread_stealers);
        }

        let abort_flag = Arc::clone(&self.abort_flag);

        scope(|s| {
            for _ in 0..traverser_vec.len() {
                let work_fn = work_fn.clone();
                let traverser = traverser_vec.pop().unwrap();
                let propagator = propagator_vec.pop().unwrap();
                let stealers = stealers.pop().unwrap();
                let abort_flag = Arc::clone(&abort_flag);
                let tx = self.tx.clone();

                s.spawn(move |_| {
                    work_fn(traverser, propagator, &stealers, abort_flag, tx);
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
