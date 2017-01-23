use std::result;

use game::*;
use game::frontends::ansi::resolve_tile;
use frontends::ansi::{self, ComplexTile, SimpleTile, AnsiColour, Style};
use coord::Coord;
use colour::Rgb24;

const MOON_COLOUR: ansi::AnsiColour = ansi::colours::MAGENTA;
const AIM_LINE_COLOUR: ansi::AnsiColour = ansi::colours::YELLOW;

const ANSI_GAME_WINDOW_X: usize = 1;
const ANSI_GAME_WINDOW_Y: usize = 1;

const MESSAGE_LOG_NUM_LINES: usize = 4;
const MESSAGE_LOG_PADDING_TOP: usize = 1;
const MESSAGE_LOG_PLAIN_COLOUR: Rgb24 = Rgb24 { red: 255, green: 255, blue: 255 };

const SCROLL_BAR_COLOUR: Rgb24 = Rgb24 { red: 255, green: 255, blue: 255 };

struct AnsiInfo {
    ch: char,
    fg: AnsiColour,
    bg: AnsiColour,
    style: Style,
}

impl Default for AnsiInfo {
    fn default() -> Self {
        AnsiInfo {
            bg: ansi::colours::DARK_GREY,
            fg: ansi::colours::DARK_GREY,
            ch: ' ',
            style: ansi::styles::NONE,
        }
    }
}

pub struct AnsiKnowledgeRenderer {
    window_allocator: Box<ansi::WindowAllocator>,
    window: ansi::Window,
    buffer: TileBuffer,
    scroll: bool,
    scroll_position: Coord,
    message_log_window: ansi::Window,
    message_log: Vec<Message>,
    total_width: usize,
    total_height: usize,
    top_left: Coord,
}

pub enum AnsiKnowledgeRendererError {
    TerminalTooSmall {
        min_width: usize,
        min_height: usize,
    },
}

impl AnsiKnowledgeRenderer {
    pub fn new(window_allocator: Box<ansi::WindowAllocator>,
               game_width: usize,
               game_height: usize,
               scroll: bool) -> result::Result<Self, AnsiKnowledgeRendererError> {

        if window_allocator.width() <= game_width || window_allocator.height() <= game_height {
            return Err(AnsiKnowledgeRendererError::TerminalTooSmall {
                min_width: game_width + ANSI_GAME_WINDOW_X,
                min_height: game_height + ANSI_GAME_WINDOW_Y,
            });
        }

        let window = window_allocator.make_window(
            ANSI_GAME_WINDOW_X as isize,
            ANSI_GAME_WINDOW_Y as isize,
            game_width,
            game_height,
            ansi::BufferType::Double);

        let message_log_window = window_allocator.make_window(
            ANSI_GAME_WINDOW_X as isize,
            (ANSI_GAME_WINDOW_Y + game_height + MESSAGE_LOG_PADDING_TOP) as isize,
            game_width,
            MESSAGE_LOG_NUM_LINES,
            ansi::BufferType::Double);

        let mut message_log = Vec::new();
        for _ in 0..MESSAGE_LOG_NUM_LINES {
            message_log.push(Message::new());
        }

        window_allocator.fill(ansi::colours::BLACK);
        window_allocator.flush();

        Ok(AnsiKnowledgeRenderer {
            window_allocator: window_allocator,
            window: window,
            buffer: TileBuffer::new(game_width, game_height),
            scroll: scroll,
            scroll_position: Coord::new(0, 0),
            message_log_window: message_log_window,
            message_log: message_log,
            total_width: game_width,
            total_height: game_height + MESSAGE_LOG_PADDING_TOP + MESSAGE_LOG_NUM_LINES,
            top_left: Coord::new(ANSI_GAME_WINDOW_X as isize, ANSI_GAME_WINDOW_Y as isize),
        })
    }

    fn simple_tile(tile: ComplexTile, is_front: bool) -> SimpleTile {
        match tile {
            ComplexTile::Simple(s) => s,
            ComplexTile::Wall { front, back } => {
                if is_front {
                    front
                } else {
                    back
                }
            }
        }
    }

    fn to_ansi_info(cell: &CellDrawInfo) -> AnsiInfo {
        let mut info = AnsiInfo::default();

        if let Some(bg_tile_type) = cell.background {
            let bg_tile = resolve_tile::resolve_tile(bg_tile_type);
            let tile = Self::simple_tile(bg_tile, cell.front);
            if let Some(c) = tile.background_colour() {
                info.bg = c;
            }
            if let Some(c) = tile.character() {
                info.ch = c;
            }
            if let Some(s) = tile.style() {
                info.style = s;
            }
        }

        if let Some(fg_tile_type) = cell.foreground {
            let fg_tile = resolve_tile::resolve_tile(fg_tile_type);
            let tile = Self::simple_tile(fg_tile, cell.front);
            if let Some(c) = tile.foreground_colour() {
                info.fg = c;
            }
            if let Some(c) = tile.character() {
                info.ch = c;
            }
            if let Some(s) = tile.style() {
                info.style = s;
            }
        }

        if cell.moon {
            info.bg = MOON_COLOUR;
        }

        if !cell.visible {
            info.fg = ansi::colours::BLACK;
            info.bg = ansi::colours::DARK_GREY;
        }

        info
    }

    fn draw_internal(&mut self) {
        for (coord, cell) in izip!(self.buffer.coord_iter(), self.buffer.iter()) {
            let info = Self::to_ansi_info(cell);
            self.window.get_cell(coord.x, coord.y).set(info.ch, info.fg, info.bg, info.style);
        }
    }

    fn draw_overlay_internal(&mut self, overlay: &RenderOverlay) {
        if let Some(ref aim_line) = overlay.aim_line {
            for coord in aim_line.iter() {
                let screen_coord = self.world_to_screen(coord);
                if let Some(cell) = self.buffer.get(screen_coord) {
                    let mut info = Self::to_ansi_info(cell);
                    info.bg = AIM_LINE_COLOUR;
                    self.window.get_cell(screen_coord.x, screen_coord.y).set(info.ch, info.fg, info.bg, info.style);
                }
            }
        }
    }

    fn render_message_part(window: &mut ansi::Window, part: &MessagePart, mut cursor: Coord) -> Coord {
        let (colour, string) = match part {
            &MessagePart::Plain(ref s) => (MESSAGE_LOG_PLAIN_COLOUR, s),
            &MessagePart::Colour(c, ref s) => (c, s),
        };

        let ansi_colour = ansi::AnsiColour::new_from_rgb24(colour);

        for ch in string.chars() {
            if cursor.x >= window.width() as isize {
                break;
            }

            window.get_cell(cursor.x, cursor.y).set(ch, ansi_colour, ansi::colours::DARK_GREY, ansi::styles::NONE);

            cursor.x += 1;
        }

        cursor
    }

    fn render_message(window: &mut ansi::Window, message: &Message, mut cursor: Coord) -> Coord {
        for part in message {
            cursor = Self::render_message_part(window, part, cursor);
        }

        while cursor.x < window.width() as isize {
            window.get_cell(cursor.x, cursor.y).set(' ', ansi::colours::DARK_GREY, ansi::colours::DARK_GREY, ansi::styles::NONE);

            cursor.x += 1;
        }

        cursor.x = 0;
        cursor.y += 1;

        cursor
    }

    fn draw_message_log_internal(&mut self) {

        let mut cursor = Coord::new(0, 0);

        for line in &self.message_log {
            cursor = Self::render_message(&mut self.message_log_window, line, cursor);
       }
    }

    fn scroll_bar(&self, num_messages: usize, offset: usize) -> Option<(Coord, usize)> {
        let num_lines = self.display_log_num_lines();
        if num_messages > num_lines {
            let scroll_bar_height = (self.total_height * num_lines) / num_messages;
            let remaining = self.total_height - scroll_bar_height;
            let max_offset = num_messages - num_lines;
            let scroll_bar_top = remaining - ((offset * remaining) / max_offset);

            Some((Coord::new(self.total_width as isize - 1, scroll_bar_top as isize), scroll_bar_height))
        } else {
            None
        }
    }
}

impl KnowledgeRenderer for AnsiKnowledgeRenderer {

    fn width(&self) -> usize {
        self.window.width()
    }

    fn height(&self) -> usize {
        self.window.height()
    }

    fn world_offset(&self) -> Coord {
        self.scroll_position
    }

    fn update(&mut self, knowledge: &DrawableKnowledgeLevel, turn_id: u64, position: Coord) {
        let scroll_position = if self.scroll {
            self.centre_offset(position)
        } else {
            Coord::new(0, 0)
        };

        self.scroll_position = self.buffer.update(knowledge, turn_id, scroll_position);
    }

    fn draw(&mut self) {
        self.draw_internal();
        self.window.flush();
        self.draw_message_log_internal();
        self.message_log_window.flush();
    }

    fn draw_with_overlay(&mut self, overlay: &RenderOverlay) {
        self.draw_internal();
        self.draw_overlay_internal(overlay);
        self.window.flush();
        self.draw_message_log_internal();
        self.message_log_window.flush();
    }

    fn update_log(&mut self, messages: &MessageLog, language: &Box<Language>) {
        for (log_entry, message) in izip!(messages.tail(MESSAGE_LOG_NUM_LINES), &mut self.message_log) {
            language.translate_repeated(log_entry.message, log_entry.repeated, message);
        }
    }

    fn display_log(&mut self, message_log: &MessageLog, offset: usize, language: &Box<Language>) {

        let mut window = self.window_allocator.make_window(self.top_left.x, self.top_left.y,
                                                                  self.total_width, self.total_height,
                                                                  ansi::BufferType::Double);
        let mut cursor = Coord::new(0, 0);
        let mut message = Message::new();
        let messages = message_log.tail_with_offset(self.display_log_num_lines(), offset);
        for log_entry in messages {
            language.translate_repeated(log_entry.message, log_entry.repeated, &mut message);
            cursor = Self::render_message(&mut window, &message, cursor);
        }

        message.clear();
        while cursor.y < self.display_log_num_lines() as isize {
            cursor = Self::render_message(&mut window, &message, cursor);
        }

        if let Some((position, size)) = self.scroll_bar(message_log.len(), offset) {
            let scroll_bar_colour = ansi::AnsiColour::new_from_rgb24(SCROLL_BAR_COLOUR);
            for i in 0..(size as isize) {
                let coord = position + Coord::new(0, i);
                window.get_cell(coord.x, coord.y).set(' ', scroll_bar_colour, scroll_bar_colour, ansi::styles::NONE);
            }
        }

        window.flush();
        self.window_allocator.fill(ansi::colours::BLACK);
        window.delete();
    }

    fn display_log_num_lines(&self) -> usize {
        self.total_height
    }
}
