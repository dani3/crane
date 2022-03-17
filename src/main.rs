use crane::Wordle;

const GAMES: &'static str = include_str!("../answers.txt");

fn main() {
    let wordle = Wordle::new();
    for answer in GAMES.split_whitespace() {
        let guesser = crane::algorithms::Naive::new();
        wordle.play(answer, guesser);
    }
}
