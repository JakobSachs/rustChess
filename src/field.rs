use std::collections::HashSet;

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum FigureType {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum FigureColor {
    White,
    Black,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Figure {
    pub color: FigureColor,
    pub figure_type: FigureType,
}

impl Figure {
    pub fn new(color: FigureColor, figure_type: FigureType) -> Self {
        Self { color, figure_type }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Field {
    pub figures: [Option<Figure>; 64],
}

impl Field {
    pub fn new() -> Self {
        Self {
            figures: [None; 64],
        }
    }

    pub fn get(&self, x: u32, y: u32) -> Option<&Figure> {
        self.figures.get((x + y * 8) as usize).unwrap().as_ref()
    }

    pub fn set(&mut self, x: u32, y: u32, figure: Figure) {
        self.figures[(x + y * 8) as usize] = Some(figure);
    }

    pub fn move_figure(
        &mut self,
        from_x: u32,
        from_y: u32,
        to_x: u32,
        to_y: u32,
    ) {
        let figure = self.get(from_x, from_y).unwrap().clone();
        self.set(to_x, to_y, figure);
        self.figures[(from_x + from_y * 8) as usize] = None;
    }

    #[inline]
    pub fn get_start_position() -> Self {
        let mut field = Self::new();
        for i in 0..8 {
            field.set(i, 6, Figure::new(FigureColor::White, FigureType::Pawn));
            field.set(i, 1, Figure::new(FigureColor::Black, FigureType::Pawn));
        }
        field.set(0, 7, Figure::new(FigureColor::White, FigureType::Rook));
        field.set(7, 7, Figure::new(FigureColor::White, FigureType::Rook));
        field.set(0, 0, Figure::new(FigureColor::Black, FigureType::Rook));
        field.set(7, 0, Figure::new(FigureColor::Black, FigureType::Rook));
        field.set(1, 7, Figure::new(FigureColor::White, FigureType::Knight));
        field.set(6, 7, Figure::new(FigureColor::White, FigureType::Knight));
        field.set(1, 0, Figure::new(FigureColor::Black, FigureType::Knight));
        field.set(6, 0, Figure::new(FigureColor::Black, FigureType::Knight));
        field.set(2, 7, Figure::new(FigureColor::White, FigureType::Bishop));
        field.set(5, 7, Figure::new(FigureColor::White, FigureType::Bishop));
        field.set(2, 0, Figure::new(FigureColor::Black, FigureType::Bishop));
        field.set(5, 0, Figure::new(FigureColor::Black, FigureType::Bishop));
        field.set(3, 7, Figure::new(FigureColor::White, FigureType::Queen));
        field.set(3, 0, Figure::new(FigureColor::Black, FigureType::Queen));
        field.set(4, 7, Figure::new(FigureColor::White, FigureType::King));
        field.set(4, 0, Figure::new(FigureColor::Black, FigureType::King));
        field
    }

    pub fn is_check(&self, color: FigureColor) -> bool {
        let king_pos = self
            .figures
            .iter()
            .enumerate()
            .find(|(_, f)| {
                f.as_ref().map_or(false, |f| {
                    f.color == color && f.figure_type == FigureType::King
                })
            })
            .unwrap()
            .0;
        let king_x = king_pos as u32 % 8;
        let king_y = king_pos as u32 / 8;
        let enemy_color = match color {
            FigureColor::White => FigureColor::Black,
            FigureColor::Black => FigureColor::White,
        };
        for x in 0..8 {
            for y in 0..8 {
                if let None = self.get(x, y) {
                    continue;
                }

                let figure = self.get(x, y).unwrap();
                if figure.color != enemy_color {
                    continue;
                }

                let moves = self.get_naive_moves(x, y);
                if moves.contains(&(king_x, king_y)) {
                    return true;
                }
            }
        }
        false
    }
    // do naive moves first

    fn get_naive_moves(&self, x: u32, y: u32) -> HashSet<(u32, u32)> {
        match self.get(x, y) {
            Some(figure) => match figure.figure_type {
                FigureType::Pawn => {
                    let mut moves = HashSet::new();
                    if figure.color == FigureColor::White {
                        // normal pawn movement
                        if self.get(x, y - 1).is_none() {
                            moves.insert((x, y - 1));
                            if y == 6 && self.get(x, y - 2).is_none() {
                                moves.insert((x, y - 2));
                            }
                        }

                        // taking diagonally
                        if x > 0 && self.get(x - 1, y - 1).is_some() {
                            moves.insert((x - 1, y - 1));
                        }
                        if x < 7 && self.get(x + 1, y - 1).is_some() {
                            moves.insert((x + 1, y - 1));
                        }
                    } else {
                        // normal pawn movement
                        if self.get(x, y + 1).is_none() {
                            moves.insert((x, y + 1));
                            if y == 1 && self.get(x, y + 2).is_none() {
                                moves.insert((x, y + 2));
                            }
                        }

                        // taking diagonally
                        if x > 0 && self.get(x - 1, y + 1).is_some() {
                            moves.insert((x - 1, y + 1));
                        }
                        if x < 7 && self.get(x + 1, y + 1).is_some() {
                            moves.insert((x + 1, y + 1));
                        }
                    }
                    let result = moves
                        .into_iter()
                        .filter(|(x, y)| {
                            self.get(*x, *y).is_none()
                                || self.get(*x, *y).unwrap().color
                                    != figure.color
                        })
                        .collect();

                    result
                }
                // Do Rook moves
                FigureType::Rook => {
                    let mut moves = HashSet::new();
                    // check horiziontal moves
                    for i in x + 1..8 {
                        moves.insert((i, y));
                        if self.get(i, y).is_some() {
                            break;
                        }
                    }

                    for i in (0..x).rev() {
                        moves.insert((i, y));
                        if self.get(i, y).is_some() {
                            break;
                        }
                    }

                    // check vertical moves
                    for i in y + 1..8 {
                        moves.insert((x, i));
                        if self.get(x, i).is_some() {
                            break;
                        }
                    }
                    for i in (0..y).rev() {
                        moves.insert((x, i));
                        if self.get(x, i).is_some() {
                            break;
                        }
                    }

                    let result = moves
                        .into_iter()
                        .filter(|(x, y)| {
                            self.get(*x, *y).is_none()
                                || self.get(*x, *y).unwrap().color
                                    != figure.color
                        })
                        .collect();

                    result
                }
                FigureType::Knight => {
                    let mut moves = Vec::new();

                    // blindly insert all possible moves
                    if x > 0 && y > 1 {
                        moves.push((x - 1, y - 2));
                    }
                    if x > 1 && y > 0 {
                        moves.push((x - 2, y - 1));
                    }
                    if x > 1 && y < 7 {
                        moves.push((x - 2, y + 1));
                    }
                    if x > 0 && y < 6 {
                        moves.push((x - 1, y + 2));
                    }
                    if x < 7 && y < 6 {
                        moves.push((x + 1, y + 2));
                    }
                    if x < 6 && y < 7 {
                        moves.push((x + 2, y + 1));
                    }
                    if x < 6 && y > 0 {
                        moves.push((x + 2, y - 1));
                    }
                    if x < 7 && y > 1 {
                        moves.push((x + 1, y - 2));
                    }

                    // filter out moves that are not possible, cuz there is a figure of the same color
                    //
                    let result = moves
                        .into_iter()
                        .filter(|(x, y)| {
                            self.get(*x, *y).is_none()
                                || self.get(*x, *y).unwrap().color
                                    != figure.color
                        })
                        .collect();
                    result
                }
                FigureType::Bishop => {
                    let mut moves = HashSet::new();

                    // check diagonal moves
                    for i in 1..8 {
                        if x + i < 8 && y + i < 8 {
                            moves.insert((x + i, y + i));
                            if self.get(x + i, y + i).is_some() {
                                break;
                            }
                        }
                    }
                    for i in 1..8 {
                        if x + i < 8 && y >= i {
                            moves.insert((x + i, y - i));
                            if self.get(x + i, y - i).is_some() {
                                break;
                            }
                        }
                    }
                    for i in 1..8 {
                        if x >= i && y + i < 8 {
                            moves.insert((x - i, y + i));
                            if self.get(x - i, y + i).is_some() {
                                break;
                            }
                        }
                    }
                    for i in 1..8 {
                        if x >= i && y >= i {
                            moves.insert((x - i, y - i));
                            if self.get(x - i, y - i).is_some() {
                                break;
                            }
                        }
                    }

                    let result = moves
                        .into_iter()
                        .filter(|(x, y)| {
                            self.get(*x, *y).is_none()
                                || self.get(*x, *y).unwrap().color
                                    != figure.color
                        })
                        .collect();
                    result
                }
                FigureType::Queen => {
                    let mut moves = HashSet::new();

                    // check horiziontal moves
                    for i in x + 1..8 {
                        moves.insert((i, y));
                        if self.get(i, y).is_some() {
                            break;
                        }
                    }

                    for i in (0..x).rev() {
                        moves.insert((i, y));
                        if self.get(i, y).is_some() {
                            break;
                        }
                    }

                    // check vertical moves
                    for i in y + 1..8 {
                        moves.insert((x, i));
                        if self.get(x, i).is_some() {
                            break;
                        }
                    }
                    for i in (0..y).rev() {
                        moves.insert((x, i));
                        if self.get(x, i).is_some() {
                            break;
                        }
                    }

                    // check diagonal moves
                    for i in 1..8 {
                        if x + i < 8 && y + i < 8 {
                            moves.insert((x + i, y + i));
                            if self.get(x + i, y + i).is_some() {
                                break;
                            }
                        }
                    }
                    for i in 1..8 {
                        if x + i < 8 && y >= i {
                            moves.insert((x + i, y - i));
                            if self.get(x + i, y - i).is_some() {
                                break;
                            }
                        }
                    }
                    for i in 1..8 {
                        if x >= i && y + i < 8 {
                            moves.insert((x - i, y + i));
                            if self.get(x - i, y + i).is_some() {
                                break;
                            }
                        }
                    }
                    for i in 1..8 {
                        if x >= i && y >= i {
                            moves.insert((x - i, y - i));
                            if self.get(x - i, y - i).is_some() {
                                break;
                            }
                        }
                    }

                    let result = moves
                        .into_iter()
                        .filter(|(x, y)| {
                            self.get(*x, *y).is_none()
                                || self.get(*x, *y).unwrap().color
                                    != figure.color
                        })
                        .collect();
                    result
                }
                FigureType::King => {
                    let mut moves = HashSet::new();

                    // check all possible moves
                    if x > 0 && y > 0 {
                        moves.insert((x - 1, y - 1));
                    }

                    if x > 0 {
                        moves.insert((x - 1, y));
                    }

                    if x > 0 && y < 7 {
                        moves.insert((x - 1, y + 1));
                    }

                    if y > 0 {
                        moves.insert((x, y - 1));
                    }

                    if y < 7 {
                        moves.insert((x, y + 1));
                    }

                    if x < 7 && y > 0 {
                        moves.insert((x + 1, y - 1));
                    }

                    if x < 7 {
                        moves.insert((x + 1, y));
                    }

                    if x < 7 && y < 7 {
                        moves.insert((x + 1, y + 1));
                    }

                    // filter out moves that are not possible, cuz there is a figure of the same color

                    let result = moves
                        .into_iter()
                        .filter(|(x, y)| {
                            self.get(*x, *y).is_none()
                                || self.get(*x, *y).unwrap().color
                                    != figure.color
                        })
                        .collect();
                    result
                }
            },
            None => HashSet::new(),
        }
    }

    pub fn get_possible_moves(
        &self,
        x: u32,
        y: u32,
        color: FigureColor,
    ) -> HashSet<(u32, u32)> {
        let naive_moves = self.get_naive_moves(x, y);
        let mut moves = HashSet::new();

        // filter out moves that are not possible, because of a check situation

        for (x_m, y_m) in naive_moves.clone() {
            let mut board = self.clone();
            board.move_figure(x, y, x_m, y_m);

            if !board.is_check(color) {
                moves.insert((x_m, y_m));
            }
        }

        // TODO: Add special moves like castling, en passant, promotion

        moves
    }

    pub fn is_checkmate(&self, color: FigureColor) -> bool {
        let mut is_checkmate = true;

        for x in 0..8 {
            for y in 0..7 {
                if self.get(x, y).is_some()
                    && self.get(x, y).unwrap().color == color
                {
                    if !self.get_possible_moves(x, y, color).is_empty() {
                        is_checkmate = false;
                        break;
                    }
                }
            }
        }

        is_checkmate && !self.is_draw()
    }

    pub fn is_draw(&self) -> bool {
        let mut is_draw = true;

        for x in 0..8 {
            for y in 0..7 {
                if self.get(x, y).is_some() {
                    if !self
                        .get_possible_moves(x, y, self.get(x, y).unwrap().color)
                        .is_empty()
                    {
                        is_draw = false;
                        break;
                    }
                }
            }
        }

        is_draw
    }
}
