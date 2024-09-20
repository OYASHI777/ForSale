use log::debug;
use rand::{thread_rng, Rng};

pub fn sample_strategy(vec: &Vec<f32>) -> usize {
    let mut rng = thread_rng();
    let sample: f32 = rng.gen::<f32>();
    let mut total: f32 = 0.0;
    for i in 0..vec.len() {
        total += vec[i];
        if sample < total {
            return i;
        }
    }
    let random_index = (sample * vec.len() as f32).floor();
    random_index as usize
}

pub fn mixed_strategy_score(strategy: &Vec<f32>, score: &Vec<f32>) -> f32 {
    debug_assert!(
        strategy.len() == score.len(),
        "strategy len: {} should be equal to score len: {}",
        strategy.len(),
        score.len()
    );
    let mut weighted_average: f32 = 0.0;
    for (&st, &sc) in strategy.iter().zip(score.iter()) {
        weighted_average += st * sc;
    }
    weighted_average
}

pub fn normalize(strategy: &mut Vec<f32>, q_value_vec: &Vec<f32>) {
    let total: f32 = q_value_vec.iter().sum();
    for (s, q) in strategy.iter_mut().zip(q_value_vec.iter()) {
        *s = *q / total;
    }
}
