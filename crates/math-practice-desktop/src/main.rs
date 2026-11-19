use std::{
    fs,
    path::{Path, PathBuf},
};

use math_practice_render::View;
use primitive_app::{font_paths, frame, write_png, App, Change, HEIGHT, WIDTH};
use primitive_ui::{Canvas, Input};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .unwrap_or_else(|| "build/previews".into()),
    );
    fs::create_dir_all(&output)?;
    let fonts =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../doannt-kindle-launcher/assets/fonts");
    let mut canvas = Canvas::new(WIDTH, HEIGHT, &font_paths(&fonts))?;
    let save = std::env::temp_dir().join(format!("math-practice-{}.json", std::process::id()));
    let mut view = View::with_seed(Some(save.clone()), 413);
    let mut capture = |view: &View, name: &str, dark: bool| {
        write_png(
            &output.join(name),
            WIDTH,
            HEIGHT,
            &frame(view, &mut canvas, dark),
        )
    };
    capture(&view, "skills.png", false)?;
    assert_eq!(view.input(Input::Tap(500.0, 820.0)), Change::Full);
    capture(&view, "example.png", false)?;
    assert_eq!(view.input(Input::Tap(400.0, 1480.0)), Change::Full);
    assert_eq!(view.input(Input::Tap(910.0, 1480.0)), Change::Partial);
    assert_eq!(view.input(Input::Tap(180.0, 940.0)), Change::Partial);
    assert_eq!(view.input(Input::Tap(1090.0, 940.0)), Change::Partial);
    assert_eq!(view.input(Input::Tap(480.0, 940.0)), Change::Partial);
    capture(&view, "practice-dark.png", true)?;
    let mut reopened = View::with_seed(Some(save.clone()), 77);
    assert_eq!(reopened.input(Input::Tap(440.0, 380.0)), Change::Full);
    assert_eq!(reopened.study.current().unwrap().entry, "1/2");
    assert_eq!(reopened.input(Input::Tap(440.0, 1480.0)), Change::Partial);
    capture(&reopened, "feedback.png", false)?;
    assert_eq!(reopened.input(Input::Tap(440.0, 1480.0)), Change::Full);
    assert_eq!(reopened.study.current().unwrap().position(), (2, 5));
    fs::remove_file(save)?;
    Ok(())
}
