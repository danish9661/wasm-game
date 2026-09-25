//! Work tools (Minecraft tiers for the iso grid): stone and iron axes bite
//! deeper into every chop, picks multiply quarry yields. Tools wear with
//! use and snap at zero — the uses-left live on `Player` (`tools` bitmask
//! + `tool_hp`), so stacks never need per-item state and saves stay flat.

use crate::resources::ResourceKind;

/// Work tools, bit-indexed into `Player::tools` (bit `k as u8`).
/// Craft costs live in the anvil recipe list (`CRAFT_RECIPES`, renderer),
/// which is the single source the craft panel renders from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolKind {
    StoneAxe,
    StonePick,
    IronAxe,
    IronPick,
}

impl ToolKind {
    pub fn bit(self) -> u8 {
        1 << (self as u8)
    }

    pub fn name(self) -> &'static str {
        match self {
            ToolKind::StoneAxe => "Stone Axe",
            ToolKind::StonePick => "Stone Pickaxe",
            ToolKind::IronAxe => "Iron Axe",
            ToolKind::IronPick => "Iron Pickaxe",
        }
    }

    /// Chops per E-press on matching nodes (bare hands: 1).
    pub fn power(self) -> u32 {
        match self {
            ToolKind::StoneAxe | ToolKind::StonePick => 2,
            ToolKind::IronAxe | ToolKind::IronPick => 3,
        }
    }

    /// Extra stone per dig (bare hands dig 2).
    pub fn dig_bonus(self) -> u32 {
        match self {
            ToolKind::StonePick => 1,
            ToolKind::IronPick => 2,
            _ => 0,
        }
    }

    /// Uses before the tool snaps.
    pub fn max_hp(self) -> u16 {
        match self {
            ToolKind::StoneAxe | ToolKind::StonePick => 60,
            ToolKind::IronAxe | ToolKind::IronPick => 140,
        }
    }

    /// True for the rock-biters' targets (ore, stone, crystal); axes take
    /// wood and everything else. Treasure caches open by hand either way.
    pub fn is_rock(kind: ResourceKind) -> bool {
        matches!(
            kind,
            ResourceKind::Rock | ResourceKind::Ore | ResourceKind::Crystal
        )
    }

    /// Best owned pick bonus (0 = bare hands).
    pub fn pick_bonus_owned(tools: u8) -> u32 {
        let mut bonus = 0;
        for t in [ToolKind::StonePick, ToolKind::IronPick] {
            if tools & t.bit() != 0 {
                bonus = bonus.max(t.dig_bonus());
            }
        }
        bonus
    }

    /// Strongest owned chopping power (1 = bare hands). `rock` selects the
    /// pick row, otherwise the axe row.
    pub fn power_owned(tools: u8, rock: bool) -> u32 {
        let row = if rock {
            [ToolKind::StonePick, ToolKind::IronPick]
        } else {
            [ToolKind::StoneAxe, ToolKind::IronAxe]
        };
        let mut power = 1;
        for t in row {
            if tools & t.bit() != 0 {
                power = power.max(t.power());
            }
        }
        power
    }

    /// The owned tool that did the work (for wear): strongest of the row,
    /// or None when working by hand.
    pub fn used_tool(tools: u8, rock: bool) -> Option<ToolKind> {
        let row = if rock {
            [ToolKind::IronPick, ToolKind::StonePick]
        } else {
            [ToolKind::IronAxe, ToolKind::StoneAxe]
        };
        row.into_iter().find(|t| tools & t.bit() != 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiers_improve_everything() {
        assert!(ToolKind::IronAxe.power() > ToolKind::StoneAxe.power());
        assert!(ToolKind::IronPick.dig_bonus() > ToolKind::StonePick.dig_bonus());
        assert!(ToolKind::IronPick.max_hp() > ToolKind::StonePick.max_hp());
        // Hands are the floor, not zero.
        assert_eq!(ToolKind::power_owned(0, false), 1);
        assert_eq!(ToolKind::power_owned(0, true), 1);
        assert_eq!(ToolKind::pick_bonus_owned(0), 0);
        assert_eq!(ToolKind::used_tool(0, false), None);
    }

    #[test]
    fn strongest_owned_tool_wins() {
        let both = ToolKind::StoneAxe.bit() | ToolKind::IronAxe.bit();
        assert_eq!(ToolKind::power_owned(both, false), 3);
        assert_eq!(ToolKind::used_tool(both, false), Some(ToolKind::IronAxe));
        let picks = ToolKind::StonePick.bit() | ToolKind::IronPick.bit();
        assert_eq!(ToolKind::pick_bonus_owned(picks), 2);
    }

    #[test]
    fn bits_are_distinct_flags() {
        let all = [ToolKind::StoneAxe, ToolKind::StonePick, ToolKind::IronAxe, ToolKind::IronPick];
        let mut mask = 0u8;
        for t in all {
            assert_eq!(mask & t.bit(), 0, "bit collision");
            mask |= t.bit();
        }
    }
}
