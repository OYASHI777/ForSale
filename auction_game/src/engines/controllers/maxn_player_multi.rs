// use crate::game_modes::traits::Game;
// use crate::models::game_state::GameState;
// use crossbeam::deque::{Injector, Steal, Stealer, Worker};
// use rand::prelude::ThreadRng;
// use rand::thread_rng;
// use std::sync::mpsc;
//
// pub struct MaxNMultiPlayer {
//     id: u8,
//     nickname: String,
//     rng: ThreadRng,
//     workers: Vec<Worker<GameState>>,
//     stealers: Vec<Stealer<GameState>>, //
//     injector: Injector<GameState>,     // Global queue where new tasks are pushed
//     tx: mpsc::Sender<GameState>,
//     rx: mpsc::Receiver<GameState>,
// }
//
// impl MaxNMultiPlayer {
//     pub fn new(id: u8, nickname: String, num_workers: usize, buffer_size: usize) -> Self {
//         let mut workers: Vec<Worker<GameState>> = Vec::with_capacity(num_workers);
//         let mut stealers: Vec<Stealer<GameState>> = Vec::with_capacity(num_workers);
//
//         let injector = Injector::new();
//
//         let (tx, rx) = mpsc::channel::<GameState>();
//
//         for _ in 0..num_workers {
//             let worker: Worker<GameState> = Worker::new_fifo();
//             stealers.push(worker.stealer());
//             workers.push(worker);
//         }
//
//         MaxNMultiPlayer {
//             id,
//             nickname,
//             rng: thread_rng(),
//             workers,
//             stealers,
//             injector,
//             tx,
//             rx,
//         }
//     }
//
//     pub fn initialise(&self) {
//         //     Start mpsc channel
//         //     Create threads for workers
//     }
//
//     fn worker_loop<T>(
//         worker: Worker<T>,
//         injector: Injector<T>,
//         stealers: Vec<Stealer<T>>,
//         tx: mpsc::Sender<T>,
//     ) where
//         T: Send + 'static,
//     {
//         loop {
//             if let Some(task) = find_task(&worker, &injector, &stealers) {
//                 // Execute the task
//                 println!("Executing task in worker: {:?}", thread::current().id());
//                 // ...
//
//                 // Send the end product through the MPSC channel
//                 tx.send(end_product).unwrap();
//             } else {
//                 // No tasks found, yield or sleep
//                 thread::yield_now();
//             }
//         }
//     }
//     fn find_task<T>(
//         worker: &Worker<T>,
//         injector: &Injector<T>,
//         stealers: &[Stealer<T>],
//     ) -> Option<T> {
//         worker.pop().or_else(|| {
//             injector
//                 .steal_batch_and_pop(worker)
//                 .or_else(|| stealers.iter().map(|s| s.steal()).collect())
//                 .and_then(|s| s.success())
//         })
//     }
// }
