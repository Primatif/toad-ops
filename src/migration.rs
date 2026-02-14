use toad_core::ProjectDetail;
use std::collections::HashSet;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MigrationPreflight {
    pub source: String,
    pub target: String,
    pub compatibility_score: u8,
    pub mismatches: Vec<String>,
    pub matching_capabilities: Vec<String>,
}

pub fn compare_projects(source: &ProjectDetail, target: &ProjectDetail) -> MigrationPreflight {
    let mut mismatches = Vec::new();
    let mut matches = Vec::new();
    let mut score = 100;

    // 1. Stack comparison
    if source.stack != target.stack {
        mismatches.push(format!("Stack mismatch: {} vs {}", source.stack, target.stack));
        score -= 40;
    } else {
        matches.push(format!("Identical stacks ({})", source.stack));
    }

    // 2. DNA Roles comparison
    let source_roles: HashSet<_> = source.dna.roles.iter().collect();
    let target_roles: HashSet<_> = target.dna.roles.iter().collect();
    
    for role in source_roles.intersection(&target_roles) {
        matches.push(format!("Matching role: {}", role));
    }
    
    for role in source_roles.difference(&target_roles) {
        mismatches.push(format!("Source role missing in target: {}", role));
        score -= 10;
    }

    // 3. Capabilities comparison
    let source_caps: HashSet<_> = source.dna.capabilities.iter().collect();
    let target_caps: HashSet<_> = target.dna.capabilities.iter().collect();

    for cap in source_caps.intersection(&target_caps) {
        matches.push(format!("Matching capability: {}", cap));
    }

    for cap in source_caps.difference(&target_caps) {
        mismatches.push(format!("Source capability missing in target: {}", cap));
        score -= 5;
    }

    MigrationPreflight {
        source: source.name.clone(),
        target: target.name.clone(),
        compatibility_score: if score > 100 { 100 } else { score as u8 },
        mismatches,
        matching_capabilities: matches,
    }
}
