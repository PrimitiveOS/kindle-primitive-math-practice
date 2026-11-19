use serde::{Deserialize, Serialize};

pub const SET_LEN: usize = 5;
pub const INPUT_LIMIT: usize = 18;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Skill {
    Arithmetic,
    Fractions,
    Percentages,
}

impl Skill {
    pub const ALL: [Self; 3] = [Self::Arithmetic, Self::Fractions, Self::Percentages];

    fn index(self) -> usize {
        match self {
            Self::Arithmetic => 0,
            Self::Fractions => 1,
            Self::Percentages => 2,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Arithmetic => "Arithmetic",
            Self::Fractions => "Fractions",
            Self::Percentages => "Percentages",
        }
    }

    pub fn example(self) -> (&'static str, &'static str) {
        match self {
            Self::Arithmetic => (
                "24 ÷ 6 = 4",
                "Find the number that makes 6 × ? = 24. Four groups of six make 24.",
            ),
            Self::Fractions => (
                "1/2 + 1/4 = 3/4",
                "Change 1/2 to 2/4 to use the same denominator. Add 2/4 and 1/4.",
            ),
            Self::Percentages => (
                "25% of 20 = 5",
                "25% means 25 out of 100, or 1/4. Divide 20 by 4.",
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Exact {
    n: i64,
    d: i64,
}

impl Exact {
    fn new(n: i64, d: i64) -> Option<Self> {
        if d == 0 {
            return None;
        }
        let sign = d.signum();
        let (mut a, mut b) = (n.abs(), d.abs());
        while b != 0 {
            (a, b) = (b, a % b);
        }
        Some(Self {
            n: n / a * sign,
            d: d / a * sign,
        })
    }

    pub fn parse(entry: &str) -> Option<Self> {
        if entry.is_empty() || entry.len() > INPUT_LIMIT {
            return None;
        }
        let integer = |part: &str| {
            let value = part.parse::<i64>().ok()?;
            (-999_999..=999_999).contains(&value).then_some(value)
        };
        if let Some((n, d)) = entry.split_once('/') {
            return Self::new(integer(n)?, integer(d)?);
        }
        if let Some((whole, fraction)) = entry.split_once('.') {
            let negative = whole.starts_with('-');
            let whole = whole.strip_prefix('-').unwrap_or(whole);
            if whole.len() > 6
                || fraction.len() > 6
                || (whole.is_empty() && fraction.is_empty())
                || !whole
                    .bytes()
                    .chain(fraction.bytes())
                    .all(|b| b.is_ascii_digit())
            {
                return None;
            }
            let d = 10_i64.pow(fraction.len() as u32);
            let n = whole.parse::<i64>().unwrap_or(0) * d + fraction.parse::<i64>().unwrap_or(0);
            return Self::new(if negative { -n } else { n }, d);
        }
        Self::new(integer(entry)?, 1)
    }

    fn display(self) -> String {
        if self.d == 1 {
            self.n.to_string()
        } else {
            format!("{}/{}", self.n, self.d)
        }
    }
}

pub struct Problem {
    pub prompt: String,
    pub hint: String,
    pub explanation: String,
    answer: Exact,
}

impl Problem {
    fn new(prompt: String, hint: &str, explanation: String, n: i64, d: i64) -> Self {
        Self {
            prompt,
            hint: hint.into(),
            explanation,
            answer: Exact::new(n, d).unwrap(),
        }
    }

    pub fn correct(&self, value: Exact) -> bool {
        self.answer == value
    }

    pub fn answer_text(&self) -> String {
        self.answer.display()
    }
}

pub fn problem(skill: Skill, seed: u64, index: usize) -> Problem {
    let pattern = (index + (seed % 4) as usize) % 4;
    let mut seed = seed.wrapping_add((index as u64).wrapping_mul(0x9e3779b97f4a7c15));
    let mut sample = |count: u64| {
        seed ^= seed >> 30;
        seed = seed.wrapping_mul(0xbf58476d1ce4e5b9);
        seed ^= seed >> 27;
        seed = seed.wrapping_mul(0x94d049bb133111eb);
        seed ^= seed >> 31;
        (seed % count) as i64
    };
    let a = 2 + sample(18);
    let b = 2 + sample(10);
    match skill {
        Skill::Arithmetic => match pattern {
            0 => Problem::new(
                format!("{a} + {b}"),
                "Count up from the larger number.",
                format!("Adding {b} to {a} gives {}.", a + b),
                a + b,
                1,
            ),
            1 => Problem::new(
                format!("{} − {b}", a + b),
                "Count up from the number being subtracted.",
                format!("Subtracting {b} from {} leaves {a}.", a + b),
                a,
                1,
            ),
            2 => Problem::new(
                format!("{a} × {b}"),
                "Think of equal groups.",
                format!("{a} groups of {b} make {}.", a * b),
                a * b,
                1,
            ),
            _ => Problem::new(
                format!("{} ÷ {b}", a * b),
                "Find the related multiplication fact.",
                format!("Since {a} × {b} = {}, the quotient is {a}.", a * b),
                a,
                1,
            ),
        },
        Skill::Fractions => {
            let c = 2 + sample(7);
            match index % 4 {
                0 => Problem::new(format!("1/{b} + 1/{c}"), "Use a common denominator.", format!("Use denominator {}. Add the numerators {c} and {b}.", b * c), b + c, b * c),
                1 => Problem::new(format!("{}/{b} − 1/{b}", b - 1), "Both fractions have the same denominator.", format!("Subtract 1 from {} and keep denominator {b}.", b - 1), b - 2, b),
                2 => Problem::new(format!("Simplify {b}/{}", b * c), "Divide numerator and denominator by the same number.", format!("Divide the numerator and denominator by {b} to get 1/{c}."), 1, c),
                _ => Problem::new(format!("({a}/{b}) × (1/{c})"), "Multiply the two numerators, then the two denominators.", format!("Multiply {a} by 1 and {b} by {c}. The product is {a}/{} before simplification.", b * c), a, b * c),
            }
        }
        Skill::Percentages => {
            let percentage = [10, 20, 25, 50, 75][index % SET_LEN];
            let total = a * 4 + b;
            Problem::new(
                format!("What is {percentage}% of {total}?"),
                "A percent is a fraction out of 100.",
                format!("Multiply {total} by {percentage}, then divide by 100."),
                percentage * total,
                100,
            )
        }
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct SkillHistory {
    pub sets: u32,
    pub best: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub skill: Skill,
    pub seed: u64,
    queue: Vec<usize>,
    step: usize,
    marked: [Option<bool>; SET_LEN],
    pub entry: String,
    pub hint: bool,
}

impl Session {
    fn new(skill: Skill, seed: u64, queue: Vec<usize>) -> Self {
        Self {
            skill,
            seed,
            queue,
            step: 0,
            marked: [None; SET_LEN],
            entry: String::new(),
            hint: false,
        }
    }

    pub fn current(&self) -> Option<usize> {
        self.queue.get(self.step).copied()
    }

    pub fn position(&self) -> (usize, usize) {
        (self.step + 1, self.queue.len())
    }

    pub fn result(&self) -> Option<bool> {
        self.marked[self.current()?]
    }

    pub fn finished(&self) -> bool {
        self.step >= self.queue.len()
    }

    pub fn score(&self) -> (usize, usize) {
        (
            self.queue
                .iter()
                .filter(|&&i| self.marked[i] == Some(true))
                .count(),
            self.queue.len(),
        )
    }

    pub fn missed(&self) -> Vec<usize> {
        self.queue
            .iter()
            .copied()
            .filter(|&i| self.marked[i] == Some(false))
            .collect()
    }

    pub fn key(&mut self, key: &str) -> bool {
        if self.finished() || self.result().is_some() {
            return false;
        }
        match key {
            "Erase" => return self.entry.pop().is_some(),
            "Clear" if !self.entry.is_empty() => self.entry.clear(),
            digit if digit.len() == 1 && self.entry.len() < INPUT_LIMIT && valid_keys(digit) => {
                self.entry.push_str(digit)
            }
            _ => return false,
        }
        true
    }

    pub fn check(&mut self) -> bool {
        let Some(index) = self.current() else {
            return false;
        };
        let Some(value) = Exact::parse(&self.entry) else {
            return false;
        };
        if self.marked[index].is_some() {
            return false;
        }
        self.marked[index] = Some(problem(self.skill, self.seed, index).correct(value));
        true
    }

    fn advance(&mut self) -> bool {
        if self.result().is_none() {
            return false;
        }
        self.step += 1;
        self.entry.clear();
        self.hint = false;
        true
    }

    fn valid(&self) -> bool {
        !self.queue.is_empty()
            && self.queue.len() <= SET_LEN
            && self.step <= self.queue.len()
            && self.entry.len() <= INPUT_LIMIT
            && valid_keys(&self.entry)
            && self.queue.iter().enumerate().all(|(i, &slot)| {
                slot < SET_LEN
                    && !self.queue[..i].contains(&slot)
                    && (i >= self.step || self.marked[slot].is_some())
            })
    }
}

fn valid_keys(entry: &str) -> bool {
    entry
        .bytes()
        .all(|b| b.is_ascii_digit() || b"./-".contains(&b))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Study {
    seed: u64,
    active: Option<Skill>,
    sessions: [Option<Session>; 3],
    history: [SkillHistory; 3],
}

impl Study {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            active: None,
            sessions: Default::default(),
            history: Default::default(),
        }
    }

    pub fn unpack(bytes: &[u8]) -> Option<Self> {
        let state = serde_json::from_slice::<Self>(bytes).ok()?;
        for (i, session) in state.sessions.iter().enumerate() {
            if session
                .as_ref()
                .is_some_and(|s| !s.valid() || s.skill.index() != i)
            {
                return None;
            }
        }
        if state
            .active
            .is_some_and(|skill| state.sessions[skill.index()].is_none())
        {
            return None;
        }
        Some(state)
    }

    pub fn pack(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("math progress is serializable")
    }

    pub fn active(&self) -> Option<Skill> {
        self.active
    }

    pub fn at(&self, skill: Skill) -> Option<&Session> {
        self.sessions[skill.index()].as_ref()
    }

    pub fn current(&self) -> Option<&Session> {
        self.at(self.active?)
    }

    pub fn current_mut(&mut self) -> Option<&mut Session> {
        self.sessions[self.active?.index()].as_mut()
    }

    pub fn history(&self, skill: Skill) -> &SkillHistory {
        &self.history[skill.index()]
    }

    pub fn resume(&mut self, skill: Skill) -> bool {
        if self.at(skill).is_some_and(|session| !session.finished()) {
            self.active = Some(skill);
            true
        } else {
            false
        }
    }

    pub fn start(&mut self, skill: Skill) {
        self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.sessions[skill.index()] = Some(Session::new(skill, self.seed, (0..SET_LEN).collect()));
        self.active = Some(skill);
    }

    pub fn retry(&mut self) -> bool {
        let Some(prior) = self.current().filter(|session| session.finished()) else {
            return false;
        };
        let queue = prior.missed();
        if queue.is_empty() {
            return false;
        }
        let skill = prior.skill;
        self.sessions[skill.index()] = Some(Session::new(skill, prior.seed, queue));
        true
    }

    pub fn advance(&mut self) -> bool {
        let Some(session) = self.current_mut() else {
            return false;
        };
        if !session.advance() {
            return false;
        }
        if session.finished() && session.queue.len() == SET_LEN {
            let (correct, _) = session.score();
            let history = &mut self.history[session.skill.index()];
            history.sets = history.sets.saturating_add(1);
            history.best = history.best.max(correct);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equivalent_answers_are_exact_and_malformed_values_are_rejected() {
        assert_eq!(Exact::parse("1/2"), Exact::parse("2/4"));
        assert_eq!(Exact::parse("1/2"), Exact::parse(".5"));
        assert_eq!(Exact::parse("-25/100"), Exact::parse("-0.25"));
        for invalid in [
            "1/0",
            "2..3",
            "1/-",
            "NaN",
            "1000000000000000000000000",
            ".",
            "-",
        ] {
            assert_eq!(Exact::parse(invalid), None, "{invalid}");
        }
    }

    #[test]
    fn exercises_are_deterministic_and_display_a_parseable_answer() {
        for skill in Skill::ALL {
            for seed in [0, 1, 4099, u64::MAX] {
                for index in 0..SET_LEN {
                    let pr = problem(skill, seed, index);
                    assert!(pr.correct(Exact::parse(&pr.answer_text()).unwrap()));
                    assert_eq!(pr.prompt, problem(skill, seed, index).prompt);
                }
            }
        }
    }

    #[test]
    fn reopens_a_set_and_retries_misses_without_replacing_history() {
        let mut study = Study::new(5);
        study.start(Skill::Arithmetic);
        let first = study.current_mut().unwrap();
        assert!(first.key("0"));
        assert!(first.check());
        let mut study = Study::unpack(&study.pack()).unwrap();
        assert_eq!(study.current().unwrap().result(), Some(false));
        assert!(study.advance());
        for _ in 1..SET_LEN {
            let round = study.current_mut().unwrap();
            round.entry = problem(round.skill, round.seed, round.current().unwrap()).answer_text();
            assert!(round.check());
            assert!(study.advance());
        }
        assert_eq!(study.history(Skill::Arithmetic).best, 4);
        assert!(study.retry());
        assert_eq!(study.current().unwrap().position(), (1, 1));
    }
}
