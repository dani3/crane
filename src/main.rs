const GAMES: &'static str = include_str!("../answers.txt");

fn main() {
    for answer in GAMES.split_whitespace() {
        let guesser = crane::algorithms::Naive::new();
        crane::play(answer, guesser);
    }
}
