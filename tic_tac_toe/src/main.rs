#[macro_use]
extern crate conrod;
#[macro_use]
extern crate clap;

use clap::{Arg, App};
use conrod::{widget, color, Colorable, Positionable, Widget};
use conrod::backend::glium::glium;
use conrod::backend::glium::glium::{DisplayBuild, Surface};

widget_ids!(
    struct Ids {
        canvas,
        grids[],
        circles[],
        xs[],
    });

#[derive(Copy, Clone, Debug)]
enum Turn {
    Black,
    White,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum BoardState {
    Circle,
    X,
    Empty,
}

fn main() {
    const TITLE: &'static str = "Tic Tac Toe";
    let matches = App::new(TITLE)
        .arg(Arg::with_name("width").short("w").takes_value(true))
        .arg(Arg::with_name("height").short("h").takes_value(true))
        .arg(Arg::with_name("cols").short("c").takes_value(true))
        .arg(Arg::with_name("rows").short("r").takes_value(true))
        .get_matches();

    let width = value_t!(matches, "width", u32).unwrap_or(300);
    let height = value_t!(matches, "height", u32).unwrap_or(300);
    let rows = value_t!(matches, "rows", usize).unwrap_or(3);
    let cols = value_t!(matches, "cols", usize).unwrap_or(3);

    println!("width: {}", width);
    println!("height: {}", height);
    println!("rows: {}", rows);
    println!("cols: {}", cols);

    // Build the window.
    let display = glium::glutin::WindowBuilder::new()
        .with_vsync()
        .with_dimensions(width, height)
        .with_title(TITLE)
        .with_multisampling(4)
        .build_glium()
        .unwrap();

    // construct our `Ui`.
    let mut ui = conrod::UiBuilder::new([width as f64, height as f64]).build();

    // Generate the widget identifiers.
    let ids = &mut Ids::new(ui.widget_id_generator());
    ids.grids
        .resize((rows - 1) * (cols - 1), &mut ui.widget_id_generator());
    ids.circles
        .resize(rows * cols, &mut ui.widget_id_generator());
    ids.xs
        .resize(rows * cols * 2, &mut ui.widget_id_generator());

    // A type used for converting `conrod::render::Primitives` into `Command`s that can be used
    // for drawing to the glium `Surface`.
    let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();

    //let mut board: [[BoardState; cols]; rows] = [[BoardState::Empty; cols]; rows];
    let mut board = vec![BoardState::Empty; cols * rows];
    let mut board: Vec<_> = board.as_mut_slice().chunks_mut(cols).collect();
    let mut board: &mut [&mut [_]] = board.as_mut_slice();
    let mut turn = Turn::Black;

    // Poll events from the window.
    let mut event_loop = EventLoop::new();
    'main: loop {
        // Handle all events.
        for event in event_loop.next(&display) {
            // Use the `winit` backend feature to convert the winit event to a conrod one.
            if let Some(event) = conrod::backend::winit::convert(event.clone(), &display) {
                ui.handle_event(event);
                event_loop.needs_update();
            }

            match event {
                // Break from the loop upon `Escape`.
                glium::glutin::Event::KeyboardInput(
                    _,
                    _,
                    Some(glium::glutin::VirtualKeyCode::Escape),
                ) |
                glium::glutin::Event::Closed => break 'main,
                glium::glutin::Event::KeyboardInput(
                    glium::glutin::ElementState::Pressed,
                    _,
                    Some(glium::glutin::VirtualKeyCode::R),
                ) => {
                    println!("reset board");
                    for y in 0..rows {
                        for x in 0..cols {
                            board[y][x] = BoardState::Empty;
                        }
                    }
                    turn = Turn::Black;
                }
                glium::glutin::Event::KeyboardInput(
                    glium::glutin::ElementState::Pressed,
                    _,
                    Some(glium::glutin::VirtualKeyCode::D),
                ) => {
                    // for debug
                    println!("turn: {:?}", turn);
                    println!("{:?}", board);
                }
                _ => {}
            }
        }

        set_widgets(ui.set_widgets(), ids, &mut board, &mut turn);

        // Render the `Ui` and then display it on the screen.
        if let Some(primitives) = ui.draw_if_changed() {
            renderer.fill(&display, primitives, &image_map);
            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 0.0, 1.0);
            renderer.draw(&display, &mut target, &image_map).unwrap();
            target.finish().unwrap();
        }
    }
}

fn set_widgets(
    ref mut ui: conrod::UiCell,
    ids: &mut Ids,
    board: &mut [&mut [BoardState]],
    turn: &mut Turn,
) {
    widget::Canvas::new()
        .pad(0.0)
        .color(color::WHITE)
        .set(ids.canvas, ui);

    let rows = board.len();
    let cols = board[0].len();
    let canvas_wh = ui.wh_of(ids.canvas).unwrap();
    let tl = [-canvas_wh[0] / 2., canvas_wh[1] / 2.];
    let sw = canvas_wh[0] / (cols as f64);
    let sh = canvas_wh[1] / (rows as f64);

    // draw grid line
    for x in 1..cols {
        widget::Line::abs(
            [tl[0] + (x as f64) * sw, tl[1]],
            [tl[0] + (x as f64) * sw, tl[1] - canvas_wh[1] + 1.],
        ).color(color::BLACK)
            .set(ids.grids[x - 1], ui);
    }
    for y in 1..rows {
        widget::Line::abs(
            [tl[0], tl[1] - (y as f64) * sh],
            [tl[0] + canvas_wh[1] - 1., tl[1] - (y as f64) * sh],
        ).color(color::BLACK)
            .set(ids.grids[y - 1 + cols - 1], ui);
    }

    // mouse click
    if let Some(mouse) = ui.widget_input(ids.canvas).mouse() {
        if mouse.buttons.left().is_down() {
            let mouse_abs_xy = mouse.abs_xy();
            let x = ((mouse_abs_xy[0] + (canvas_wh[0] / 2.)) / (canvas_wh[0] / (cols as f64))) as
                usize;
            let y = ((canvas_wh[1] / 2. - mouse_abs_xy[1]) / (canvas_wh[1] / (rows as f64))) as
                usize;
            // println!("{:?}, {}, {}", mouse_abs_xy, x, y);

            // when resizing, x can be greater than cols (so do y)
            if x < cols && y < rows {
                if board[y][x] == BoardState::Empty {
                    match *turn {
                        Turn::White => {
                            board[y][x] = BoardState::Circle;
                            *turn = Turn::Black;
                        }
                        Turn::Black => {
                            board[y][x] = BoardState::X;
                            *turn = Turn::White;
                        }
                    }
                }
            }
        }
    }

    // draw circle or x
    for y in 0..rows {
        for x in 0..cols {
            match board[y][x] {
                BoardState::Circle => {
                    widget::Circle::outline_styled(
                        sw / 3.,
                        widget::line::Style::new().thickness(2.),
                    ).x(tl[0] + sw * (x as f64) + sw / 2.)
                        .y(tl[1] - sh * (y as f64) - sh / 2.)
                        .color(color::RED)
                        .set(ids.circles[y * cols + x], ui);
                }
                BoardState::X => {
                    widget::Line::abs(
                        [
                            tl[0] + sw * (x as f64) + sw / 5.,
                            tl[1] - sh * (y as f64) - sh / 5.,
                        ],
                        [
                            tl[0] + sw * ((x + 1) as f64) - sw / 5.,
                            tl[1] - sh * ((y + 1) as f64) + sh / 5.,
                        ],
                    ).color(color::BLACK)
                        .thickness(2.)
                        .set(ids.xs[y * cols + x], ui);

                    widget::Line::abs(
                        [
                            tl[0] + sw * ((x + 1) as f64) - sw / 5.,
                            tl[1] - sh * (y as f64) - sh / 5.,
                        ],
                        [
                            tl[0] + sw * (x as f64) + sw / 5.,
                            tl[1] - sh * ((y + 1) as f64) + sh / 5.,
                        ],
                    ).color(color::BLACK)
                        .thickness(2.)
                        .set(ids.xs[y * cols + x + rows * cols], ui);
                }
                _ => {}
            }
        }
    }
}

struct EventLoop {
    ui_needs_update: bool,
    last_update: std::time::Instant,
}

impl EventLoop {
    pub fn new() -> Self {
        EventLoop {
            last_update: std::time::Instant::now(),
            ui_needs_update: true,
        }
    }

    /// Produce an iterator yielding all available events.
    pub fn next(&mut self, display: &glium::Display) -> Vec<glium::glutin::Event> {
        // We don't want to loop any faster than 60 FPS, so wait until it has been at least 16ms
        // since the last yield.
        let last_update = self.last_update;
        let sixteen_ms = std::time::Duration::from_millis(16);
        let duration_since_last_update = std::time::Instant::now().duration_since(last_update);
        if duration_since_last_update < sixteen_ms {
            std::thread::sleep(sixteen_ms - duration_since_last_update);
        }

        // Collect all pending events.
        let mut events = Vec::new();
        events.extend(display.poll_events());

        // If there are no events and the `Ui` does not need updating, wait for the next event.
        if events.is_empty() && !self.ui_needs_update {
            events.extend(display.wait_events().next());
        }

        self.ui_needs_update = false;
        self.last_update = std::time::Instant::now();

        events
    }

    /// Notifies the event loop that the `Ui` requires another update whether or not there are any
    /// pending events.
    ///
    /// This is primarily used on the occasion that some part of the `Ui` is still animating and
    /// requires further updates to do so.
    pub fn needs_update(&mut self) {
        self.ui_needs_update = true;
    }
}
