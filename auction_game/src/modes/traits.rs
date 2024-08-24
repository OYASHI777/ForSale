pub trait Game {
    fn game_run();
}

pub trait PlayerController {
    fn nickname(&self) -> String;
}
