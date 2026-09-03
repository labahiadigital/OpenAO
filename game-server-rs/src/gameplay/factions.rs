pub struct FactionRank {
    pub rank: i32,
    pub title: &'static str,
    pub min_level: i32,
    pub min_score: i32,
}

pub struct FactionConfig {
    pub key: &'static str,
    pub enlist_npc_id: i32,
    pub color: &'static str,
    pub ranks: &'static [FactionRank],
}

static ARMADA_RANKS: &[FactionRank] = &[
    FactionRank { rank: 1, title: "Soldado", min_level: 25, min_score: 100 },
    FactionRank { rank: 2, title: "Caballero", min_level: 30, min_score: 500 },
    FactionRank { rank: 3, title: "Capitan", min_level: 35, min_score: 1000 },
    FactionRank { rank: 4, title: "Protector del Reino", min_level: 40, min_score: 2500 },
    FactionRank { rank: 5, title: "Campeon de la Luz", min_level: 43, min_score: 5000 },
];

static CAOS_RANKS: &[FactionRank] = &[
    FactionRank { rank: 1, title: "Acolito", min_level: 25, min_score: 350 },
    FactionRank { rank: 2, title: "Emisario del Caos", min_level: 30, min_score: 1000 },
    FactionRank { rank: 3, title: "Sanguinario", min_level: 35, min_score: 2500 },
    FactionRank { rank: 4, title: "Caballero de la Oscuridad", min_level: 40, min_score: 5000 },
    FactionRank { rank: 5, title: "Devorador de Almas", min_level: 43, min_score: 10000 },
];

pub fn get_faction_color(faction: &str) -> Option<&'static str> {
    match faction {
        "armada" => Some("#00AFFF"),
        "caos" => Some("#9B0000"),
        _ => None,
    }
}

pub fn get_max_eligible_rank(faction: &str, level: i32, score: i32) -> i32 {
    let ranks = match faction {
        "armada" => ARMADA_RANKS,
        "caos" => CAOS_RANKS,
        _ => return 0,
    };

    let mut eligible = 0;
    for rank in ranks {
        if level >= rank.min_level && score >= rank.min_score {
            eligible = rank.rank;
        }
    }
    eligible
}

pub fn get_rank_title(faction: &str, rank: i32) -> Option<&'static str> {
    let ranks = match faction {
        "armada" => ARMADA_RANKS,
        "caos" => CAOS_RANKS,
        _ => return None,
    };

    ranks.iter().find(|r| r.rank == rank).map(|r| r.title)
}

pub fn calculate_faction_score(_attacker_level: i32, _victim_level: i32) -> i32 {
    10
}

pub struct RewardClaimResult {
    pub ok: bool,
    pub message: String,
    pub new_rank: i32,
}

pub fn claim_faction_rewards(
    faction: &str,
    level: i32,
    score: i32,
    current_rank: i32,
) -> RewardClaimResult {
    if faction != "armada" && faction != "caos" {
        return RewardClaimResult {
            ok: false,
            message: "No perteneces a ninguna facción.".to_string(),
            new_rank: 0,
        };
    }

    let eligible_rank = get_max_eligible_rank(faction, level, score);

    if eligible_rank <= current_rank {
        let ranks = match faction {
            "armada" => ARMADA_RANKS,
            "caos" => CAOS_RANKS,
            _ => &[],
        };
        let next = ranks.iter().find(|r| r.rank > current_rank);
        let msg = match next {
            Some(nr) => format!(
                "Ya reclamaste tu rango actual ({}). Próximo rango: {} (nivel {}, {} puntos).",
                current_rank, nr.title, nr.min_level, nr.min_score
            ),
            None => format!("Ya tienes el rango máximo ({}).", current_rank),
        };
        return RewardClaimResult {
            ok: false,
            message: msg,
            new_rank: current_rank,
        };
    }

    let title = get_rank_title(faction, eligible_rank).unwrap_or("Desconocido");
    RewardClaimResult {
        ok: true,
        message: format!("Has alcanzado el rango {}: {}!", eligible_rank, title),
        new_rank: eligible_rank,
    }
}
