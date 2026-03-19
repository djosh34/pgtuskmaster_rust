path: /home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/decide.rs 401-528

I found smell 1:


I think this is smell 1 because failover candidate selection is split across `classify_candidate`, `candidate_rank`, `compare_candidate_rank`, and `best_failover_candidate`, even though they all restate the same eligibility information. The code projects local state into more local helper state instead of matching directly on `ElectionEligibility` at the decision site.


code:
```rust
fn best_failover_candidate(
    peers: &std::collections::BTreeMap<MemberId, PeerKnowledge>,
    self_peer: &PeerKnowledge,
    self_id: &MemberId,
) -> Option<MemberId> {
    let peer_candidate = peers
        .iter()
        .filter(|(_, peer)| classify_candidate(peer).is_some())
        .map(|(member_id, peer)| (member_id.clone(), peer))
        .max_by(|(left_id, left_peer), (right_id, right_peer)| {
            compare_candidate_rank(
                candidate_rank(&left_peer.eligibility),
                left_id,
                candidate_rank(&right_peer.eligibility),
                right_id,
            )
        })
        .map(|(member_id, _)| member_id);
}

enum CandidateRank {
    Promote(WalPosition),
    Bootstrap,
}

fn candidate_rank(value: &ElectionEligibility) -> Option<CandidateRank> {
    match value {
        ElectionEligibility::PromoteEligible(position) => {
            Some(CandidateRank::Promote(position.clone()))
        }
        ElectionEligibility::BootstrapEligible => Some(CandidateRank::Bootstrap),
        ElectionEligibility::Ineligible(_) => None,
    }
}

fn classify_candidate(peer: &PeerKnowledge) -> Option<()> {
    match &peer.eligibility {
        ElectionEligibility::BootstrapEligible | ElectionEligibility::PromoteEligible(_) => {
            Some(())
        }
        ElectionEligibility::Ineligible(_) => None,
    }
}
```
