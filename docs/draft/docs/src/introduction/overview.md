# Overview

pgtuskmaster is a high availability manager for PostgreSQL that automates cluster orchestration across multiple nodes. It operates as a distributed system where each node runs an independent daemon that cooperates through a shared configuration store to maintain cluster consistency.

## System Purpose

The system manages PostgreSQL instances across a cluster to provide continuous availability. When a primary database fails, pgtuskmaster automatically promotes a replica to take over. When you need planned maintenance, it orchestrates graceful switchovers without data loss. The system tracks cluster state, handles member join and departure, and ensures PostgreSQL processes remain aligned with the intended topology.

## Core Concepts

**Cluster** represents a logical PostgreSQL deployment spanning multiple nodes. Each node runs a **pgtuskmaster daemon** that manages a local PostgreSQL instance. Nodes coordinate through a **Distributed Configuration Store (DCS)** backend (etcd) to establish consensus about cluster state and leader identity.

The **High Availability (HA) engine** continuously evaluates cluster health through a timed decision loop. This loop examines local PostgreSQL state, DCS state, and other node states to determine appropriate actions. The engine transitions through distinct phases including initialization, replica operation, candidacy for leadership, primary operation, rewinding, bootstrapping, fencing, and fail-safe handling.

## Architecture Components

**API Layer** exposes cluster state and accepts administrative operations through an HTTP API with configurable TLS and authentication settings.

**Process Management** handles PostgreSQL lifecycle operations: initialization, startup, shutdown, and running maintenance tools like pg_rewind for reintegration.

**Connection Management** maintains distinct identities for different operational contexts: local management, replication, and recovery operations.

**Logging Infrastructure** captures PostgreSQL logs and system events with configurable sinks and retention policies.

The system is organized into worker-oriented modules that separate API handling, DCS integration, HA decision logic, PostgreSQL process control, runtime startup, and logging.
