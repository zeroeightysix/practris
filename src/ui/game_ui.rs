use std::collections::VecDeque;

use arrayvec::ArrayVec;
use libtetris::*;
use nannou::draw::primitive::Rect as PRect;
use nannou::prelude::*;

use crate::game::{*, Event};
use crate::ui::skin::Skin;
use crate::util::RectExt;
use crate::wgpu::Texture;

pub struct SingleplayerGameUi {
    draw_state: GameDrawState,
    time: u32,
}

pub struct GameDrawState {
    board: ArrayVec<[ColoredRow; 40]>,
    state: State,
    statistics: Statistics,
    garbage_queue: u32,
    dead: bool,
    hold_piece: Option<Piece>,
    next_queue: VecDeque<Piece>,
    game_time: u32,
    combo_splash: Option<(u32, u32)>,
    back_to_back_splash: Option<u32>,
    clear_splash: Option<(&'static str, u32)>,
    name: String,
    skin: Skin
}

enum State {
    Falling(FallingPiece, FallingPiece),
    LineClearAnimation(ArrayVec<[i32; 4]>, i32),
    Delay,
}

impl SingleplayerGameUi {
    pub fn new(
        game: &Game,
        player_name: String,
        texture: Texture,
    ) -> Self {
        Self {
            draw_state: GameDrawState::new(
                game.board.next_queue(),
                player_name,
                texture,
            ),
            time: 0,
        }
    }

    pub fn update(
        &mut self,
        update: PlayerUpdate,
    ) {
        self.time += 1;
        self.draw_state.update(update, self.time);
    }

    pub fn draw(&self, draw: &Draw, rect: Rect) {
        let sq = letterbox(rect);
        self.draw_state.draw(draw, sq);
    }
}

impl GameDrawState {
    pub fn new(queue: impl IntoIterator<Item=Piece>, name: String, texture: Texture) -> Self {
        Self::new_from_board(ArrayVec::from([*ColoredRow::EMPTY; 40]), queue, name, texture)
    }

    pub fn new_from_board(board: ArrayVec<[ColoredRow; 40]>, queue: impl IntoIterator<Item=Piece>, name: String, texture: Texture) -> Self {
        Self {
            board,
            state: State::Delay,
            statistics: Statistics::default(),
            garbage_queue: 0,
            dead: false,
            hold_piece: None,
            next_queue: queue.into_iter().collect(),
            game_time: 0,
            combo_splash: None,
            back_to_back_splash: None,
            clear_splash: None,
            name,
            skin: Skin::Basic(texture)
        }
    }

    pub fn update(
        &mut self,
        update: PlayerUpdate,
        time: u32,
    ) {
        self.garbage_queue = update.garbage_queue;
        self.game_time = time;
        if let State::LineClearAnimation(_, ref mut frames) = self.state {
            *frames += 1;
        }
        if let Some((_, timer)) = &mut self.combo_splash {
            if *timer == 0 {
                self.combo_splash = None;
            } else {
                *timer -= 1;
            }
        }
        if let Some(timer) = &mut self.back_to_back_splash {
            if *timer == 0 {
                self.back_to_back_splash = None;
            } else {
                *timer -= 1;
            }
        }
        if let Some((_, timer)) = &mut self.clear_splash {
            if *timer == 0 {
                self.clear_splash = None;
            } else {
                *timer -= 1;
            }
        }
        for event in &update.events {
            match event {
                Event::PiecePlaced { piece, locked, .. } => {
                    self.statistics.update(&locked);
                    for &(x, y) in &piece.cells() {
                        self.board[y as usize].set(x as usize, piece.kind.0.color());
                    }
                    if locked.cleared_lines.is_empty() {
                        self.state = State::Delay;
                    } else {
                        self.state = State::LineClearAnimation(locked.cleared_lines.clone(), 0);
                    }
                    if locked.b2b {
                        self.back_to_back_splash = Some(75);
                    }
                    let combo = locked.combo.unwrap_or(0);
                    if combo > 0 {
                        self.combo_splash = Some((combo, 75));
                    }
                    if locked.perfect_clear {
                        self.clear_splash = Some(("Perfect Clear", 135));
                        self.back_to_back_splash = None;
                    } else if locked.placement_kind.is_hard() {
                        self.clear_splash = Some((locked.placement_kind.name(), 75));
                    }
                }
                Event::PieceHeld(piece) => {
                    self.hold_piece = Some(*piece);
                    self.state = State::Delay;
                }
                Event::PieceSpawned { new_in_queue } => {
                    self.next_queue.push_back(*new_in_queue);
                    self.next_queue.pop_front();
                }
                Event::PieceFalling(piece, ghost) => {
                    self.state = State::Falling(*piece, *ghost);
                }
                Event::EndOfLineClearDelay => {
                    self.state = State::Delay;
                    self.board.retain(|row| !row.is_full());
                    while !self.board.is_full() {
                        self.board.push(*ColoredRow::EMPTY);
                    }
                }
                Event::GarbageAdded(columns) => {
                    self.board.truncate(40 - columns.len());
                    for &col in columns {
                        let mut row = *ColoredRow::EMPTY;
                        for x in 0..10 {
                            if x != col {
                                row.set(x, CellColor::Garbage);
                            }
                        }
                        self.board.insert(0, row);
                    }
                }
                Event::GameOver => self.dead = true,
                _ => {}
            }
        }
    }

    pub fn draw(&self, draw: &Draw, rect: Rect) {
        const VIS_BOARD: usize = 20;

        let mino_size = (rect.h() / VIS_BOARD as f32).floor();

        let play_area = Rect::from_xy_wh(rect.xy(), Vec2::new(mino_size * 10., mino_size * VIS_BOARD as f32)).floor();
        draw.a::<PRect>(play_area.into())
            .color(BLACK);

        let bl = play_area.bottom_left();
        let mino_xy = |x: i32, y: i32| -> (f32, f32) {
            (bl.x + x as f32 * mino_size, bl.y + y as f32 * mino_size)
        };

        let board = &self.board;
        for y in 0..VIS_BOARD {
            let row = board[y];
            for x in 0..10 {
                let color = row.cell_color(x);
                let (x, y) = mino_xy(x as i32, y as i32);
                self.skin.draw_mino(&draw, color, x, y, mino_size);
            }
        }

        #[inline]
        fn cell_color_from_piece(piece: Piece) -> CellColor {
            match piece {
                Piece::I => CellColor::I,
                Piece::O => CellColor::O,
                Piece::T => CellColor::T,
                Piece::L => CellColor::L,
                Piece::J => CellColor::J,
                Piece::S => CellColor::S,
                Piece::Z => CellColor::Z,
            }
        }

        let draw_cells = |color: CellColor, cells: [(i32, i32); 4]| {
            for (x, y) in cells {
                let (x, y) = mino_xy(x, y);
                self.skin.draw_mino(&draw, color, x, y, mino_size);
            }
        };

        if let State::Falling(fall, ghost) = self.state {
            let color = cell_color_from_piece(fall.kind.0);
            draw_cells(CellColor::Unclearable, ghost.cells());
            draw_cells(color, fall.cells());
        }

        let draw_within = |piece: PieceState, rect: Rect| {
            let color = cell_color_from_piece(piece.0);
            let x_offset = match piece.0 {
                Piece::I | Piece::O => -1.,
                _ => -0.5,
            };
            for (x, y) in piece.cells() {
                self.skin.draw_mino(&draw, color, rect.x() + (x as f32 + x_offset) * mino_size, rect.y() + y as f32 * mino_size, mino_size);
            }
        };

        for (index, piece) in self.next_queue.iter().take(7).enumerate() {
            let piece = PieceState(*piece, RotationState::North);
            let rect = Rect::from_wh(Vec2::new(mino_size, mino_size) * 5.)
                .align_top_of(play_area)
                .right_of(play_area)
                .shift_y(index as f32 * -3. * mino_size);

            draw_within(piece, rect);
        }

        if let Some(piece) = self.hold_piece {
            let piece = PieceState(piece, RotationState::North);
            let rect = Rect::from_wh(Vec2::new(mino_size, mino_size) * 5.)
                .align_top_of(play_area)
                .left_of(play_area);

            draw_within(piece, rect);
        }
    }
}

fn letterbox(size: Rect) -> Rect {
    let d = size.w().min(size.h());
    Rect::from_xy_wh(size.xy(), Vec2::new(d, d))
}