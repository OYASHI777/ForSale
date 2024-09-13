// use crossbeam_deque::{Injector, Stealer, Worker};
// use std::sync::atomic::AtomicBool;
// use std::sync::atomic::Ordering::{Relaxed, SeqCst};
// use std::sync::{mpsc, Arc};
// use std::thread;
//
// enum Task<T, P> {
//     Traverse(T),
//     Propagate(P),
// }
// struct WorkStealingMaxN<T> {
//     traverser_vec: Vec<Worker<T>>,
//     propagator_vec: Vec<Worker<T>>,
//     stealers: Vec<Stealer<T>>,
//     abort_flag: Arc<AtomicBool>,
// }
//
// impl<T: Send + 'static> WorkStealingMaxN<T> {
//     pub fn new(num_threads: usize) -> Self {
//         let mut traverser_vec = Vec::with_capacity(num_threads);
//         let mut propagator_vec = Vec::with_capacity(num_threads);
//         let mut stealers = Vec::with_capacity(num_threads);
//
//         for _ in 0..num_threads {
//             let traverser = Worker::new_fifo();
//             let stealer = traverser.stealer();
//             let propagator = Worker::new_fifo();
//             traverser_vec.push(traverser);
//             propagator_vec.push(propagator);
//             stealers.push(stealer);
//         }
//
//         WorkStealingMaxN {
//             traverser_vec,
//             propagator_vec,
//             stealers,
//             abort_flag: Arc::new(AtomicBool::new(false)),
//         }
//     }
//
//     pub fn spawn_workers<F, T, P>(&self, work_fn: F)
//     where
//         F: Fn(&Worker<T>, &Worker<P>, &[Stealer<T>], Arc<AtomicBool>) + Send + Clone + 'static,
//     {
//         let stealers = Arc::new(self.stealers.clone());
//
//         let num_threads = self.traverser_vec.len();
//
//         for i in 0..num_threads {
//             let work_fn = work_fn.clone();
//             let stealers = Arc::clone(&stealers);
//             let traverser = self.traverser_vec[i].clone();
//             let propagator = self.propagator_vec[i].clone();
//
//             thread::spawn(move || {
//                 work_fn(traverser, propagator, &stealers, self.abort_flag.clone());
//             });
//         }
//     }
//
//     pub fn end_slavery(&mut self) {
//         for propagator in self.propagator_vec {
//             while propagator.pop().is_some() {}
//         }
//         for traverser in self.traverser_vec {
//             while traverser.pop().is_some() {}
//         }
//         //     TODO: Clear mpsc side of things
//     }
// }
//
// fn find_task<T, P>(
//     traverser: &Worker<T>,
//     propagator: &Worker<P>,
//     stealers: &[Stealer<T>],
// ) -> Option<Task<T, P>> {
//     match propagator.pop() {
//         Some(task) => Some(Task::Propagate(task)),
//         None => {}
//     }
//     match traverser.pop() {
//         Some(task) => Some(Task::Traverse(task)),
//         None => {}
//     }
//     match stealers.iter().find_map(|s| s.steal().success()) {
//         Some(task) => Some(Task::Traverse(task)),
//         None => None,
//     }
// }
//
// fn worker_fn<T>(
//     traverser: &Worker<T>,
//     propagator: &Worker<T>,
//     stealers: &[Stealer<T>],
//     abort_flag: Arc<AtomicBool>,
// ) {
//     while abort_flag.load(Relaxed) {
//         match find_task(traverser, propagator, stealers) {
//             Some(Task::Propagate(task)) => {
//                 todo!();
//             }
//             Some(Task::Traverse(task)) => {
//                 todo!();
//             }
//             None => {}
//         }
//     }
// }
//
// fn mpsc_thread(rx: mpsc::Receiver<Task1Result>, injector: Injector<Work2>) {
//     for result in rx {
//         let new_tasks = compute_new_tasks(result);
//         for task in new_tasks {
//             injector.push(task);
//         }
//     }
// }
