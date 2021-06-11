#![feature(destructuring_assignment)]
extern crate rand;
extern crate sfml;

use rand::distributions::Distribution;
use rand::distributions::Uniform;
use sfml::graphics::*;
use sfml::graphics::blend_mode::Equation::ReverseSubtract;
use sfml::graphics::blend_mode::Factor;
use sfml::system::*;
use sfml::window::*;
use rand::{thread_rng, Rng};
// use std::time::{SystemTime, UNIX_EPOCH};

const T_START: f32 = -3.0;
const T_END: f32 = 3.0;
const ITERS: usize = 800;
const STEPS_PER_FRAME: usize = 150;
const DELTA_PER_STEP: f32 = 0.00001;
const DELTA_MINIMUM: f32 = 0.0000001;
const NUM_PARAMS: usize = 18;
const VERTEX_ARRAY_SIZE: usize = ITERS * STEPS_PER_FRAME;

const WINDOW_H: u32 = 1200;
const WINDOW_W: u32 = 1800;
const WINDOWS_BITS: u32 = 24;

fn get_random_color() -> Color {
    let mut rng = thread_rng();

    let r: u8 = rng.gen_range(0..=255);
    let g: u8 = rng.gen_range(0..=255);
    let b: u8 = rng.gen_range(0..=255);

    Color::rgb(r, g, b)
}

fn to_screen(x: f32, y: f32, plot_scale: f32, plot_x: f32, plot_y: f32) -> Vector2f {
    let s: f32 = plot_scale * ((WINDOW_H / 2) as f32);
    let nx: f32 = WINDOW_W as f32 * 0.5 + (x - plot_x) * s;
    let ny: f32 = WINDOW_H as f32 * 0.5 + (y - plot_y) * s;
    Vector2f::new(nx, ny)
}

fn params_to_string(params: &Vec<f32>) -> String {
    let base27 = String::from("_ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    let mut a: usize = 0;
    let mut n: usize = 0;
    let mut result = String::from("");
    for i in 0..NUM_PARAMS {
        a = a * 3 + params[i] as usize + 1;
        n += 1;
        if n == 3 {
            result.push(base27.chars().nth(a).unwrap());
            a = 0;
            n = 0;
        }
    }
    result
}

// fn string_to_params(param_string: String, mut params: [f32; NUM_PARAMS]) {
//     for i in 0..NUM_PARAMS / 3 {
//         let mut a = 0;
//         let c = if i < param_string.chars().count() {
//             param_string.chars().nth(i).unwrap()
//         } else {
//             '_'
//         };
//         if c >= 'A' && c <= 'Z' {
//             a = (c as i32 - 'A' as i32) + 1;
//         } else if c >= 'a' && c <= 'z' {
//             a = (c as i32 - 'a' as i32) + 1;
//         }
//
//         params[i * 3 + 2] = (a % 3) as f32 - 1.0;
//         a /= 3;
//         params[i * 3 + 1] = (a % 3) as f32 - 1.0;
//         a /= 3;
//         params[i * 3 + 0] = (a % 3) as f32 - 1.0;
//     }
// }

fn make_equation_str(params: &Vec<f32>) -> String {
    let mut ss: String = String::from("");
    let mut is_first: bool = true;
    let equation_parts: Vec<&str> = vec![
        "x\u{00b2}", "y\u{00b2}", "t\u{00b2}", "xy", "xt", "yt", "x", "y", "t",
    ];

    for (i, eq_part) in equation_parts.iter().enumerate() {
        // println!("With Param: {}, New I: {} and Eq_part: {}", params[i], i, eq_part);
        if params[i] != 0.0 {
            if is_first {
                // println!("Is First...");
                if params[i] == -1.0 {
                    ss.push('-');
                }
            } else {
                // println!("Is Not First...");
                if params[i] == -1.0 {
                    ss.push_str(" - ");
                } else {
                    ss.push_str(" + ");
                }
            }
            ss.push_str(eq_part);
            is_first = false;
        }
    }

    ss
}

// fn reset_plot(mut plot_scale: f32, mut plot_x: f32, mut plot_y: f32) {
//     println!("Resetting Plot...");
//     plot_scale = 0.25;
//     plot_x = 0.0;
//     plot_y = 0.0;
// }

fn create_render_window() -> RenderWindow {
    let mut context_settings: ContextSettings = ContextSettings::default();
    context_settings.set_depth_bits(24);
    context_settings.set_stencil_bits(8);
    context_settings.set_antialiasing_level(8);
    context_settings.set_major_version(3);
    context_settings.set_minor_version(0);

    const VIDEO_MODE: VideoMode = VideoMode::new(WINDOW_W as u32, WINDOW_H as u32, WINDOWS_BITS);
    let mut window: RenderWindow = RenderWindow::new(
        VIDEO_MODE,
        "Rust Chaos Equations",
        Style::CLOSE,
        &Default::default(),
    );

    window.set_framerate_limit(0);
    window.set_active(false);
    window.set_vertical_sync_enabled(true);
    window.set_position(Vector2i::new(0, 0));
    window.request_focus();
    window.clear(Color::BLACK);
    window
}

fn center_plot(history: &mut Vec<Vector2f>) -> (f32, f32, f32) {
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;
    for n in 0..ITERS {
        min_x = f32::min(min_x, history[n].x);
        max_x = f32::max(max_x, history[n].x);
        min_y = f32::min(min_y, history[n].y);
        max_y = f32::max(min_y, history[n].y);
    }

    max_x = f32::min(max_x, 4.0);
    max_y = f32::min(max_y, 4.0);
    min_x = f32::max(min_x, -4.0);
    min_y = f32::max(min_y, -4.0);
    ((max_x + min_x) * 0.5, (max_y + min_y) * 0.5, 1.0 / f32::max(f32::max(max_x - min_x, max_y - min_y) * 0.6, 0.1))
    // println!("Plot X: {}, Y: {}, Scale: {}", plot_x, plot_y, plot_scale);
}


fn main() {
    let mut window = create_render_window();
    let mut t: f32 = T_START;
    let mut history: Vec<Vector2f> = Vec::with_capacity(ITERS);
    let font = Font::from_file("./Roboto-Regular.ttf").unwrap();
    // for _ in 0..ITERS {
    //     history.push(Vector2f::default());
    // }
    let mut rolling_delta = DELTA_PER_STEP;

    let mut paused = false;
    let trail_type = 1;
    let mut plot_scale: f32 = 0.25;
    let mut plot_x: f32 = 0.0;
    let mut plot_y: f32 = 0.0;
    let _dot_type = 0;
    let _load_started = false;
    let shuffle_equ = true;
    let iteration_limit = false;
    const FADE_SPEEDS: [u8; 4] = [10, 2, 0, 255];
    let mut speed_multi: f32 = 3.0;

    let mut vertex_array: VertexArray = VertexArray::new(PrimitiveType::POINTS, VERTEX_ARRAY_SIZE);
    for _ in 0..=VERTEX_ARRAY_SIZE {
        let new_vertex: Vertex = Vertex::with_pos_color(
            Vector2f::new(0 as f32, 0 as f32),
            get_random_color()
        );
        vertex_array.append(&new_vertex);
    }

    // reset_plot(plot_scale, plot_x, plot_y);
    let mut params: Vec<f32> = Vec::new();
    for _ in 0..NUM_PARAMS {
        // println!("Updating Random Params: {}", n);
        let r: usize = Uniform::new_inclusive(0, 3).sample(&mut rand::thread_rng());
        match r {
            0 => {
                params.push(1.0)
            }
            1 => {
                params.push(-1.0)
            }
            _ => {
                params.push(0.0)
            }
        };
    }

    let equ_code= params_to_string(&params);
    let mut equation_str: String = String::from("x' = ");
    equation_str.push_str(make_equation_str(&params).as_str());
    equation_str.push_str("\ny' = ");
    equation_str.push_str(make_equation_str(&params).as_str());
    equation_str.push_str("\nCode: ");
    equation_str.push_str(equ_code.as_str());
    println!("New Equ String: {}", equ_code.as_str());

    let mut equ_text: Text = Text::default();
    equ_text.set_character_size(60);
    equ_text.set_font(&font);
    println!("Equation String: {}", equation_str.as_str());

    equ_text.set_string(equation_str.as_str());
    equ_text.set_fill_color(Color::WHITE);
    equ_text.set_character_size(30);
    equ_text.set_position(Vector2f::new(10.0, 10.0));
    let mut black_box: RectangleShape = RectangleShape::new();
    let text_bounds: sfml::graphics::FloatRect = equ_text.global_bounds();
    black_box.set_position(Vector2f::new(text_bounds.left, text_bounds.height));
    black_box.set_size(Vector2f::new(text_bounds.width, text_bounds.height));
    black_box.set_fill_color(Color::BLACK);
    let fade: BlendMode = BlendMode::new(Factor::One, Factor::One, ReverseSubtract, Factor::SrcColor, Factor::SrcColor, ReverseSubtract);
    let mut render_blur: RenderStates = RenderStates::default();
    render_blur.set_blend_mode(fade);
    let mut fullscreen_rect = RectangleShape::default();
    fullscreen_rect.set_position(Vector2f::new(0.0, 0.0));
    fullscreen_rect.set_size(Vector2f::new(WINDOW_W as f32, WINDOW_H as f32));

    let fade_speed: u8 = FADE_SPEEDS[trail_type];


    loop {
        // let start = SystemTime::now();
        // events
        while let Some(ev) = window.poll_event() {
            match ev {
                Event::Closed => {
                    window.close();
                    return;
                }
                Event::KeyPressed {
                    code: Key::ESCAPE, ..
                } => {
                    window.close();
                    return;
                }
                Event::KeyPressed {
                    code: Key::Q, ..
                } => {
                    window.close();
                    return;
                }
                Event::KeyPressed {
                    code: Key::P, ..
                } => {
                    paused = !paused;
                    println!("Paused App? : {}", paused);
                }
                Event::KeyPressed {
                    code: Key::LSHIFT, ..
                } => {
                    println!("Decreasing Speed...");
                    speed_multi = 0.1;
                }
                Event::KeyPressed {
                    code: Key::RSHIFT, ..
                } => {
                    println!("Increasing Speed..");
                    speed_multi = 10.0;
                }
                Event::KeyPressed {
                    code: Key::SPACE, ..
                } => {
                    println!("Resetting Speed..");
                    speed_multi = 1.0;
                }
                Event::KeyPressed {
                    code: Key::C, ..
                } => {
                    println!("Centering Plot..");
                    (plot_x, plot_y, plot_scale) = center_plot(&mut history);
                }
                _ => {}
            }
        }

        if paused {
            window.display();
            continue;
        }

        window.clear(Color::BLACK);
        // window.draw(&black_box);
        window.draw(&equ_text);
        if fade_speed >= 1 {
            fullscreen_rect.set_fill_color(Color::rgba(fade_speed, fade_speed, fade_speed, 0));
            window.draw_rectangle_shape(&fullscreen_rect, &render_blur);
        }
        // Automatic restart
        if t > T_END {
            if shuffle_equ {
                // reset_plot(plot_scale, plot_x, plot_y);
                params = Vec::new();
                for _ in 0..NUM_PARAMS {
                    let r: usize = Uniform::new_inclusive(0, 3).sample(&mut rand::thread_rng());
                    match r {
                        0 => {
                            params.push(1.0)
                        }
                        1 => {
                            params.push(-1.0)
                        }
                        _ => {
                            params.push(0.0)
                        }
                    };
                }
                // generate_new(&mut window, &params, &font);
            }
        }

        // println!("Before Chaos Time: {}", SystemTime::now().duration_since(start).unwrap().as_millis());
        // let chaos_start_time = SystemTime::now();
        let delta: f32 = DELTA_PER_STEP * speed_multi;
        rolling_delta = rolling_delta * 0.99 + delta * 0.01;
        for step in 0..STEPS_PER_FRAME {
            // let step_start_time = SystemTime::now();
            let mut is_off_screen: bool = true;
            let mut x = t;
            let mut y = t;

            for iter in 0..=ITERS {
                let xx = x * x;
                let yy = y * y;
                let tt = t * t;
                let xy = x * y;
                let xt = x * t;
                let yt = y * t;
                let nx = xx * params[0] + yy * params[1] + tt * params[2] + xy * params[3] + xt * params[4] + yt * params[5] + x * params[6] + y * params[7] + t * params[8];
                let ny = xx * params[9] + yy * params[10] + tt * params[11] + xy * params[12] + xt * params[13] + yt * params[14] + x * params[15] + y * params[16] + t * params[17];
                x = nx;
                y = ny;
                let mut screen_pt = to_screen(x, y, plot_scale, plot_x, plot_y);
                if iteration_limit && iter < 100 {
                    // println!("Setting XY to Max...");
                    screen_pt.x = std::f32::MAX;
                    screen_pt.y = std::f32::MAX;
                }

                vertex_array[step * ITERS + iter].position = screen_pt;
                // println!("Vertex @ {} is {:?}", step * ITERS + iter, screen_pt);
                if screen_pt.x > 0.0 && screen_pt.y > 0.0 && screen_pt.x < WINDOW_W as f32 && screen_pt.y < WINDOW_H as f32 {
                    // println!("Checking if Delta needs to be adjusted..");
                    let dx = history[iter].x - x;
                    let dy = history[iter].y - y;
                    let dist = 500.0 * (dx * dx + dy * dy).sqrt();
                    rolling_delta = f32::min(rolling_delta, f32::max(delta / (dist + 1e-5), DELTA_MINIMUM as f32 * speed_multi as f32));
                    is_off_screen = false;
                }

                if history.get(iter).is_none() {
                    history.push(Vector2f::(x, y));
                }
                else {
                    history[iter].x = x;
                    history[iter].y = y;
                }
            }

            if is_off_screen {
                t += 0.01;
            } else {
                t += rolling_delta;
            }
            // println!("Step took: {}", SystemTime::now().duration_since(step_start_time).unwrap().as_millis());
        }
        // println!("Chaos Step took: {}", SystemTime::now().duration_since(chaos_start_time).unwrap().as_millis());
        // println!("Before Draw Time: {}", SystemTime::now().duration_since(start).unwrap().as_millis());
        let mut t_text = Text::default();
        t_text.set_character_size(60);
        let text_string = format!("t = {}", t);
        // println!("t = {}", t);
        t_text.set_string(text_string.as_str());
        t_text.set_fill_color(Color::WHITE);
        t_text.set_font(&font);
        t_text.set_character_size(28);
        t_text.set_position(Vector2f::new(WINDOW_W as f32 - 200.0, 10.0));

        let mut t_text_box: RectangleShape = RectangleShape::new();
        let t_text_bounds: sfml::graphics::FloatRect = t_text.global_bounds();
        t_text_box.set_position(Vector2f::new(t_text_bounds.left, t_text_bounds.height));
        t_text_box.set_size(Vector2f::new(t_text_bounds.width + 5.0, t_text_bounds.height + 10.0));
        t_text_box.set_fill_color(Color::BLACK);

        window.draw(&t_text_box);
        window.draw(&t_text);
        window.draw(&vertex_array);

        window.display();
        // println!("End Time: {}", SystemTime::now().duration_since(start).unwrap().as_millis());
    }
}
