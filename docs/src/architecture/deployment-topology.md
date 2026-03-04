# Deployment Topology

The system is typically deployed as multiple nodes plus a shared etcd cluster.

```mermaid
flowchart TB
  subgraph ETCDCluster[etcd cluster]
    E1[(etcd)]
    E2[(etcd)]
    E3[(etcd)]
  end

  subgraph NodeA[Node A]
    ARun[pgtuskmaster]
    APG[(PostgreSQL)]
    ARun --> APG
  end

  subgraph NodeB[Node B]
    BRun[pgtuskmaster]
    BPG[(PostgreSQL)]
    BRun --> BPG
  end

  subgraph NodeC[Node C]
    CRun[pgtuskmaster]
    CPG[(PostgreSQL)]
    CRun --> CPG
  end

  ARun <-->|DCS| ETCDCluster
  BRun <-->|DCS| ETCDCluster
  CRun <-->|DCS| ETCDCluster
```

Operational takeaway:
- etcd availability affects **coordination trust**, but does not directly represent PostgreSQL health.
- Each node has local signals (its own PostgreSQL process), and remote signals (DCS + other members’ records), and the HA logic treats those signals differently when DCS trust degrades.
