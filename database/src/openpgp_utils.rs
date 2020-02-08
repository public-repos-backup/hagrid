use failure::Fallible as Result;

use openpgp::{
    Cert,
    RevocationStatus,
    armor::{Writer, Kind},
    packet::UserID,
    serialize::Serialize as OpenPgpSerialize,
    policy::StandardPolicy,
};

lazy_static! {
    pub static ref POLICY: StandardPolicy = StandardPolicy::new();
}

pub fn is_status_revoked(status: RevocationStatus) -> bool {
    match status {
        RevocationStatus::Revoked(_) => true,
        RevocationStatus::CouldBe(_) => false,
        RevocationStatus::NotAsFarAsWeKnow => false,
    }
}

pub fn tpk_to_string(tpk: &Cert) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    {
        let mut armor_writer = Writer::new(&mut buf, Kind::PublicKey, &[][..])?;
        tpk.serialize(&mut armor_writer)?;
        armor_writer.finalize()?;
    }
    Ok(buf)
}

pub fn tpk_clean(tpk: &Cert) -> Result<Cert> {
    // Iterate over the Cert, pushing packets we want to merge
    // into the accumulator.
    let mut acc = Vec::new();

    // The primary key and related signatures.
    let pk_bundle = tpk.primary_key().bundle();
    acc.push(pk_bundle.key().clone().mark_role_primary().into());
    for s in pk_bundle.self_signatures() { acc.push(s.clone().into()) }
    for s in pk_bundle.self_revocations()  { acc.push(s.clone().into()) }
    for s in pk_bundle.other_revocations() { acc.push(s.clone().into()) }

    // The subkeys and related signatures.
    for skb in tpk.keys().subkeys() {
        acc.push(skb.key().clone().into());
        for s in skb.self_signatures()   { acc.push(s.clone().into()) }
        for s in skb.self_revocations()  { acc.push(s.clone().into()) }
        for s in skb.other_revocations() { acc.push(s.clone().into()) }
    }

    // Updates for UserIDs fulfilling `filter`.
    for uidb in tpk.userids().bundles() {
        acc.push(uidb.userid().clone().into());
        for s in uidb.self_signatures()   { acc.push(s.clone().into()) }
        for s in uidb.self_revocations()  { acc.push(s.clone().into()) }
        for s in uidb.other_revocations() { acc.push(s.clone().into()) }
    }

    Cert::from_packet_pile(acc.into())
}

/// Filters the Cert, keeping only those UserIDs that fulfill the
/// predicate `filter`.
pub fn tpk_filter_userids<F>(tpk: &Cert, filter: F) -> Result<Cert>
    where F: Fn(&UserID) -> bool
{
    // Iterate over the Cert, pushing packets we want to merge
    // into the accumulator.
    let mut acc = Vec::new();

    // The primary key and related signatures.
    let pk_bundle = tpk.primary_key().bundle();
    acc.push(pk_bundle.key().clone().mark_role_primary().into());
    for s in pk_bundle.self_signatures() { acc.push(s.clone().into()) }
    for s in pk_bundle.certifications()    { acc.push(s.clone().into()) }
    for s in pk_bundle.self_revocations()  { acc.push(s.clone().into()) }
    for s in pk_bundle.other_revocations() { acc.push(s.clone().into()) }

    // The subkeys and related signatures.
    for skb in tpk.keys().subkeys() {
        acc.push(skb.key().clone().into());
        for s in skb.self_signatures()   { acc.push(s.clone().into()) }
        for s in skb.certifications()    { acc.push(s.clone().into()) }
        for s in skb.self_revocations()  { acc.push(s.clone().into()) }
        for s in skb.other_revocations() { acc.push(s.clone().into()) }
    }

    // Updates for UserIDs fulfilling `filter`.
    for uidb in tpk.userids().bundles() {
        // Only include userids matching filter
        if filter(uidb.userid()) {
            acc.push(uidb.userid().clone().into());
            for s in uidb.self_signatures()   { acc.push(s.clone().into()) }
            for s in uidb.certifications()    { acc.push(s.clone().into()) }
            for s in uidb.self_revocations()  { acc.push(s.clone().into()) }
            for s in uidb.other_revocations() { acc.push(s.clone().into()) }
        }
    }

    Cert::from_packet_pile(acc.into())
}
