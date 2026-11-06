mod king_safety;
mod pawn_structure;

use crate::{
    game::{
        GameState,
        eval::{king_safety::king_safety, pawn_structure::pawn_structure},
    },
    types::{Color, PieceType},
};

#[derive(Copy, Clone, Debug)]
pub struct Evaluation {
    material: i32,
    pst: i32,
    bishop_pairs: i32,
    king_safety: i32,
    mobility: i32,
    passed_pawns: i32,
    isolated_pawns: i32,
    doubled_pawns: i32,
}

impl std::fmt::Display for Evaluation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Evaluation:\n\tmaterial: {},\n\tpst: {},\n\tbishop pairs: {},\n\tking safety: {}\n\tmobility: {}\n\tpassed pawns: {}\n\tisolated pawns: {}\n\tdoubled pawns: {}",
            self.material,
            self.pst,
            self.bishop_pairs,
            self.king_safety,
            self.mobility,
            self.passed_pawns,
            self.isolated_pawns,
            self.doubled_pawns
        )
    }
}

impl Evaluation {
    pub fn score(&self) -> i32 {
        self.material
            + self.pst
            + self.bishop_pairs
            + self.king_safety
            + self.mobility
            + self.passed_pawns
            + self.isolated_pawns
            + self.doubled_pawns
    }
}

impl GameState {
    pub fn eval(&self) -> Evaluation {
        let mut eval: Evaluation = Evaluation {
            material: 0,
            pst: 0,
            bishop_pairs: 0,
            king_safety: 0,
            mobility: 0,
            passed_pawns: 0,
            isolated_pawns: 0,
            doubled_pawns: 0,
        };

        eval.pst = self.pst_score;
        eval.mobility = self.mobility_score;

        let pawn = PieceType::Pawn;
        let knight = PieceType::Knight;
        let bishop = PieceType::Bishop;
        let rook = PieceType::Rook;
        let queen = PieceType::Queen;

        let wm = self.material_count[self.material_idx(pawn, Color::White)] as u16 * pawn.value()
            + self.material_count[self.material_idx(knight, Color::White)] as u16 * knight.value()
            + self.material_count[self.material_idx(bishop, Color::White)] as u16 * bishop.value()
            + self.material_count[self.material_idx(rook, Color::White)] as u16 * rook.value()
            + self.material_count[self.material_idx(queen, Color::White)] as u16 * queen.value();

        let bm = self.material_count[self.material_idx(pawn, Color::Black)] as u16 * pawn.value()
            + self.material_count[self.material_idx(knight, Color::Black)] as u16 * knight.value()
            + self.material_count[self.material_idx(bishop, Color::Black)] as u16 * bishop.value()
            + self.material_count[self.material_idx(rook, Color::Black)] as u16 * rook.value()
            + self.material_count[self.material_idx(queen, Color::Black)] as u16 * queen.value();

        eval.material = wm as i32 - bm as i32;

        if self.wb.count_ones() >= 2 {
            eval.bishop_pairs += 30;
        }
        if self.bb.count_ones() >= 2 {
            eval.bishop_pairs -= 30;
        }

        let material_on_board = wm + bm;

        eval.king_safety = king_safety(self, material_on_board);

        let pawn_structure = pawn_structure(self.wp, self.bp);
        eval.passed_pawns = pawn_structure.passed;
        eval.isolated_pawns = pawn_structure.isolated;
        eval.doubled_pawns = pawn_structure.doubled;

        eval
    }
}

#[cfg(test)]
mod tests {
    use crate::game::{GameState, MoveGenMode};
    #[test]
    fn test_mirror_position() {
        let position = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1";
        let mirror_position = "rnbqkbnr/pppp1ppp/8/4p3/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1";
        let gs = GameState::from_fen(position).unwrap();
        let mirror_gs = GameState::from_fen(mirror_position).unwrap();
        let eval = gs.eval();
        let mirror_eval = mirror_gs.eval();
        assert_eq!(eval.score(), -mirror_eval.score());
    }

    fn mirror_fen(fen: &str) -> String {
        let p: Vec<&str> = fen.split_whitespace().collect();
        let swap = |c: char| {
            if c.is_ascii_uppercase() {
                c.to_ascii_lowercase()
            } else if c.is_ascii_lowercase() {
                c.to_ascii_uppercase()
            } else {
                c
            }
        };
        let mut ranks: Vec<String> = p[0]
            .split('/')
            .map(|r| r.chars().map(swap).collect())
            .collect();
        ranks.reverse();
        let side = if p[1] == "w" { "b" } else { "w" };
        let castling: String = if p[2] == "-" {
            "-".into()
        } else {
            p[2].chars().map(swap).collect()
        };
        let ep = if p[3] == "-" {
            "-".into()
        } else {
            let f = p[3].as_bytes()[0] as char;
            let r = match p[3].as_bytes()[1] as char {
                '3' => '6',
                '6' => '3',
                other => other,
            };
            format!("{f}{r}")
        };
        format!("{} {} {} {} {} {}", ranks.join("/"), side, castling, ep, p[4], p[5])
    }

    #[test]
    fn test_eval_colour_symmetry() {
        // Mirror every position and require eval(p) == -eval(mirror(p)).
        // Covers castled kings, passed/doubled/isolated pawns, and a short
        // deterministic walk that exercises captures and promotions.
        let fens = [
            "4k3/8/8/3P4/3p4/8/8/4K3 w - - 0 1",
            "4k3/2p5/2p5/8/8/2P5/2P5/4K3 w - - 0 1",
            "4k3/p7/8/8/8/8/P7/4K3 w - - 0 1",
            "5rk1/5ppp/8/8/8/8/5PPP/5RK1 w - - 0 1",
            "2kr4/2pp4/8/8/8/8/2PP4/2KR4 w - - 0 1",
            "4k3/8/8/8/8/8/8/2B1KB2 w - - 0 1",
            "r3k2r/1p3ppp/8/3P4/3p4/8/1P3PPP/R3K2R w KQkq - 0 1",
            "r1bq1rk1/2p1bppp/p1np1n2/1p2p3/4P3/1BP2N2/PP1P1PPP/RNBQR1K1 w - - 0 1",
            "rnbqkb1r/1p2pppp/p2p1n2/8/3NP3/2N5/PPP2PPP/R1BQKB1R w KQkq - 0 1",
        ];
        let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut rng = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        let mut checked = 0;
        for fen in fens {
            let Ok(mut gs) = GameState::from_fen(fen) else { continue };
            for _ in 0..40 {
                let here = gs.to_fen();
                if let Ok(mgs) = GameState::from_fen(&mirror_fen(&here)) {
                    // Recompute both from FEN so this measures symmetry only,
                    // not incremental bookkeeping.
                    let fresh = GameState::from_fen(&here).unwrap();
                    let a = fresh.eval().score();
                    let b = mgs.eval().score();
                    checked += 1;
                    assert_eq!(a, -b, "colour asymmetry at {here}");
                }
                let moves = gs.legal_moves(MoveGenMode::All);
                if moves.is_empty() {
                    break;
                }
                let m = moves[(rng() as usize) % moves.len()];
                gs.make_move(m);
            }
        }
        assert!(checked > 0);
    }

    #[test]
    fn test_incremental_eval_matches_fresh() {
        // Special moves are the risky ones for incremental PST/mobility:
        // promotions (all four pieces), en passant and castling. After each
        // move the lazily-updated scores must equal a full recompute.
        let fens = [
            "r3k3/1P6/8/8/8/8/8/4K3 w - - 0 1",    // promotions, incl. capture
            "4k3/8/8/8/8/8/1p6/4K3 b - - 0 1",      // black promotion
            "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1",    // en passant
            "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1", // castling both sides
            "2kr4/2pp4/8/8/8/8/2PP4/2KR4 w - - 0 1",
            "r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1",
        ];
        for fen in fens {
            let mut gs = GameState::from_fen(fen).unwrap();
            let before = (gs.eval().score(), gs.pst_score, gs.mobility_score);
            assert!(
                !gs.legal_moves(MoveGenMode::All).is_empty(),
                "no moves: {fen}"
            );
            for m in gs.legal_moves(MoveGenMode::All) {
                let undo = gs.make_move(m);
                let here = gs.to_fen();
                let fresh = GameState::from_fen(&here).unwrap();
                assert_eq!(gs.pst_score, fresh.pst_score, "pst drift after {m:?} in {fen}");
                assert_eq!(
                    gs.mobility_score, fresh.mobility_score,
                    "mobility drift after {m:?} in {fen}"
                );
                assert_eq!(
                    gs.eval().score(),
                    fresh.eval().score(),
                    "eval drift after {m:?} in {fen}"
                );
                gs.unmake_move(undo);
                assert_eq!(
                    (gs.eval().score(), gs.pst_score, gs.mobility_score),
                    before,
                    "unmake mismatch after {m:?} in {fen}"
                );
            }
        }
    }

    fn exhaustive_walk(gs: &mut GameState, depth: u32, checks: &mut usize, fails: &mut usize) {
        let here = gs.to_fen();
        let snap = (
            gs.eval().score(),
            gs.pst_score,
            gs.mobility_score,
            gs.zobrist,
            gs.wp,
            gs.bp,
            gs.wb,
            gs.bb,
        );
        let fresh = GameState::from_fen(&here).unwrap();
        *checks += 1;
        let inc_state = (gs.pst_score, gs.mobility_score, gs.wp, gs.bp, gs.wb, gs.bb);
        let fresh_state = (fresh.pst_score, fresh.mobility_score, fresh.wp, fresh.bp, fresh.wb, fresh.bb);
        if inc_state != fresh_state || gs.eval().score() != fresh.eval().score() {
            *fails += 1;
            if *fails <= 5 {
                println!(
                    "INCREMENTAL MISMATCH {here}\n inc={inc_state:?} score={}\n fresh={fresh_state:?} score={}",
                    gs.eval().score(),
                    fresh.eval().score()
                );
            }
        }
        if let Ok(mgs) = GameState::from_fen(&mirror_fen(&here)) {
            if fresh.eval().score() != -mgs.eval().score() {
                *fails += 1;
                if *fails <= 5 {
                    println!(
                        "SYMMETRY MISMATCH {here}: {} vs {}",
                        fresh.eval().score(),
                        mgs.eval().score()
                    );
                }
            }
        }
        if depth == 0 {
            return;
        }
        for m in gs.legal_moves(MoveGenMode::All) {
            let undo = gs.make_move(m);
            exhaustive_walk(gs, depth - 1, checks, fails);
            gs.unmake_move(undo);
            let after = (
                gs.eval().score(),
                gs.pst_score,
                gs.mobility_score,
                gs.zobrist,
                gs.wp,
                gs.bp,
                gs.wb,
                gs.bb,
            );
            if gs.to_fen() != here || after != snap {
                *fails += 1;
                if *fails <= 5 {
                    println!("UNMAKE MISMATCH after {m:?} in {here}");
                }
            }
        }
    }

    #[test]
    fn test_exhaustive_incremental_and_symmetry() {
        // Exhaustive depth-limited trees: every node is checked for
        // incremental-vs-fresh agreement, colour symmetry, and exact undo.
        let cases = [
            ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 2),
            ("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 2),
            ("r3k3/1P6/8/8/8/8/8/4K3 w - - 0 1", 3),
            ("4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1", 3),
            ("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1", 3),
            ("2kr4/2pp4/8/8/8/8/2PP4/2KR4 w - - 0 1", 3),
        ];
        let mut checks = 0;
        let mut fails = 0;
        for (fen, depth) in cases {
            let mut gs = GameState::from_fen(fen).unwrap();
            exhaustive_walk(&mut gs, depth, &mut checks, &mut fails);
        }
        println!("exhaustive: {checks} nodes, {fails} failures");
        assert_eq!(fails, 0);
    }

    #[test]
    fn test_castled_mirror_position() {
        // Perfectly colour-symmetric castled position: both kings on the
        // g-file with identical pawn shields, so the eval must be 0. This is
        // the case the opening-position mirror test cannot reach, because
        // king safety only applies once a king is castled (g1/c1 or g8/c8).
        let gs = GameState::from_fen("5rk1/5ppp/8/8/8/8/5PPP/5RK1 w - - 0 1").unwrap();
        assert_eq!(gs.eval().score(), 0);
    }
}
