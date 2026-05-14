//! Plan 10 step 4 — piece-intrinsic threat modifiers.
//!
//! Twelve modifier instances of the same struct (one per piece type)
//! plus one for Neutral-carrier passenger threats. Replaces the
//! single bridge `LegacyPieceAttacksModifier` from step 3.
//!
//! Each modifier scans the board, finds pieces of its kind, asks each
//! one for its threat set via `Piece::attacks`, and filters through
//! `Piece::would_capture_at`. Same logic the legacy bridge ran,
//! sliced per piece type so future step 6/7 train filters can compose
//! at higher priorities without re-implementing the iteration.
//!
//! **Trait method status:** `Piece::attacks` and
//! `Piece::would_capture_at` are kept on the trait — they're the
//! per-piece primitives these modifiers compose. The trait method
//! removal sketched in plan 10 is not landed (and likely never will
//! be — keeping the methods makes the per-piece overrides
//! discoverable from the piece's own file rather than the modifier
//! file).

use crate::board::Board;
use crate::movement::stack::{
    EventKindMask, MovementEffect, MovementEvent, MovementModifier,
};
use crate::pieces::Color;
use crate::pieces::piecetype::PieceType;

/// Discriminator for which piece type a `PieceAttacksModifier` handles.
/// One value per `PieceType` variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PieceKind {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
    Monkey,
    Goblin,
    Skibidi,
    Bus,
    Locomotive,
    Carriage,
}

impl PieceKind {
    fn matches(self, piece: &PieceType) -> bool {
        matches!(
            (self, piece),
            (Self::Pawn, PieceType::Pawn(_))
                | (Self::Rook, PieceType::Rook(_))
                | (Self::Knight, PieceType::Knight(_))
                | (Self::Bishop, PieceType::Bishop(_))
                | (Self::Queen, PieceType::Queen(_))
                | (Self::King, PieceType::King(_))
                | (Self::Monkey, PieceType::Monkey(_))
                | (Self::Goblin, PieceType::Goblin(_))
                | (Self::Skibidi, PieceType::Skibidi(_))
                | (Self::Bus, PieceType::Bus(_))
                | (Self::Locomotive, PieceType::Locomotive(_))
                | (Self::Carriage, PieceType::Carriage(_))
        )
    }
}

/// Generic piece-attacks modifier. One instance per piece type,
/// registered with a unique id and a `PieceKind` filter. All instances
/// share priority 40 — same-priority modifiers run in registration
/// order (stable sort).
pub struct PieceAttacksModifier {
    pub kind: PieceKind,
    pub id_str: &'static str,
}

impl MovementModifier for PieceAttacksModifier {
    fn id(&self) -> &'static str {
        self.id_str
    }

    fn priority(&self) -> u32 {
        40
    }

    fn touches(&self) -> EventKindMask {
        EventKindMask::THREAT_QUERY
    }

    fn apply(&self, board: &Board, event: &MovementEvent) -> MovementEffect {
        let MovementEvent::ThreatQuery {
            target,
            attacker_color,
        } = event
        else {
            return MovementEffect::Keep;
        };
        if *attacker_color == Color::Neutral {
            return MovementEffect::Keep;
        }
        let mut threats: Vec<MovementEvent> = Vec::new();
        for (coord, piece) in board.iter_pieces() {
            if !self.kind.matches(piece) {
                continue;
            }
            // Stranded pieces (on closed Gate / Block / Vent / Turret)
            // project no threats — mirrors WalkabilityFilter's source
            // check on the movement side. Without this, an inert
            // piece would still pin / give check / block castling.
            if !board.is_walkable_at(&coord) {
                continue;
            }
            let pc = piece.get_color();
            // Same colour-filter as the legacy iteration: attacker-
            // colour pieces, plus Neutral pieces (trains) whose own
            // movement threatens every side. Passenger threats are a
            // separate concern handled by
            // `NeutralCarrierPassengerThreatModifier`.
            if pc != *attacker_color && pc != Color::Neutral {
                continue;
            }
            for c in piece.attacks(board, &coord) {
                if &c == target && piece.would_capture_at(board, &coord, target) {
                    threats.push(MovementEvent::Threat {
                        attacker: coord.clone(),
                        attacker_piece: piece.clone(),
                        target: target.clone(),
                    });
                    break;
                }
            }
        }
        MovementEffect::Augment(threats)
    }
}

/// Neutral-carrier passenger threats: a Black king-passenger on a
/// Neutral cart threatens for Black only. Sits at priority 60 (just
/// above the piece-attack band so it sees the full threat picture
/// before any 100+ filters apply).
///
/// For colour-matched carriers (Bus) the passenger threats are
/// already covered by the cart's own `attacks` — boarding is same-
/// colour by invariant so the cart and its passengers share a side.
pub struct NeutralCarrierPassengerThreatModifier;

impl MovementModifier for NeutralCarrierPassengerThreatModifier {
    fn id(&self) -> &'static str {
        "piece_attacks.neutral_passenger"
    }

    fn priority(&self) -> u32 {
        60
    }

    fn touches(&self) -> EventKindMask {
        EventKindMask::THREAT_QUERY
    }

    fn apply(&self, board: &Board, event: &MovementEvent) -> MovementEffect {
        let MovementEvent::ThreatQuery {
            target,
            attacker_color,
        } = event
        else {
            return MovementEffect::Keep;
        };
        if *attacker_color == Color::Neutral {
            return MovementEffect::Keep;
        }
        let mut threats: Vec<MovementEvent> = Vec::new();
        for (coord, piece) in board.iter_pieces() {
            if piece.get_color() != Color::Neutral {
                continue;
            }
            // Stranded Neutral cart projects no passenger threats —
            // a parked carrier on a closed Gate must not surface its
            // king-passenger's threats and give spurious check.
            if !board.is_walkable_at(&coord) {
                continue;
            }
            let Some(passengers) = piece.passengers() else {
                continue;
            };
            for passenger in passengers {
                if passenger.get_color() != *attacker_color {
                    continue;
                }
                for c in passenger.attacks(board, &coord) {
                    if &c == target
                        && passenger.would_capture_at(board, &coord, target)
                    {
                        threats.push(MovementEvent::Threat {
                            attacker: coord.clone(),
                            attacker_piece: passenger.clone(),
                            target: target.clone(),
                        });
                        break;
                    }
                }
            }
        }
        MovementEffect::Augment(threats)
    }
}

/// All twelve piece-attack modifier instances + the Neutral-passenger
/// modifier, ready to register on the default stack.
pub fn all_piece_attack_modifiers() -> Vec<Box<dyn MovementModifier>> {
    use PieceKind::*;
    let kinds: &[(PieceKind, &'static str)] = &[
        (Pawn, "piece_attacks.pawn"),
        (Rook, "piece_attacks.rook"),
        (Knight, "piece_attacks.knight"),
        (Bishop, "piece_attacks.bishop"),
        (Queen, "piece_attacks.queen"),
        (King, "piece_attacks.king"),
        (Monkey, "piece_attacks.monkey"),
        (Goblin, "piece_attacks.goblin"),
        (Skibidi, "piece_attacks.skibidi"),
        (Bus, "piece_attacks.bus"),
        (Locomotive, "piece_attacks.locomotive"),
        (Carriage, "piece_attacks.carriage"),
    ];
    let mut out: Vec<Box<dyn MovementModifier>> = kinds
        .iter()
        .map(|(kind, id_str)| {
            Box::new(PieceAttacksModifier {
                kind: *kind,
                id_str,
            }) as Box<dyn MovementModifier>
        })
        .collect();
    out.push(Box::new(NeutralCarrierPassengerThreatModifier));
    out
}
