use std::collections::HashSet;
use std::iter::FromIterator;

pub mod algorithms;

const DICTIONARY: &str = include_str!("../dictionary.txt");

pub struct Wordle {
    dictionary: HashSet<&'static str>,
}

impl Wordle {
    pub fn new() -> Self {
        Self {
            dictionary: HashSet::from_iter(DICTIONARY.lines().map(|line| {
                line.split_once(' ')
                    .expect("every line must have the answer and its frequency count")
                    .0
            })),
        }
    }

    pub fn play<G: Guesser>(&self, answer: &'static str, mut guesser: G) -> Option<usize> {
        let mut history: Vec<Guess> = Vec::new();
        // Wordle only allows six guesses.
        // We allow more in order to avoid cutting off the score distribution for stats purposes.
        for i in 1..=32 {
            let guess = guesser.guess(&history);
            // Check that it's a valid answer.
            assert!(self.dictionary.contains(&*guess));

            if guess == answer {
                return Some(i);
            }

            let correctness = Correctness::compute(answer, &guess);
            history.push(Guess {
                word: guess,
                mask: correctness,
            });
        }

        None
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Correctness {
    /// Green
    Correct,
    /// Yellow
    Misplaced,
    /// Gray
    Wrong,
}

impl Correctness {
    fn compute(answer: &str, guess: &str) -> [Self; 5] {
        assert_eq!(answer.len(), 5);
        assert_eq!(guess.len(), 5);

        let mut c = [Correctness::Wrong; 5];
        // Mark letters green.
        for (i, (a, g)) in answer.chars().zip(guess.chars()).enumerate() {
            if a == g {
                c[i] = Correctness::Correct;
            }
        }

        let mut used = [false; 5];
        for (i, &c) in c.iter().enumerate() {
            if c == Correctness::Correct {
                used[i] = true;
            }
        }

        // Mark letters yellow.
        for (i, g) in guess.chars().enumerate() {
            if c[i] == Correctness::Correct {
                continue;
            }

            if answer.chars().enumerate().any(|(i, a)| {
                if a == g && !used[i] {
                    used[i] = true;
                    return true;
                }

                false
            }) {
                c[i] = Correctness::Misplaced;
            }
        }

        c
    }
}

pub struct Guess {
    pub word: String,
    pub mask: [Correctness; 5],
}

impl Guess {
    pub fn new(word: String, mask: [Correctness; 5]) -> Self {
        Self { word, mask }
    }

    pub fn matches(&self, word: &str) -> bool {
        assert_eq!(self.word.len(), 5);
        assert_eq!(word.len(), 5);

        let mut used = [false; 5];
        for (i, ((g, &m), w)) in self
            .word
            .chars()
            .zip(&self.mask)
            .zip(word.chars())
            .enumerate()
        {
            if m == Correctness::Correct {
                if g != w {
                    return false;
                } else {
                    used[i] = true;
                }
            }
        }

        dbg!(used);

        for (i, (w, &m)) in word.chars().zip(&self.mask).enumerate() {
            if m == Correctness::Correct {
                // Must be correct, or we would have returned in the earlier loop.
                continue;
            }

            let mut plausible = true;
            if self
                .word
                .chars()
                .zip(&self.mask)
                .enumerate()
                .any(|(j, (g, m))| {
                    if g != w {
                        return false;
                    }
                    if used[j] {
                        return false;
                    }

                    match m {
                        Correctness::Correct => unreachable!("we shouldn't reach this point"),
                        Correctness::Misplaced if j == i => {
                            // `w` was misplaced the last time around, which
                            // means that the word cannot be the answer.
                            plausible = false;
                            return false;
                        }
                        Correctness::Misplaced => {
                            used[j] = true;
                            return true;
                        }
                        Correctness::Wrong => {
                            // TODO: early return
                            plausible = false;
                            return false;
                        }
                    }
                })
                && plausible
            {
                // The character `w` was either guessed correctly, or was previously misplaced.
            } else if !plausible {
                return false;
            } else {
                // We have no information about the character `w`, so the word might still match.
            }
        }

        true
    }
}

pub trait Guesser {
    fn guess(&mut self, history: &[Guess]) -> String;
}

#[cfg(test)]
macro_rules! guesser {
    (|$history:ident| $impl:block) => {{
        struct G;
        impl $crate::Guesser for G {
            fn guess(&mut self, $history: &[$crate::Guess]) -> String {
                $impl
            }
        }
        G
    }};
}

#[cfg(test)]
macro_rules! mask {
    (C) => { $crate::Correctness::Correct };
    (M) => { $crate::Correctness::Misplaced };
    (W) => { $crate::Correctness::Wrong };
    ($($c:tt)+) => {
        [$(mask!($c)),+]
    }
}

#[cfg(test)]
mod tests {
    mod guess_matcher {
        use crate::Guess;

        macro_rules! check {
            ($prev:literal + [$($mask:tt)+] allows $next:literal) => {
                let g = Guess::new($prev.to_string(), mask![$($mask)+]);
                assert!(g.matches($next))
            };
            ($prev:literal + [$($mask:tt)+] disallows $next:literal) => {
                let g = Guess::new($prev.to_string(), mask![$($mask)+]);
                assert!(!g.matches($next))
            };
        }

        #[test]
        fn matches() {
            check!("abcde" + [C C C C C] allows "abcde");
            check!("abcdf" + [C C C C C] disallows "abcde");
            check!("abcde" + [W W W W W] allows "mnopq");
            check!("abcde" + [M M M M M] allows "eabcd");
            check!("baaaa" + [W C M W W] allows "aaccc");
            check!("baaaa" + [W C M W W] disallows "caacc");
            check!("aaabb" + [C M W W W] disallows "accaa");
            check!("abcde" + [W W W W W] disallows "baedc");
        }
    }

    mod play {
        use crate::Wordle;

        #[test]
        fn genius() {
            let w = Wordle::new();
            let guesser = guesser!(|_history| { "right".to_string() });
            assert_eq!(w.play("right", guesser), Some(1));
        }

        #[test]
        fn magnificent() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 1 {
                    return "right".to_string();
                }

                return "wrong".to_string();
            });
            assert_eq!(w.play("right", guesser), Some(2));
        }

        #[test]
        fn impressive() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 2 {
                    return "right".to_string();
                }

                return "wrong".to_string();
            });
            assert_eq!(w.play("right", guesser), Some(3));
        }

        #[test]
        fn splendid() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 3 {
                    return "right".to_string();
                }

                return "wrong".to_string();
            });
            assert_eq!(w.play("right", guesser), Some(4));
        }

        #[test]
        fn great() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 4 {
                    return "right".to_string();
                }

                return "wrong".to_string();
            });
            assert_eq!(w.play("right", guesser), Some(5));
        }

        #[test]
        fn phew() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 5 {
                    return "right".to_string();
                }

                return "wrong".to_string();
            });
            assert_eq!(w.play("right", guesser), Some(6));
        }

        #[test]
        fn oops() {
            let w = Wordle::new();
            let guesser = guesser!(|_history| { "wrong".to_string() });
            assert_eq!(w.play("right", guesser), None);
        }
    }

    mod compute {
        use crate::Correctness;

        #[test]
        fn all_green() {
            assert_eq!(Correctness::compute("abcde", "abcde"), mask!(C C C C C));
        }

        #[test]
        fn all_gray() {
            assert_eq!(Correctness::compute("abcde", "fghij"), mask!(W W W W W));
        }

        #[test]
        fn all_yellow() {
            assert_eq!(Correctness::compute("abcde", "baecd"), mask!(M M M M M));
        }

        #[test]
        fn repeat_green() {
            assert_eq!(Correctness::compute("aabbb", "aaccc"), mask!(C C W W W));
        }

        #[test]
        fn repeat_yellow() {
            assert_eq!(Correctness::compute("aabbb", "ccaac"), mask!(W W M M W));
        }

        #[test]
        fn some_green_and_yellow() {
            assert_eq!(Correctness::compute("azzaz", "aaabb"), mask!(C M W W W));
        }
    }
}
