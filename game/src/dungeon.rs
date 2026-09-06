//! Dungeon vault loop (single-player interiors): deterministic foe spawns
//! per dungeon tile + floor, and the one-time vault reward.
//!
//! The renderer keeps `Enemy` foes in *room* coordinates while inside and
//! drives them with the regular `Enemy::update` (chase/windup/contact), so
//! dungeon fights feel exactly like surface fights in miniature.

use crate::enemy::EnemyKind;
use crate::items::ItemKind;

/// Foe spawns for a dungeon at world tile `(bx, by)`, per floor, in room
/// coordinates (relative to the room center the renderer draws around).
/// Floor 1 is patrolled; floor 2 guards the vault. Deterministic per tile.
pub fn dungeon_foes(bx: i32, by: i32, floor: u8) -> Vec<(EnemyKind, f32, f32)> {
    let h = ((bx as u32).wrapping_mul(73856093) ^ (by as u32).wrapping_mul(19349663)).wrapping_add(floor as u32 * 7919);
    let pick = |i: u32| (h.wrapping_add(i.wrapping_mul(2654435761)) >> 16) % 100;
    let spot = |i: u32, dx: f32, dy: f32| {
        let jx = ((pick(i) % 7) as f32 - 3.0) * 0.3;
        let jy = ((pick(i + 40) % 7) as f32 - 3.0) * 0.3;
        (dx + jx, dy + jy)
    };
    match floor {
        1 => {
            // Two bats patrolling the entry hall.
            let (x1, y1) = spot(1, -1.0, -0.5);
            let (x2, y2) = spot(2, 1.0, 0.5);
            vec![(EnemyKind::Bat, x1, y1), (EnemyKind::Bat, x2, y2)]
        }
        _ => {
            // Vault guard: two skeletons and a spider around the back wall.
            let (x1, y1) = spot(3, -1.5, -0.5);
            let (x2, y2) = spot(4, -1.0, 0.8);
            let (x3, y3) = spot(5, -2.0, 0.2);
            vec![
                (EnemyKind::Skeleton, x1, y1),
                (EnemyKind::Skeleton, x2, y2),
                (EnemyKind::Spider, x3, y3),
            ]
        }
    }
}

/// One-time vault reward for the dungeon at `(bx, by)`: the vault pays
/// deeper than surface loot (gems/iron, sometimes gold) plus a XP purse.
pub fn vault_loot(bx: i32, by: i32) -> (ItemKind, u32, u32) {
    let r = ((bx as u32).wrapping_mul(83492791) ^ (by as u32).wrapping_mul(2971215073)) % 100;
    let (kind, n) = match r % 5 {
        0 => (ItemKind::Gem, 3),
        1 => (ItemKind::Iron, 4),
        2 => (ItemKind::Gold, 5),
        3 => (ItemKind::Herb, 4),
        _ => (ItemKind::Gem, 2),
    };
    (kind, n, 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foe_tables_are_deterministic_and_shaped() {
        let a = dungeon_foes(10, -20, 1);
        let b = dungeon_foes(10, -20, 1);
        assert_eq!(a, b, "same tile+floor must spawn the same patrol");
        assert_eq!(a.len(), 2, "entry hall: two bats");
        assert!(a.iter().all(|(k, _, _)| *k == EnemyKind::Bat));
        let c = dungeon_foes(10, -20, 2);
        assert_eq!(c.len(), 3, "vault floor: three guards");
        assert!(c.iter().any(|(k, _, _)| *k == EnemyKind::Skeleton));
        // Different tiles differ (the hash actually mixes).
        let d = dungeon_foes(-40, 60, 1);
        assert_ne!(a, d, "different dungeons patrol differently");
    }

    #[test]
    fn vault_pays_deep_loot() {
        for (x, y) in [(10, -20), (-40, 60), (0, 0), (123, -456)] {
            let (kind, n, xp) = vault_loot(x, y);
            assert!(n >= 2, "vault must pay at least 2");
            assert_eq!(xp, 60);
            assert!(
                matches!(kind, ItemKind::Gem | ItemKind::Iron | ItemKind::Gold | ItemKind::Herb),
                "vault pays dungeon-grade goods"
            );
        }
    }
}
