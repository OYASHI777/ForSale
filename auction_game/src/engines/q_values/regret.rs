pub struct Regret {}

impl Regret {
    pub fn q_values(stored_q_values: &mut Vec<f32>, score: &Vec<f32>) {
        let max_score = score.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let regret: Vec<f32> = score
            .iter()
            .map(|&score| (max_score - score).max(0.0))
            .collect();
        todo!()
    }
}
