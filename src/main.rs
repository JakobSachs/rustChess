extern crate sdl2;

mod field;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::libc::SYS_process_vm_writev;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;

use sdl2::image::LoadTexture;

use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Duration;

static BG_COLOR: Color = Color::RGB(50, 50, 50);
static AUX_COLOR: Color = Color::RGB(100, 100, 100);
static FG_COLOR: Color = Color::RGB(170, 170, 170);
static HIGHLIGHT_COLOR: Color = Color::RGB(255, 92, 51);
static WIDTH: u32 = 1200;
static HEIGHT: u32 = 900;

// minor ui constants
static BORDER_WIDTH: i32 = 2;
static PADDING: u32 = 5;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, PartialEq, Eq)]
struct GameState {
    field: field::Field,
    whites_turn: bool,
    captured_white: Vec<field::Figure>,
    captured_black: Vec<field::Figure>,
    marked: Option<(u32, u32)>,
    possible_moves: HashSet<(u32, u32)>,
    checkmate: bool,
    draw: bool,
}

fn render_field(
    canvas: &mut WindowCanvas,
    bounds: Rect,
    state: &GameState,
    textures: &HashMap<field::Figure, sdl2::render::Texture>,
    font: &sdl2::ttf::Font,
    texture_creator: &sdl2::render::TextureCreator<sdl2::video::WindowContext>,
) {
    // draw field border
    canvas.set_draw_color(AUX_COLOR);
    canvas.fill_rect(bounds).unwrap();

    // update bounds of field to account border
    let bounds = Rect::new(
        bounds.x() + BORDER_WIDTH,
        bounds.y() + BORDER_WIDTH,
        bounds.width() - (BORDER_WIDTH * 2) as u32,
        bounds.height() - (BORDER_WIDTH * 2) as u32,
    );
    // draw field background
    canvas.set_draw_color(BG_COLOR);
    canvas.fill_rect(bounds).unwrap();

    let square_size: u32 = bounds.width() / 8;

    for x in 0..8 {
        for y in 0..8 {
            let color = if (x + y) % 2 == 0 { FG_COLOR } else { BG_COLOR };

            let mut square = Rect::new(
                bounds.x() + (x * square_size) as i32,
                bounds.y() + (y * square_size) as i32,
                square_size,
                square_size,
            );

            // account for uneven division of field size by 8
            if x == 7 {
                if square.x() + square.width() as i32
                    != bounds.x() + bounds.width() as i32
                {
                    let diff = (bounds.x() + bounds.width() as i32)
                        - (square.x() + square.width() as i32);
                    square.set_width((square.width() as i32 + diff) as u32);
                }
            }

            if y == 7 {
                if square.y() + square.height() as i32
                    != bounds.y() + bounds.height() as i32
                {
                    let diff = (bounds.y() + bounds.height() as i32)
                        - (square.y() + square.height() as i32);
                    square.set_height((square.height() as i32 + diff) as u32);
                }
            }

            canvas.set_draw_color(color);
            canvas.fill_rect(square).unwrap();

            // for edges draw letters and numbers
            if x == 0 {
                let text = font
                    .render(&format!("{}", 8 - y))
                    .blended(if color == FG_COLOR {
                        BG_COLOR
                    } else {
                        FG_COLOR
                    })
                    .unwrap();
                let texture =
                    texture_creator.create_texture_from_surface(text).unwrap();

                let dimensions = texture.query();
                // offset so that number is always at the top left corner of the square
                let target = Rect::new(
                    square.x() + PADDING as i32,
                    square.y() + PADDING as i32,
                    dimensions.width,
                    dimensions.height,
                );

                canvas.copy(&texture, None, target).unwrap();
            }

            // Render Column Letters
            if y == 7 {
                let text = font
                    .render(&format!("{}", (x as u8 + b'a') as char))
                    .blended(if color == FG_COLOR {
                        BG_COLOR
                    } else {
                        FG_COLOR
                    })
                    .unwrap();
                let texture =
                    texture_creator.create_texture_from_surface(text).unwrap();

                let dimensions = texture.query();
                // offset so letter is always at the bottom right
                let target = Rect::new(
                    square.x() - dimensions.width as i32
                        + square.width() as i32
                        - PADDING as i32,
                    square.y() - dimensions.height as i32
                        + square.height() as i32
                        - PADDING as i32,
                    dimensions.width,
                    dimensions.height,
                );

                canvas.copy(&texture, None, target).unwrap();
            }

            // draw piece
            if let Some(figure) = state.field.get(x, y) {
                let sprite = textures.get(figure).unwrap();

                // offset  figure a bit from the square
                let target = Rect::new(
                    square.x() + PADDING as i32,
                    square.y() + PADDING as i32,
                    square.width() - (PADDING * 2) as u32,
                    square.height() - (PADDING * 2) as u32,
                );
                canvas.copy(sprite, None, target).unwrap();

                // draw mark if square is marked
                if Some((x, y)) == state.marked {
                    canvas.set_draw_color(HIGHLIGHT_COLOR);
                    // inset mark a bit
                    let mark = Rect::new(
                        square.x() + PADDING as i32,
                        square.y() + PADDING as i32,
                        square.width() - (PADDING * 2) as u32,
                        square.height() - (PADDING * 2) as u32,
                    );

                    // draw rect with width using 4 rects
                    let upper =
                        Rect::new(mark.x(), mark.y(), mark.width(), PADDING);
                    let lower = Rect::new(
                        mark.x(),
                        mark.y() + mark.height() as i32 - PADDING as i32,
                        mark.width(),
                        PADDING,
                    );
                    let left =
                        Rect::new(mark.x(), mark.y(), PADDING, mark.height());
                    let right = Rect::new(
                        mark.x() + mark.width() as i32 - PADDING as i32,
                        mark.y(),
                        PADDING,
                        mark.height(),
                    );
                    canvas.fill_rect(upper).unwrap();
                    canvas.fill_rect(lower).unwrap();
                    canvas.fill_rect(left).unwrap();
                    canvas.fill_rect(right).unwrap();
                }
            }
            // check if square is in possible_moves
            if state.possible_moves.contains(&(x, y)) {
                canvas.set_draw_color(AUX_COLOR);
                // inset mark a bit
                let mark = Rect::new(
                    square.x() + PADDING as i32,
                    square.y() + PADDING as i32,
                    square.width() - (PADDING * 2) as u32,
                    square.height() - (PADDING * 2) as u32,
                );

                if let Some(..) = state.field.get(x, y) {
                    // draw rect with width using 4 rects
                    let upper =
                        Rect::new(mark.x(), mark.y(), mark.width(), PADDING);
                    let lower = Rect::new(
                        mark.x(),
                        mark.y() + mark.height() as i32 - PADDING as i32,
                        mark.width(),
                        PADDING,
                    );
                    let left =
                        Rect::new(mark.x(), mark.y(), PADDING, mark.height());
                    let right = Rect::new(
                        mark.x() + mark.width() as i32 - PADDING as i32,
                        mark.y(),
                        PADDING,
                        mark.height(),
                    );
                    canvas.fill_rect(upper).unwrap();
                    canvas.fill_rect(lower).unwrap();
                    canvas.fill_rect(left).unwrap();
                    canvas.fill_rect(right).unwrap();
                } else {
                    // for empty field draw a small rect
                    let mark = Rect::from_center(
                        mark.center(),
                        3 * PADDING as u32,
                        3 * PADDING as u32,
                    );
                    canvas.fill_rect(mark).unwrap();
                }
            }
        }
    }
}

fn render_sidebar(
    canvas: &mut WindowCanvas,
    bounds: Rect,
    state: &GameState,
    mediumfont: &sdl2::ttf::Font,
    smallfont: &sdl2::ttf::Font,
    sprites: &HashMap<field::Figure, sdl2::render::Texture>,
    texture_creator: &sdl2::render::TextureCreator<sdl2::video::WindowContext>,
) {
    // draw sidebar border
    canvas.set_draw_color(AUX_COLOR);
    canvas.fill_rect(bounds).unwrap();
    canvas.set_draw_color(BG_COLOR);
    canvas
        .fill_rect(Rect::new(
            bounds.x() + BORDER_WIDTH,
            bounds.y() + BORDER_WIDTH,
            bounds.width() - (BORDER_WIDTH * 2) as u32,
            bounds.height() - (BORDER_WIDTH * 2) as u32,
        ))
        .unwrap();

    // split sidebar into 3 parts (20%, 60%, 20%)
    let top =
        Rect::new(bounds.x(), bounds.y(), bounds.width(), bounds.height() / 5);
    let middle = Rect::new(
        bounds.x(),
        bounds.y() + top.height() as i32,
        bounds.width(),
        (bounds.height() as f32 * 0.6) as u32,
    );
    let bottom = Rect::new(
        bounds.x(),
        bounds.y() + top.height() as i32 + middle.height() as i32,
        bounds.width(),
        (bounds.height() as f32 * 0.2) as u32,
    );

    // TOP PART

    // first line
    let text_str = match state.whites_turn {
        true => "White's",
        false => "Black's",
    };
    let text = mediumfont.render(text_str).blended(FG_COLOR).unwrap();
    let texture = texture_creator.create_texture_from_surface(text).unwrap();
    let fl_dimensions = texture.query();
    let target = Rect::new(
        top.center().x() - fl_dimensions.width as i32 / 2,
        top.center().y() - fl_dimensions.height as i32 / 2,
        fl_dimensions.width,
        fl_dimensions.height,
    );
    canvas.copy(&texture, None, target).unwrap();

    // second line
    let text = mediumfont.render("turn").blended(FG_COLOR).unwrap();
    let texture = texture_creator.create_texture_from_surface(text).unwrap();
    let sl_dimensions = texture.query();
    let target = Rect::new(
        bounds.center().x() - sl_dimensions.width as i32 / 2,
        top.center().y() - sl_dimensions.height as i32 / 2
            + fl_dimensions.height as i32,
        sl_dimensions.width,
        sl_dimensions.height,
    );
    canvas.copy(&texture, None, target).unwrap();

    // MIDDLE PART
    // TODO: notate move order

    // BOTTOM PART
    // TODO: render pictograms of caputred pieces (WIP)
    let mut x = bottom.x() + PADDING as i32;
    let y = bottom.y() + PADDING as i32;
    let mut h = 0;
    for piece in state.captured_white.iter() {
        let texture = sprites.get(piece).unwrap();
        let dimensions = texture.query();

        let target =
            Rect::new(x, y, dimensions.width * 2, dimensions.height * 2);
        canvas.copy(&texture, None, target).unwrap();
        x += (dimensions.width * 2) as i32 + PADDING as i32;
        h = dimensions.height * 2;
    }

    let y = bottom.y + bottom.height() as i32 - h as i32 - PADDING as i32;
    for piece in state.captured_black.iter() {
        let texture = sprites.get(piece).unwrap();
        let dimensions = texture.query();

        let target =
            Rect::new(x, y, dimensions.width * 2, dimensions.height * 2);
        canvas.copy(&texture, None, target).unwrap();
        x += (dimensions.width * 2) as i32 + PADDING as i32;
    }
}

fn render_winning_screen(
    canvas: &mut WindowCanvas,
    state: &GameState,
    bounds: Rect,
    font: &sdl2::ttf::Font,
    texture_creator: &sdl2::render::TextureCreator<sdl2::video::WindowContext>,
) {
    // fill background with border
    canvas.set_draw_color(AUX_COLOR);
    canvas.fill_rect(bounds).unwrap();

    let internal_bounds = Rect::new(
        bounds.x() + BORDER_WIDTH * 4,
        bounds.y() + BORDER_WIDTH * 4,
        bounds.width() - (BORDER_WIDTH * 8) as u32,
        bounds.height() - (BORDER_WIDTH * 8) as u32,
    );
    canvas.set_draw_color(BG_COLOR);
    canvas.fill_rect(internal_bounds).unwrap();

    let text = match state.checkmate {
        true => match state.whites_turn {
            false => "White won!",
            true => "Black won!",
        },
        false => "Draw!",
    };

    let surface = font.render(text).blended(FG_COLOR).unwrap();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .unwrap();
    let dimensions = texture.query();
    let target = Rect::new(
        internal_bounds.x() + internal_bounds.width() as i32 / 2
            - dimensions.width as i32 / 2,
        internal_bounds.y() + internal_bounds.height() as i32 / 2
            - dimensions.height as i32 / 2,
        dimensions.width,
        dimensions.height,
    );

    canvas.copy(&texture, None, target).unwrap();
}

// the main render method
fn render(
    canvas: &mut WindowCanvas,
    state: &GameState,
    textures: &HashMap<field::Figure, sdl2::render::Texture>,
    smallfont: &sdl2::ttf::Font,
    mediumfont: &sdl2::ttf::Font,
    bigfont: &sdl2::ttf::Font,
    texture_creator: &sdl2::render::TextureCreator<sdl2::video::WindowContext>,
) {
    let screen_size = Rect::new(0, 0, WIDTH, HEIGHT);
    let field_bounds = Rect::new(0, 0, HEIGHT, HEIGHT);
    let sidebar_bounds = Rect::new(HEIGHT as i32, 0, WIDTH - HEIGHT, HEIGHT);

    render_field(
        canvas,
        field_bounds,
        state,
        textures,
        smallfont,
        texture_creator,
    );

    render_sidebar(
        canvas,
        sidebar_bounds,
        state,
        mediumfont,
        smallfont,
        textures,
        texture_creator,
    );

    if state.checkmate || state.draw {
        let dialog_bounds =
            Rect::from_center(screen_size.center(), WIDTH / 2, HEIGHT / 3);
        render_winning_screen(
            canvas,
            state,
            dialog_bounds,
            bigfont,
            texture_creator,
        );
    }
    canvas.present();
}

fn load_sprites(
    texture_creator: &'_ sdl2::render::TextureCreator<
        sdl2::video::WindowContext,
    >,
) -> HashMap<field::Figure, sdl2::render::Texture> {
    let mut sprites = HashMap::new();

    let mut load_sprite = |path: &str, figure_type: field::Figure| {
        let sprite = texture_creator.load_texture(path).unwrap();
        sprites.insert(figure_type, sprite);
    };

    load_sprite(
        "resources/sprites/w_pawn.png",
        field::Figure::new(field::FigureColor::White, field::FigureType::Pawn),
    );
    load_sprite(
        "resources/sprites/w_rook.png",
        field::Figure::new(field::FigureColor::White, field::FigureType::Rook),
    );

    load_sprite(
        "resources/sprites/b_pawn.png",
        field::Figure::new(field::FigureColor::Black, field::FigureType::Pawn),
    );
    load_sprite(
        "resources/sprites/b_rook.png",
        field::Figure::new(field::FigureColor::Black, field::FigureType::Rook),
    );

    load_sprite(
        "resources/sprites/w_knight.png",
        field::Figure::new(
            field::FigureColor::White,
            field::FigureType::Knight,
        ),
    );

    load_sprite(
        "resources/sprites/b_knight.png",
        field::Figure::new(
            field::FigureColor::Black,
            field::FigureType::Knight,
        ),
    );

    load_sprite(
        "resources/sprites/w_bishop.png",
        field::Figure::new(
            field::FigureColor::White,
            field::FigureType::Bishop,
        ),
    );

    load_sprite(
        "resources/sprites/b_bishop.png",
        field::Figure::new(
            field::FigureColor::Black,
            field::FigureType::Bishop,
        ),
    );

    load_sprite(
        "resources/sprites/w_queen.png",
        field::Figure::new(field::FigureColor::White, field::FigureType::Queen),
    );

    load_sprite(
        "resources/sprites/b_queen.png",
        field::Figure::new(field::FigureColor::Black, field::FigureType::Queen),
    );

    load_sprite(
        "resources/sprites/w_king.png",
        field::Figure::new(field::FigureColor::White, field::FigureType::King),
    );

    load_sprite(
        "resources/sprites/b_king.png",
        field::Figure::new(field::FigureColor::Black, field::FigureType::King),
    );

    sprites
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let title = format!("Rusty Chess - {}", VERSION).to_owned();
    let window = video_subsystem
        .window(&title, WIDTH, HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    // setup rendering resources
    // TODO: figure out how to bundle these in a convenient struct
    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let texture_creator = canvas.texture_creator();
    let ttf_context = sdl2::ttf::init().unwrap();
    let lspr = load_sprites(&texture_creator);
    let lsmallfnt = ttf_context
        .load_font("resources/C64_Pro-STYLE.ttf", 24)
        .unwrap();
    let lmediumfnt = ttf_context
        .load_font("resources/C64_Pro-STYLE.ttf", 32)
        .unwrap();
    let lbigfnt = ttf_context
        .load_font("resources/C64_Pro-STYLE.ttf", 64)
        .unwrap();

    let mut state = GameState {
        field: field::Field::get_start_position(),
        whites_turn: true,
        captured_white: Vec::new(),
        captured_black: Vec::new(),
        marked: None,
        possible_moves: HashSet::new(),
        checkmate: false,
        draw: false,
    };

    //Main Loop
    let mut previous_buttons = HashSet::new();
    'running: loop {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        render(
            &mut canvas,
            &state,
            &lspr,
            &lsmallfnt,
            &lmediumfnt,
            &lbigfnt,
            &texture_creator,
        );

        // Main event handler
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        let mouse_state = event_pump.mouse_state();
        let buttons: HashSet<MouseButton> =
            mouse_state.pressed_mouse_buttons().collect();

        let new_buttons = &buttons - &previous_buttons;
        // This might be useful in the future for dragging pieces
        //let released_buttons = &previous_buttons - &buttons;

        if new_buttons.contains(&MouseButton::Left) {
            // Clicks
            let x = mouse_state.x();
            let y = mouse_state.y();

            if x < 0 || y < 0 {
                // ignore clicks outside the window
                continue;
            }

            let x = x as u32;
            let y = y as u32;

            // check if click is in field
            if x < HEIGHT && y < HEIGHT && !state.checkmate {
                let field_x: u32 = x / (HEIGHT / 8);
                let field_y: u32 = y / (HEIGHT / 8);

                let mut marked = Some((field_x, field_y));
                let players_color = if state.whites_turn {
                    field::FigureColor::White
                } else {
                    field::FigureColor::Black
                };

                let opponent_color = if state.whites_turn {
                    field::FigureColor::Black
                } else {
                    field::FigureColor::White
                };

                // if there is a figure on the clicked field, show possible moves
                if let Some(figure) = state.field.get(field_x, field_y) {
                    // if the clicked figure is not the current player's, ignore
                    let is_players_figure = match state.whites_turn {
                        true => figure.color == field::FigureColor::White,
                        false => figure.color == field::FigureColor::Black,
                    };
                    if is_players_figure {
                        state.possible_moves = state.field.get_possible_moves(
                            field_x,
                            field_y,
                            players_color,
                        );
                    } else {
                        if !state.possible_moves.contains(&(field_x, field_y)) {
                            marked = None;
                            state.possible_moves.clear();
                        }
                    }
                }
                if let Some((old_x, old_y)) = state.marked {
                    if state.possible_moves.contains(&(field_x, field_y)) {
                        // see if we are about to capture a figure
                        if let Some(figure) = state.field.get(field_x, field_y)
                        {
                            if figure.color == opponent_color {
                                if figure.color == field::FigureColor::White {
                                    state.captured_white.push(figure.clone());
                                } else {
                                    state.captured_black.push(figure.clone());
                                }
                            }
                        }

                        // move figure
                        state.field.move_figure(old_x, old_y, field_x, field_y);

                        marked = None;
                        state.whites_turn = !state.whites_turn;
                        state.possible_moves.clear();
                    }
                }

                // check for checkmate
                state.checkmate = state.field.is_checkmate(opponent_color);
                state.draw = state.field.is_draw();
                state.marked = marked;
            }
        }

        previous_buttons = buttons;

        canvas.present();
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
    }
}
