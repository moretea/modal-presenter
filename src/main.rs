extern crate libxdo;
extern crate sdl2;

#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::TextureQuery;

#[derive(Debug, PartialEq, Deserialize)]
struct Scenario {
    meta: Option<Meta>,
    steps: Vec<Step>,
}

#[derive(Debug, PartialEq, Deserialize)]
struct Meta {
    title: Option<String>,
}

#[derive(Debug, PartialEq, Deserialize)]
struct Step {
    desc: Option<String>,
    cmd: String,
    enter: Option<bool>,
}

// Button configuration for NES30.
const BUTTON_SELECT: u8 = 10;
const BUTTON_START: u8 = 11;
const BUTTON_L: u8 = 6;
const BUTTON_R: u8 = 7;
const BUTTON_A: u8 = 0;
const BUTTON_B: u8 = 1;

const BUTTON_X: u8 = 3;
const BUTTON_Y: u8 = 4;

impl Step {
    fn press_enter(&self) -> bool {
        self.enter.is_none() || self.enter.unwrap() == true
    }
}

fn load_scenario_from_args() -> Scenario {
    let mut args = std::env::args().skip(1); // Skip first argument (program name)
    let scenario_file_name = args.next().expect("No scenario given");
    let scenario_file =
        std::fs::File::open(scenario_file_name).expect("Could not open scenario file");

    return match serde_yaml::from_reader(scenario_file) {
        Ok(x) => x,
        Err(e) => panic!("Could not parse scenario file, because: {}", e),
    };
}

#[derive(Debug, PartialEq)]
enum ViewMode {
    Demo,
    Presentation,
}

impl ViewMode {
    fn other(self) -> ViewMode {
        if self == ViewMode::Demo {
            return ViewMode::Presentation;
        } else {
            return ViewMode::Demo;
        }
    }
}

const SCROLL_LINES: usize = 10;
const DEFAULT_TITLE: &'static str = "NES presenter";
const FONT_BYTES: &'static [u8] = include_bytes!("../FiraMono-Regular.ttf");

fn main() {
    let mut scenario = load_scenario_from_args();
    let xdo = libxdo::XDo::new(None).expect("Could not initialize libxdo");
    let sdl_context = sdl2::init().expect("Could not initialize sdl2");
    let video_subsys = sdl_context
        .video()
        .expect("Could not initialize sdl2 video");
    let ttf_context = sdl2::ttf::init().expect("Could not initialize sdl2 ttf");
    let mut event_pump = sdl_context
        .event_pump()
        .expect("Could not initialize sdl2 event pump");
    let joystick_system = sdl_context
        .joystick()
        .expect("Could not initialize joystick subsystem");
    joystick_system.set_event_state(true); // Process controller events, pretty please.
    let _joystick = joystick_system.open(0);

    let font_time = ttf_context
        .load_font_from_rwops(sdl2::rwops::RWops::from_bytes(FONT_BYTES).unwrap(), 128)
        .unwrap();
    let font = ttf_context
        .load_font_from_rwops(sdl2::rwops::RWops::from_bytes(FONT_BYTES).unwrap(), 50)
        .unwrap();

    let mut scenario_step: usize = 0;
    let mut current_mode = ViewMode::Demo;
    let mut presentation_started_at: Option<std::time::Instant> = None;

    sdl2::hint::set("SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS", "1");

    let window_title: String = if let Some(meta) = scenario.meta {
        if let Some(title) = meta.title {
            title.into()
        } else {
            DEFAULT_TITLE.into()
        }
    } else {
        DEFAULT_TITLE.into()
    };

    let window = video_subsys
        .window(&window_title, 800, 600)
        .resizable()
        .opengl()
        .build()
        .expect("Could not open window");

    let mut canvas = window
        .into_canvas()
        .target_texture()
        .present_vsync()
        .build()
        .expect("Could not create canvas from window");

    let texture_creator = canvas.texture_creator();

    let mut reload_scenario = false;

    'running: loop {
        if reload_scenario {
            scenario = load_scenario_from_args();
            if scenario_step >= scenario.steps.len() {
                scenario_step = scenario.steps.len() - 1;
            }

            reload_scenario = false;
        }

        let current_step = &scenario.steps[scenario_step];

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    println!("See ya");
                    break 'running;
                }
                Event::JoyAxisMotion {
                    axis_idx: 1, value, ..
                } => {
                    if value > 0 {
                        if scenario_step < scenario.steps.len() - 1 {
                            scenario_step += 1;
                        }
                    } else if value < 0 {
                        if scenario_step > 0 {
                            scenario_step -= 1;
                        }
                    }
                }
                Event::JoyButtonDown { button_idx: BUTTON_A, .. } |
                Event::JoyButtonDown { button_idx: BUTTON_L, .. } => {
                    // A -> Send page up
                    if current_mode == ViewMode::Demo {
                        for _ in 1..SCROLL_LINES {
                            xdo.send_keysequence("ctrl+shift+Up", 0).unwrap();
                        }
                    } else {
                        xdo.send_keysequence("Page_Up", 0).unwrap();
                    }
                }
                Event::JoyButtonDown { button_idx: BUTTON_B, .. } |
                Event::JoyButtonDown { button_idx: BUTTON_R, .. } => {
                    // B -> Send page down
                    if current_mode == ViewMode::Demo {
                        for _ in 1..SCROLL_LINES {
                            xdo.send_keysequence("ctrl+shift+Down", 0).unwrap();
                        }
                    } else {
                        if presentation_started_at == None {
                            presentation_started_at = Some(std::time::Instant::now());
                        }
                        xdo.send_keysequence("Page_Down", 0).unwrap();
                    }
                }
                Event::JoyButtonDown { button_idx: BUTTON_SELECT, .. } => {
                    // Select; switch between live demo and presentation.
                    current_mode = current_mode.other();

                    match current_mode {
                        ViewMode::Presentation => {
                            xdo.send_keysequence("Super+1", 0).unwrap();
                        }
                        ViewMode::Demo => {
                            xdo.send_keysequence("Super+2", 0).unwrap();
                        }
                    }
                }
                Event::JoyButtonDown { button_idx: BUTTON_START, .. } => {
                    // Start; execute command. Jump to next item in line.
                    if current_mode == ViewMode::Demo {
                        xdo.enter_text(&current_step.cmd, 50000).unwrap();

                        if current_step.press_enter() {
                            xdo.send_keysequence("Return", 10).unwrap();
                        }

                        if scenario_step < scenario.steps.len() - 1 {
                            scenario_step += 1;
                        }
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    repeat: false,
                    ..
                } => {
                    presentation_started_at = None;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::L),
                    repeat: false,
                    ..
                } => {
                    reload_scenario = true;
                }
                _ => {/* ignore other events */ }
            }
        }

        match current_mode {
            ViewMode::Demo => {
                canvas.set_draw_color(Color::RGB(255, 0, 0));
            }
            ViewMode::Presentation => {
                canvas.set_draw_color(Color::RGB(0, 255, 0));
            }
        }

        canvas.clear();

        let (ref window_width, ref window_height) = canvas.window().size();

        let top: i32;
        let bottom: i32;

        // Print mode
        {
            let mode_surface = font_time
                .render(&format!("{:?}", &current_mode))
                .blended(Color::RGBA(0, 0, 0, 255))
                .expect("Could not render mode");
            let texture = texture_creator
                .create_texture_from_surface(&mode_surface)
                .unwrap();

            let TextureQuery { width, height, .. } = texture.query();

            let target = Rect::new((window_width / 2 - width / 2) as i32, 64, width, height);
            canvas.copy(&texture, None, Some(target)).unwrap();

            top = (64 /* top margin */ + height + 64/* margin between mode and next snippet */)
                as i32;
        }

        // Draw time of presentation
        if let Some(start_instant) = presentation_started_at {
            let total_seconds = start_instant.elapsed().as_secs();
            let minutes = total_seconds / 60;
            let seconds = total_seconds % 60;

            let time_surface = font_time
                .render(&format!("{:02}:{:02}", minutes, seconds))
                .blended(Color::RGBA(0, 0, 0, 255))
                .expect("Could not render time");
            let texture = texture_creator
                .create_texture_from_surface(&time_surface)
                .unwrap();

            let TextureQuery { width, height, .. } = texture.query();

            let target = Rect::new(
                (window_width / 2 - width / 2) as i32,
                (window_height - 64 - height) as i32,
                width,
                height,
            );
            canvas.copy(&texture, None, Some(target)).unwrap();
            bottom = (window_height - 64 - height - 64) as i32;
        } else {
            bottom = (window_height - 64) as i32;
        }

        // Print next command
        if current_mode == ViewMode::Demo {
            let mut next_top: i32 = top + 64;
            let lines: Vec<_> = current_step.cmd.lines().collect();
            for line in lines {
                if next_top > bottom {
                    break;
                }

                let color = if current_step.press_enter() {
                    Color::RGBA(0, 0, 0, 255)
                } else {
                    Color::RGBA(128, 128, 128, 255)
                };

                let surface = font
                    .render(line)
                    .blended(color)
                    .expect("Could not render cmd line");
                let texture = texture_creator
                    .create_texture_from_surface(&surface)
                    .unwrap();
                let TextureQuery { width, height, .. } = texture.query();

                let target = Rect::new(64, next_top, width, height);
                canvas.copy(&texture, None, Some(target)).unwrap();
                next_top += height as i32 + 10;
            }
        }

        canvas.present();

        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
