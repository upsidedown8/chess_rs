#[repr(usize)]
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Color {
    White = 1,
    Black = 0,
}

impl Color {
    pub fn enemy(&self) -> Color {
        match *self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    pub fn is_white(&self) -> bool {
        *self == Color::White
    }
    pub fn is_black(&self) -> bool {
        !self.is_white()
    }

    pub fn as_letter(&self) -> char {
        match *self {
            Color::White => 'w',
            Color::Black => 'b'
        }
    }

    #[inline(always)]
    pub fn idx(&self) -> usize {
        *self as usize
    }
}

#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Pieces {
    WhitePawn    = 0,
    WhiteKnight  = 1,
    WhiteBishop  = 2,
    WhiteRook    = 3,
    WhiteQueen   = 4,
    WhiteKing    = 5,
    BlackPawn    = 6,
    BlackKnight  = 7,
    BlackBishop  = 8,
    BlackRook    = 9,
    BlackQueen   = 10,
    BlackKing    = 11,
}

impl Pieces {
    pub fn color(&self) -> Color {
        match *self {
            Pieces::WhitePawn | 
            Pieces::WhiteKnight | 
            Pieces::WhiteBishop | 
            Pieces::WhiteRook | 
            Pieces::WhiteQueen | 
            Pieces::WhiteKing => Color::White,
            
            Pieces::BlackPawn | 
            Pieces::BlackKnight | 
            Pieces::BlackBishop | 
            Pieces::BlackRook | 
            Pieces::BlackQueen | 
            Pieces::BlackKing => Color::Black,
        }
    }
    pub fn enemy_color(&self) -> Color {
        self.color().enemy()
    }
    pub fn enemy_piece(&self) -> Pieces {
        match *self {
            Pieces::WhitePawn   => Pieces::BlackPawn, 
            Pieces::WhiteKnight => Pieces::BlackKnight,
            Pieces::WhiteBishop => Pieces::BlackBishop,
            Pieces::WhiteRook   => Pieces::BlackRook,
            Pieces::WhiteQueen  => Pieces::BlackQueen,
            Pieces::WhiteKing   => Pieces::BlackKing,
            
            Pieces::BlackPawn   => Pieces::WhitePawn,
            Pieces::BlackKnight => Pieces::WhiteKnight, 
            Pieces::BlackBishop => Pieces::WhiteBishop,
            Pieces::BlackRook   => Pieces::WhiteRook,
            Pieces::BlackQueen  => Pieces::WhiteQueen,
            Pieces::BlackKing   => Pieces::WhiteKing,
        }
    }
    
    pub fn notation(&self) -> char {
        match *self {
            Pieces::WhitePawn   => 'P', 
            Pieces::WhiteKnight => 'N',
            Pieces::WhiteBishop => 'B',
            Pieces::WhiteRook   => 'R',
            Pieces::WhiteQueen  => 'Q',
            Pieces::WhiteKing   => 'K',
            
            Pieces::BlackPawn   => 'p',
            Pieces::BlackKnight => 'n', 
            Pieces::BlackBishop => 'b',
            Pieces::BlackRook   => 'r',
            Pieces::BlackQueen  => 'q',
            Pieces::BlackKing   => 'k',
        }
    }

    #[inline(always)]
    pub fn idx(&self) -> usize {
        *self as usize
    }

    pub fn is_pawn(&self) -> bool {
        matches!(*self, Pieces::WhitePawn | Pieces::BlackPawn)
    }
    pub fn is_knight(&self) -> bool {
        matches!(*self, Pieces::WhiteKnight | Pieces::BlackKnight)
    }
    pub fn is_bishop(&self) -> bool {
        matches!(*self, Pieces::WhiteBishop | Pieces::BlackBishop)
    }
    pub fn is_rook(&self) -> bool {
        matches!(*self, Pieces::WhiteRook | Pieces::BlackRook)
    }
    pub fn is_queen(&self) -> bool {
        matches!(*self, Pieces::WhiteQueen | Pieces::BlackQueen)
    }
    pub fn is_king(&self) -> bool {
        matches!(*self, Pieces::WhiteKing | Pieces::BlackKing)
    }

    pub fn pawn(color: Color) -> Pieces {
        match color {
            Color::White => Pieces::WhitePawn,
            Color::Black => Pieces::BlackPawn,
        }
    }
    pub fn knight(color: Color) -> Pieces {
        match color {
            Color::White => Pieces::WhiteKnight,
            Color::Black => Pieces::BlackKnight,
        }
    }
    pub fn bishop(color: Color) -> Pieces {
        match color {
            Color::White => Pieces::WhiteBishop,
            Color::Black => Pieces::BlackBishop,
        }
    }
    pub fn rook(color: Color) -> Pieces {
        match color {
            Color::White => Pieces::WhiteRook,
            Color::Black => Pieces::BlackRook,
        }
    }
    pub fn queen(color: Color) -> Pieces {
        match color {
            Color::White => Pieces::WhiteQueen,
            Color::Black => Pieces::BlackQueen,
        }
    }
    pub fn king(color: Color) -> Pieces {
        match color {
            Color::White => Pieces::WhiteKing,
            Color::Black => Pieces::BlackKing,
        }
    }
}
