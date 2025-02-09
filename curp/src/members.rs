use itertools::Itertools;
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::Hasher,
};

/// Server Id
pub type ServerId = u64;

/// Cluster member
#[derive(Debug)]
pub struct Member {
    /// Server id of the member
    id: ServerId,
    /// Name of the member
    name: String,
    /// Address of the member
    address: String,
}

/// cluster members information
#[derive(Debug)]
pub struct ClusterInfo {
    /// cluster id
    cluster_id: u64,
    /// current server id
    member_id: ServerId,
    /// all members information
    members: HashMap<ServerId, Member>,
}

impl ClusterInfo {
    /// Construct a new `ClusterInfo`
    /// # Panics
    /// panic if `all_members` is empty
    #[inline]
    #[must_use]
    pub fn new(all_members: HashMap<String, String>, self_name: &str) -> Self {
        let mut member_id = 0;
        let mut members = HashMap::new();
        for (name, address) in all_members {
            let id = Self::calculate_member_id(&address, "", None);
            if name == self_name {
                member_id = id;
            }
            let member = Member { id, name, address };
            let _ig = members.insert(id, member);
        }
        debug_assert!(member_id != 0, "self_id should not be 0");
        let mut cluster_info = Self {
            cluster_id: 0,
            member_id,
            members,
        };
        cluster_info.gen_cluster_id();
        cluster_info
    }

    /// Get server address via server id
    #[must_use]
    #[inline]
    pub fn address(&self, id: ServerId) -> Option<&str> {
        self.members
            .values()
            .find(|t| t.id == id)
            .map(|t| t.address.as_str())
    }

    /// Get the current member
    #[allow(clippy::indexing_slicing)] // self member id must be in members
    fn self_member(&self) -> &Member {
        &self.members[&self.member_id]
    }

    /// Get the current server address
    #[must_use]
    #[inline]
    pub fn self_address(&self) -> &str {
        &self.self_member().address
    }

    /// Get the current server id
    #[must_use]
    #[inline]
    pub fn self_name(&self) -> &str {
        &self.self_member().name
    }

    /// Get peers id
    #[must_use]
    #[inline]
    pub fn peers_ids(&self) -> Vec<ServerId> {
        self.members
            .values()
            .filter(|t| t.id != self.member_id)
            .map(|t| t.id)
            .collect()
    }

    /// Calculate the member id
    fn calculate_member_id(address: &str, cluster_name: &str, timestamp: Option<u64>) -> ServerId {
        let mut hasher = DefaultHasher::new();
        hasher.write(address.as_bytes());
        hasher.write(cluster_name.as_bytes());
        if let Some(ts) = timestamp {
            hasher.write_u64(ts);
        }
        hasher.finish()
    }

    /// Calculate the cluster id
    fn gen_cluster_id(&mut self) {
        let mut hasher = DefaultHasher::new();
        for id in self.members.keys().sorted() {
            hasher.write_u64(*id);
        }
        self.cluster_id = hasher.finish();
    }

    /// Get member id
    #[must_use]
    #[inline]
    pub fn self_id(&self) -> ServerId {
        self.member_id
    }

    /// Get cluster id
    #[must_use]
    #[inline]
    pub fn cluster_id(&self) -> u64 {
        self.cluster_id
    }

    /// Get peers
    #[must_use]
    #[inline]
    pub fn peers(&self) -> HashMap<ServerId, String> {
        self.members
            .values()
            .filter(|t| t.id != self.member_id)
            .map(|t| (t.id, t.address.clone()))
            .collect()
    }

    /// Get all members
    #[must_use]
    #[inline]
    pub fn all_members(&self) -> HashMap<ServerId, String> {
        self.members
            .values()
            .map(|t| (t.id, t.address.clone()))
            .collect()
    }

    /// Get length of peers
    #[must_use]
    #[inline]
    pub fn members_len(&self) -> usize {
        self.members.len()
    }

    /// Get id by name
    #[must_use]
    #[inline]
    #[cfg(test)]
    pub fn get_id_by_name(&self, name: &str) -> Option<ServerId> {
        self.members
            .iter()
            .find_map(|(_, m)| (m.name == name).then_some(m.id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_id() {
        let all_members: HashMap<String, String> = vec![
            ("S1".to_owned(), "S1".to_owned()),
            ("S2".to_owned(), "S2".to_owned()),
            ("S3".to_owned(), "S3".to_owned()),
        ]
        .into_iter()
        .collect();

        let node1 = ClusterInfo::new(all_members.clone(), "S1");
        let node2 = ClusterInfo::new(all_members.clone(), "S2");
        let node3 = ClusterInfo::new(all_members, "S3");

        assert_ne!(node1.self_id(), node2.self_id());
        assert_ne!(node1.self_id(), node3.self_id());
        assert_ne!(node3.self_id(), node2.self_id());

        assert_eq!(node1.cluster_id(), node2.cluster_id());
        assert_eq!(node3.cluster_id(), node2.cluster_id());
    }

    #[test]
    fn test_get_peers() {
        let all_members = HashMap::from([
            ("S1".to_owned(), "S1".to_owned()),
            ("S2".to_owned(), "S2".to_owned()),
            ("S3".to_owned(), "S3".to_owned()),
        ]);

        let node1 = ClusterInfo::new(all_members, "S1");
        let peers = node1.peers();
        let node1_id = node1.self_id();
        let node1_url = node1.self_address();
        assert!(!peers.contains_key(&node1_id));
        assert_eq!(peers.len(), 2);
        assert_eq!(node1.members_len(), peers.len() + 1);

        let peer_urls = peers.values().collect::<Vec<_>>();

        let peer_ids = node1.peers_ids();

        assert_eq!(peer_ids.len(), peer_urls.len());

        assert!(peer_urls
            .iter()
            .find(|url| url.as_str() == node1_url)
            .is_none());
        assert!(peer_ids.iter().find(|id| **id == node1_id).is_none());
    }
}
