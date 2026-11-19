use std::{
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use math_practice_core::{problem, Skill, Study, SET_LEN};
use primitive_app::{App, Change, WIDTH};
use primitive_ui::{
    back_button, header, screen_rect, Canvas, Input, Rect, TextStyle, BODY, CAPTION, PAPER,
};

const KEYS: [&str; 15] = [
    "7", "8", "9", "Erase", "4", "5", "6", "Clear", "1", "2", "3", "/", "-", "0", ".",
];

#[derive(Clone, Copy)]
enum Screen {
    Skills,
    Example(Skill),
    Practice,
}

pub struct View {
    pub study: Study,
    screen: Screen,
    path: Option<PathBuf>,
    pending_save: bool,
    invalid: bool,
}

impl View {
    pub fn open(path: PathBuf) -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(5, |now| now.as_nanos() as u64);
        Self::with_seed(Some(path), seed)
    }

    pub fn with_seed(path: Option<PathBuf>, seed: u64) -> Self {
        let study = path
            .as_deref()
            .and_then(load)
            .unwrap_or_else(|| Study::new(seed));
        Self {
            study,
            screen: Screen::Skills,
            path,
            pending_save: false,
            invalid: false,
        }
    }

    fn persist(&mut self) {
        self.pending_save = true;
        let _ = self.save();
    }

    fn skills(&self, canvas: &mut Canvas, w: i32) {
        header(canvas, "Math Practice", "Done");
        canvas.text(
            screen_rect(w, 40, 209, 1184, 75),
            "Choose a skill",
            TextStyle::new(48).bold(),
        );
        if let Some(skill) = self
            .study
            .active()
            .filter(|_| self.study.current().is_some_and(|s| !s.finished()))
        {
            canvas.button(first(w, 325), "Continue", true, false);
            canvas.text(screen_rect(w, 660, 357, 564, 64), skill.name(), BODY);
        }
        for (i, skill) in Skill::ALL.into_iter().enumerate() {
            let history = self.study.history(skill);
            canvas.button(skill_button(w, i), skill.name(), false, false);
            if self.study.at(skill).is_some_and(|s| !s.finished()) {
                canvas.text(
                    screen_rect(w, 40, 657 + i as i32 * 264, 1184, 62),
                    "Set in progress",
                    CAPTION.center(),
                );
            } else if history.sets > 0 {
                canvas.text(
                    screen_rect(w, 40, 657 + i as i32 * 264, 1184, 62),
                    &format!(
                        "Sets completed: {}   Best: {} of {SET_LEN}",
                        history.sets, history.best
                    ),
                    CAPTION.center(),
                );
            }
        }
    }

    fn example(&self, canvas: &mut Canvas, w: i32, skill: Skill) {
        header(canvas, "Math Practice", "Back");
        canvas.text(
            screen_rect(w, 40, 209, 1184, 75),
            skill.name(),
            TextStyle::new(48).bold(),
        );
        canvas.panel(screen_rect(w, 40, 330, 1184, 700), 22, 2, PAPER);
        canvas.text(screen_rect(w, 88, 395, 1088, 65), "Worked example", CAPTION);
        let (equation, explanation) = skill.example();
        canvas.text(
            screen_rect(w, 88, 515, 1088, 100),
            equation,
            TextStyle::new(64).bold(),
        );
        canvas.text(
            screen_rect(w, 88, 680, 1088, 270),
            explanation,
            TextStyle::new(42).lines(5),
        );
        canvas.button(first(w, 1450), "Start", true, false);
        canvas.text(
            screen_rect(w, 660, 1484, 564, 56),
            "Five questions",
            CAPTION,
        );
    }

    fn practice(&self, canvas: &mut Canvas, w: i32) {
        header(canvas, "Math Practice", "Back");
        let Some(session) = self.study.current() else {
            return;
        };
        canvas.text(
            screen_rect(w, 40, 190, 1184, 62),
            session.skill.name(),
            BODY.bold(),
        );
        let Some(index) = session.current() else {
            let (correct, total) = session.score();
            canvas.text(
                screen_rect(w, 40, 420, 1184, 93),
                "Set complete",
                TextStyle::new(64).bold().center(),
            );
            canvas.text(
                screen_rect(w, 40, 555, 1184, 70),
                &format!("{correct} of {total} correct"),
                BODY.center(),
            );
            canvas.button(first(w, 960), "New set", correct == total, false);
            if correct < total {
                canvas.button(second(w, 960), "Retry missed", true, false);
            }
            canvas.button(first(w, 1450), "Skills", false, false);
            return;
        };
        let pr = problem(session.skill, session.seed, index);
        let (number, total) = session.position();
        canvas.text(
            screen_rect(w, 40, 275, 1184, 56),
            &format!("Question {number} of {total}"),
            CAPTION,
        );
        canvas.text(
            screen_rect(w, 40, 355, 1184, 112),
            &pr.prompt,
            TextStyle::new(58).bold().lines(2),
        );
        let field = screen_rect(w, 40, 488, 1184, 106);
        canvas.panel(field, 18, 2, PAPER);
        if session.entry.is_empty() {
            canvas.text(screen_rect(w, 74, 520, 1116, 59), "Your answer", CAPTION);
        } else {
            canvas.text(
                screen_rect(w, 74, 508, 1116, 72),
                &session.entry,
                TextStyle::new(46),
            );
        }
        if session.result().is_none() {
            for (i, key) in KEYS.iter().enumerate() {
                canvas.button(key_button(w, i), key, false, false);
            }
            let tip = if self.invalid {
                "Enter a whole number, decimal, or fraction with a nonzero denominator."
            } else if session.hint {
                &pr.hint
            } else {
                ""
            };
            canvas.text(screen_rect(w, 40, 1154, 1184, 176), tip, BODY.lines(3));
            canvas.button(first(w, 1450), "Check", true, false);
            canvas.button(second(w, 1450), "Hint", false, false);
        } else {
            let result = if session.result() == Some(true) {
                "Correct"
            } else {
                "Not quite"
            };
            canvas.panel(screen_rect(w, 40, 688, 1184, 450), 22, 2, PAPER);
            canvas.text(
                screen_rect(w, 88, 742, 1088, 80),
                result,
                TextStyle::new(48).bold(),
            );
            canvas.text(
                screen_rect(w, 88, 846, 1088, 166),
                &pr.explanation,
                BODY.lines(3),
            );
            canvas.text(
                screen_rect(w, 88, 1026, 1088, 66),
                &format!("Answer: {}", pr.answer_text()),
                BODY.bold(),
            );
            canvas.button(first(w, 1450), "Next", true, false);
        }
    }

    fn tap(&mut self, x: f32, y: f32) -> Change {
        let w = WIDTH as i32;
        if back_button(w).contains(x, y) {
            if matches!(self.screen, Screen::Skills) {
                return Change::Exit;
            }
            self.screen = Screen::Skills;
            self.invalid = false;
            return Change::Full;
        }

        match self.screen {
            Screen::Skills => {
                if first(w, 325).contains(x, y)
                    && self.study.current().is_some_and(|s| !s.finished())
                {
                    self.screen = Screen::Practice;
                } else if let Some(i) =
                    (0..Skill::ALL.len()).find(|&i| skill_button(w, i).contains(x, y))
                {
                    let skill = Skill::ALL[i];
                    if self.study.resume(skill) {
                        self.screen = Screen::Practice;
                        self.persist();
                    } else {
                        self.screen = Screen::Example(skill);
                    }
                } else {
                    return Change::None;
                }
                Change::Full
            }
            Screen::Example(skill) => {
                if !first(w, 1450).contains(x, y) {
                    return Change::None;
                }
                self.study.start(skill);
                self.screen = Screen::Practice;
                self.persist();
                Change::Full
            }
            Screen::Practice => {
                let Some(session) = self.study.current_mut() else {
                    return Change::None;
                };
                if session.finished() {
                    if first(w, 1450).contains(x, y) {
                        self.screen = Screen::Skills;
                    } else if first(w, 960).contains(x, y) {
                        self.screen = Screen::Example(session.skill);
                    } else if second(w, 960).contains(x, y) && self.study.retry() {
                        self.persist();
                    } else {
                        return Change::None;
                    }
                    return Change::Full;
                }
                if session.result().is_some() {
                    if first(w, 1450).contains(x, y) && self.study.advance() {
                        self.persist();
                        return Change::Full;
                    }
                } else if first(w, 1450).contains(x, y) {
                    self.invalid = !session.check();
                    if !self.invalid {
                        self.persist();
                    }
                    return Change::Partial;
                } else if second(w, 1450).contains(x, y) {
                    self.invalid = false;
                    session.hint = true;
                    self.persist();
                    return Change::Partial;
                } else if let Some(i) = (0..KEYS.len()).find(|&i| key_button(w, i).contains(x, y)) {
                    if session.key(KEYS[i]) {
                        self.invalid = false;
                        self.persist();
                        return Change::Partial;
                    }
                }
                Change::None
            }
        }
    }
}

impl App for View {
    fn draw(&self, canvas: &mut Canvas) {
        let w = canvas.bounds().w;
        match self.screen {
            Screen::Skills => self.skills(canvas, w),
            Screen::Example(skill) => self.example(canvas, w, skill),
            Screen::Practice => self.practice(canvas, w),
        }
        if self.pending_save {
            canvas.text(
                screen_rect(w, 40, 1590, 1184, 60),
                "Progress could not be saved yet.",
                CAPTION,
            );
        }
    }

    fn input(&mut self, input: Input) -> Change {
        match input {
            Input::Tap(x, y) => self.tap(x, y),
            Input::PageBack if !matches!(self.screen, Screen::Skills) => {
                self.screen = Screen::Skills;
                self.invalid = false;
                Change::Full
            }
            _ => Change::None,
        }
    }

    fn save(&mut self) -> io::Result<()> {
        if self.pending_save {
            if let Some(path) = &self.path {
                primitive_app::save(path, &self.study.pack())?;
            }
            self.pending_save = false;
        }
        Ok(())
    }

    fn storage_returned(&mut self) -> io::Result<()> {
        self.save()
    }
}

fn load(path: &Path) -> Option<Study> {
    primitive_app::with_storage(|| {
        let mut bytes = Vec::new();
        File::open(path)?.take(16_385).read_to_end(&mut bytes)?;
        Ok(Study::unpack(&bytes).filter(|_| bytes.len() <= 16_384))
    })
    .ok()
    .flatten()
}

fn first(w: i32, y: i32) -> Rect {
    screen_rect(w, 40, y, 564, 110)
}

fn second(w: i32, y: i32) -> Rect {
    screen_rect(w, 660, y, 564, 110)
}

fn skill_button(w: i32, i: usize) -> Rect {
    screen_rect(w, 40, 510 + i as i32 * 264, 1184, 126)
}

fn key_button(w: i32, i: usize) -> Rect {
    screen_rect(
        w,
        40 + (i % 4) as i32 * 300,
        620 + (i / 4) as i32 * 127,
        284,
        106,
    )
}
