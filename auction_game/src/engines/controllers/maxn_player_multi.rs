use crate::game_modes::traits::Game;
use crate::models::enums::GamePhase;
use crate::models::game_state::GameState;
use crossbeam::scope;
use crossbeam_deque::{Injector, Stealer, Worker};
use dashmap::DashMap;
use log::warn;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::process::abort;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Relaxed, SeqCst};
use std::sync::mpsc::{Receiver, SendError, Sender};
use std::sync::{mpsc, Arc};
use std::thread;

// TODO: Make a score updater trait for the P
struct ScoreMaxN {
    pub game_state: GameState,
    pub score: Vec<f32>,
    pub remaining_children: usize,
    pub average_count: usize,
}

impl ScoreMaxN {
    pub fn default(
        game_state: &GameState,
        remaining_children: usize,
        average_count: usize,
    ) -> Self {
        let no_players = game_state.no_players() as usize;
        ScoreMaxN {
            game_state: game_state.clone(),
            score: vec![f32::MIN; no_players],
            remaining_children,
            average_count,
        }
    }
}

#[derive(Clone)]
struct GameStateJob {
    pub game_state: GameState,
    pub root_round_no: u8,
    pub root_turn_no: u8,
    pub terminal_condition: TerminalCondition,
}

impl GameStateJob {
    pub fn new(
        game_state: GameState,
        root_round_no: u8,
        root_turn_no: u8,
        terminal_condition: TerminalCondition,
    ) -> Self {
        GameStateJob {
            game_state,
            root_round_no,
            root_turn_no,
            terminal_condition,
        }
    }
}

#[derive(Copy, Clone)]
enum TerminalCondition {
    // Contains number of rounds after root round to terminate
    RoundEnd(u8),
}

enum Task {
    Traverse(GameStateJob),
    Propagate(ScoreMaxN),
}
enum CompletedJob<L, M> {
    Traverse(L),
    Propagate(M),
}

struct WorkStealingMaxN {
    num_threads: usize,
    scores: Arc<DashMap<String, ScoreMaxN>>,
    pause_flag: Arc<AtomicBool>,
    abort_flag: Arc<AtomicBool>,
    tx: Sender<ScoreMaxN>,
    rx: Receiver<ScoreMaxN>,
}

impl WorkStealingMaxN {
    pub fn new(num_threads: usize) -> Self {
        let (tx, rx) = mpsc::channel();
        let scores = Arc::new(DashMap::with_capacity(50000));
        WorkStealingMaxN {
            num_threads,
            scores,
            pause_flag: Arc::new(AtomicBool::new(false)),
            abort_flag: Arc::new(AtomicBool::new(false)),
            tx,
            rx,
        }
    }

    pub fn spawn_workers<F>(&mut self, work_fn: F)
    where
        F: Fn(
                Worker<GameStateJob>,
                Worker<ScoreMaxN>,
                &[Stealer<GameStateJob>],
                &[Stealer<ScoreMaxN>],
                Arc<DashMap<String, ScoreMaxN>>,
                Arc<AtomicBool>,
                Arc<AtomicBool>,
                Sender<GameStateJob>,
            ) + Send
            + Sync
            + Clone
            + 'static,
    {
        let (tx, rx) = mpsc::channel::<GameStateJob>();

        let mut traverser_vec: Vec<Worker<GameStateJob>> = Vec::with_capacity(self.num_threads);
        let mut propagator_vec: Vec<Worker<ScoreMaxN>> = Vec::with_capacity(self.num_threads);
        let mut traverser_stealers_vec: Vec<Vec<Stealer<GameStateJob>>> =
            Vec::with_capacity(self.num_threads);
        let mut propagator_stealers_vec: Vec<Vec<Stealer<ScoreMaxN>>> =
            Vec::with_capacity(self.num_threads);
        // Creating Workers
        for _ in 0..self.num_threads {
            let traverser: Worker<GameStateJob> = Worker::new_lifo();
            let propagator: Worker<ScoreMaxN> = Worker::new_lifo();
            traverser_vec.push(traverser);
            propagator_vec.push(propagator);
        }

        // Creating Stealers
        for i in 0..self.num_threads {
            let mut temp_traverser_stealers = Vec::with_capacity(self.num_threads - 1);
            let mut temp_propagator_stealers = Vec::with_capacity(self.num_threads - 1);
            for j in 0..self.num_threads {
                if i != j {
                    let traverser_stealer = traverser_vec[j].stealer();
                    let propagator_stealer = propagator_vec[j].stealer();
                    temp_traverser_stealers.push(traverser_stealer);
                    temp_propagator_stealers.push(propagator_stealer);
                }
            }
            temp_traverser_stealers.shuffle(&mut thread_rng());
            temp_propagator_stealers.shuffle(&mut thread_rng());
            traverser_stealers_vec.push(temp_traverser_stealers);
            propagator_stealers_vec.push(temp_propagator_stealers);
        }

        scope(|s| {
            for _ in 0..traverser_vec.len() {
                let work_fn = work_fn.clone();
                let traverser = traverser_vec.pop().unwrap();
                let propagator = propagator_vec.pop().unwrap();
                let traverser_stealers = traverser_stealers_vec.pop().unwrap();
                let propagator_stealers = propagator_stealers_vec.pop().unwrap();
                let scores = Arc::clone(&self.scores);
                let pause_flag = Arc::clone(&self.pause_flag);
                let abort_flag = Arc::clone(&self.abort_flag);
                let tx_move = tx.clone();

                s.spawn(move |_| {
                    work_fn(
                        traverser,
                        propagator,
                        &traverser_stealers,
                        &propagator_stealers,
                        scores,
                        pause_flag,
                        abort_flag,
                        tx_move,
                    );
                });
            }
        })
        .unwrap();
        //     TODO: Add Injectors for rx to send Score workload out
        let abort_flag = Arc::clone(&self.abort_flag);
        thread::spawn(move || handle_received_scores(rx, abort_flag));
    }
    //     TODO: Function to get evaluation going
    //     TODO: Function to end evaluation early
}

fn find_task(
    traverser: &Worker<GameStateJob>,
    propagator: &Worker<ScoreMaxN>,
    traverser_stealers: &[Stealer<GameStateJob>],
    propagator_stealers: &[Stealer<ScoreMaxN>],
) -> Option<Task> {
    if let Some(task) = propagator.pop() {
        return Some(Task::Propagate(task));
    }
    if let Some(task) = traverser.pop() {
        return Some(Task::Traverse(task));
    }
    if let Some(task) = traverser_stealers.iter().find_map(|s| s.steal().success()) {
        return Some(Task::Traverse(task));
    }
    match propagator_stealers.iter().find_map(|s| s.steal().success()) {
        Some(task) => Some(Task::Propagate(task)),
        None => None,
    }
}

fn worker_fn(
    traverser: Worker<GameStateJob>,
    propagator: Worker<ScoreMaxN>,
    traverser_stealers: &[Stealer<GameStateJob>],
    propagator_stealers: &[Stealer<ScoreMaxN>],
    scores: Arc<DashMap<String, ScoreMaxN>>,
    pause_flag: Arc<AtomicBool>,
    abort_flag: Arc<AtomicBool>,
    tx: Sender<GameStateJob>,
) {
    while !abort_flag.load(Relaxed) {
        while !pause_flag.load(Relaxed) {
            // TODO: Figure a proper way to pause and save computation cost
            match find_task(
                &traverser,
                &propagator,
                traverser_stealers,
                propagator_stealers,
            ) {
                Some(Task::Propagate(task)) => {
                    todo!();
                }
                Some(Task::Traverse(job)) => {
                    traverse(&traverser, &scores, &tx, &job);
                }
                None => {}
            }
        }
        end_slavery(&traverser, &propagator)
    }
}

fn traverse(
    traverser: &Worker<GameStateJob>,
    scores: &Arc<DashMap<String, ScoreMaxN>>,
    tx: &Sender<GameStateJob>,
    job: &GameStateJob,
) {
    match job.terminal_condition {
        TerminalCondition::RoundEnd(search_depth) => {
            if job.game_state.auction_end() {
                if job.game_state.round_no() == job.root_round_no + search_depth
                    || job.game_state.game_phase() == GamePhase::Sell
                {
                    // send terminal state to mpsc
                    get_score(&tx, &job);
                } else {
                    // TODO: ++ Custom depth by round
                    // End of round but not terminal
                    deepen_auction_end(&scores, &job);
                }
            } else {
                deepen_standard(&traverser, &scores, &job);
            }
        }
        _ => {
            unimplemented!();
        }
    }
}

fn get_score(tx: &Sender<GameStateJob>, job: &GameStateJob) {
    loop {
        match tx.send(job.clone()) {
            Ok(_) => {
                todo!("Score the thing and send it away!");
                break;
            }
            Err(_) => {
                warn!(
                    "Failed to send job to mpsc {:?}",
                    job.game_state.get_path_encoding()
                );
            }
        }
    }
}

fn deepen_auction_end(scores: &Arc<DashMap<String, ScoreMaxN>>, job: &GameStateJob) {
    let random_sample = true;
    let n_samples = 1;
    let chances_leaves = job
        .game_state
        .reveal_auction_perms(random_sample, n_samples);
    for game_state in chances_leaves {
        let score = ScoreMaxN::default(&game_state, n_samples as usize, 0);
        scores.insert(game_state.get_path_encoding(), score);
    }
}

fn deepen_standard(
    traverser: &Worker<GameStateJob>,
    scores: &Arc<DashMap<String, ScoreMaxN>>,
    game_state_job: &GameStateJob,
) {
    let legal_moves: Vec<u8> = game_state_job
        .game_state
        .legal_moves(game_state_job.game_state.current_player());
    let child_states_count: usize = legal_moves.len();
    for action in legal_moves {
        let child_state = game_state_job
            .game_state
            .manual_next_state_bid(game_state_job.game_state.current_player(), action);
        let next_game_state_job = GameStateJob::new(
            child_state,
            game_state_job.root_round_no,
            game_state_job.root_turn_no,
            game_state_job.terminal_condition,
        );
        traverser.push(next_game_state_job);
    }
    let score = ScoreMaxN::default(&game_state_job.game_state, child_states_count, 0);
    scores.insert(game_state_job.game_state.get_path_encoding(), score);
}
fn deepen_average(
    traverser: &Worker<GameStateJob>,
    scores: &Arc<DashMap<String, ScoreMaxN>>,
    game_state_job: GameStateJob,
) {
    todo!("Average of Permutations!");
}

fn handle_received_scores(rx: Receiver<GameStateJob>, abort_flag: Arc<AtomicBool>) {
    while !abort_flag.load(Relaxed) {
        match rx.recv() {
            Ok(gamestate) => {
                todo!("Evaluate Scores");
            }
            Err(_) => {
                // Channel has been closed, exit the loop
                break;
            }
        }
    }
}

pub fn end_slavery(traverser: &Worker<GameStateJob>, propagator: &Worker<ScoreMaxN>) {
    while propagator.pop().is_some() {}
    while traverser.pop().is_some() {}
}
