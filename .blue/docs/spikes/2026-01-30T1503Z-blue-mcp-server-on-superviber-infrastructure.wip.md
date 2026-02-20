# Spike: Blue MCP Server on Superviber Infrastructure

| | |
|---|---|
| **Status** | In Progress |
| **Date** | 2026-01-30 |
| **Time Box** | 1 hour |

---

## Question

How can we run the Blue MCP server on Superviber infrastructure while maintaining client data sovereignty, encryption, and revocable access?

---

## Context

The current architecture (Appendix A of the financial portfolio doc) assumes Blue MCP runs in the client's AWS account. However, running MCP on Superviber infrastructure offers benefits:

- **Simpler client onboarding**: No deployment required in client account
- **Centralized updates**: Push new features without client coordination
- **Operational visibility**: Better observability and debugging
- **Cost efficiency**: Shared infrastructure across clients

The challenge: maintain data sovereignty guarantees while centralizing compute.

---

## Architecture Options

### Option A: Proxy Model with Client Data Store

MCP server on Superviber infra acts as stateless compute. All persistent data remains on client infrastructure, accessed via secure API.

```mermaid
flowchart TB
    subgraph CLIENT["Client AWS Account"]
        direction TB
        DS[("Data Store<br/>(S3/DynamoDB)")]
        KMS["Client KMS"]
        API["Client API Gateway"]
        DS --- KMS
        API --- DS
    end

    subgraph SV["Superviber Infrastructure"]
        direction TB
        MCP["Blue MCP Server"]
        INF["Infisical<br/>(Secrets)"]
        MCP --- INF
    end

    subgraph CLAUDE["Claude Code"]
        CC["User Session"]
    end

    CC -->|"MCP Protocol<br/>(TLS 1.3)"| MCP
    MCP -->|"Cross-Account<br/>AssumeRole"| API

    style CLIENT fill:#e8f5e9
    style SV fill:#e3f2fd
    style CLAUDE fill:#fff3e0
```

**Data Flow:**
1. Claude Code connects to Blue MCP on Superviber infra
2. MCP assumes cross-account role to access client API
3. Client API reads/writes to encrypted data store
4. Data encrypted by client KMS - MCP never sees plaintext keys
5. MCP processes in memory, never persists client data

### Option B: PrivateLink Model

AWS PrivateLink provides private connectivity without traversing public internet.

```mermaid
flowchart LR
    subgraph CLIENT["Client VPC"]
        direction TB
        DS[("Encrypted<br/>Data Store")]
        EP["VPC Endpoint<br/>(PrivateLink)"]
        DS --- EP
    end

    subgraph SV["Superviber VPC"]
        direction TB
        MCP["Blue MCP"]
        NLB["Network Load<br/>Balancer"]
        ES["Endpoint<br/>Service"]
        MCP --- NLB --- ES
    end

    EP <-->|"Private<br/>Connection"| ES

    style CLIENT fill:#e8f5e9
    style SV fill:#e3f2fd
```

**Pros:** Traffic never leaves AWS backbone, lower latency
**Cons:** More complex setup, per-client PrivateLink costs

### Option C: Hybrid with Edge Cache

MCP runs on Superviber with optional edge caching for read-heavy ADR/RFC data.

```mermaid
flowchart TB
    subgraph CLIENT["Client Account"]
        DS[("Source of Truth<br/>(Encrypted)")]
        HOOK["Webhook<br/>on Change"]
    end

    subgraph SV["Superviber"]
        direction TB
        MCP["Blue MCP"]
        CACHE[("Edge Cache<br/>(Ephemeral)")]
        MCP --- CACHE
    end

    DS -->|"Sync on<br/>Change"| HOOK
    HOOK -->|"Invalidate"| CACHE
    MCP <-->|"Read/Write"| DS

    style CLIENT fill:#e8f5e9
    style SV fill:#e3f2fd
```

**Pros:** Better performance for read-heavy workloads
**Cons:** Cache adds complexity, eventual consistency

---

## Recommended Architecture: Option A (Proxy Model)

The proxy model is simplest and maintains strongest data sovereignty guarantees.

### Detailed Architecture

```mermaid
flowchart TB
    subgraph CLAUDE["Claude Code (User Machine)"]
        CC["Claude Session"]
    end

    subgraph SV["Superviber Infrastructure"]
        direction TB

        subgraph MCP_CLUSTER["MCP Cluster (EKS)"]
            MCP1["MCP Pod 1"]
            MCP2["MCP Pod 2"]
            MCPN["MCP Pod N"]
        end

        ALB["Application<br/>Load Balancer"]
        INF["Infisical"]

        ALB --> MCP1 & MCP2 & MCPN
        MCP1 & MCP2 & MCPN --> INF
    end

    subgraph CLIENT["Client AWS Account"]
        direction TB

        subgraph VPC["Client VPC"]
            APIGW["API Gateway<br/>(Private)"]
            LAMBDA["Lambda<br/>(Data Access)"]

            subgraph DATA["Data Layer"]
                S3[("S3 Bucket<br/>(Dialogues, RFCs)")]
                DDB[("DynamoDB<br/>(State, Index)")]
            end

            KMS["KMS Key<br/>(Client Owned)"]
        end

        IAM["IAM Role<br/>(Cross-Account)"]

        APIGW --> LAMBDA --> DATA
        DATA --> KMS
    end

    CC -->|"① MCP over TLS 1.3"| ALB
    MCP1 -->|"② AssumeRole"| IAM
    IAM -->|"③ Scoped Access"| APIGW

    style CLAUDE fill:#fff3e0
    style SV fill:#e3f2fd
    style CLIENT fill:#e8f5e9
    style DATA fill:#c8e6c9
```

### Request Flow

```mermaid
sequenceDiagram
    participant CC as Claude Code
    participant MCP as Blue MCP<br/>(Superviber)
    participant INF as Infisical
    participant STS as AWS STS
    participant API as Client API
    participant KMS as Client KMS
    participant S3 as Client S3

    CC->>MCP: blue_rfc_get("0042")

    MCP->>INF: Get client credentials
    INF-->>MCP: Client ID, Role ARN

    MCP->>STS: AssumeRole(client_role_arn)
    STS-->>MCP: Temporary credentials (1hr)

    MCP->>API: GET /rfcs/0042
    API->>S3: GetObject(rfcs/0042.md)
    S3->>KMS: Decrypt(data_key)
    KMS-->>S3: Plaintext key
    S3-->>API: Decrypted content
    API-->>MCP: RFC content

    MCP-->>CC: RFC document

    Note over MCP: Data processed in memory<br/>Never persisted
```

### Access Control Matrix

| Resource | Superviber Access | Client Control |
|----------|-------------------|----------------|
| Blue MCP Server | Owns & operates | N/A |
| Client API Gateway | Invoke via role | Creates/deletes endpoint |
| Client S3 Bucket | Read/write via role | Owns bucket, sets policy |
| Client DynamoDB | Read/write via role | Owns table, sets policy |
| Client KMS Key | **No access** | Full control |
| Infisical Secrets | Read (membership) | Owns workspace, can revoke |
| IAM Cross-Account Role | AssumeRole | Creates/deletes role |

### Client IAM Role Policy

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "AllowBlueMCPAccess",
      "Effect": "Allow",
      "Action": [
        "s3:GetObject",
        "s3:PutObject",
        "s3:ListBucket"
      ],
      "Resource": [
        "arn:aws:s3:::client-blue-data",
        "arn:aws:s3:::client-blue-data/*"
      ]
    },
    {
      "Sid": "AllowDynamoDBAccess",
      "Effect": "Allow",
      "Action": [
        "dynamodb:GetItem",
        "dynamodb:PutItem",
        "dynamodb:Query",
        "dynamodb:UpdateItem"
      ],
      "Resource": "arn:aws:dynamodb:*:*:table/blue-*"
    },
    {
      "Sid": "DenyKMSAccess",
      "Effect": "Deny",
      "Action": "kms:*",
      "Resource": "*"
    }
  ]
}
```

**Key point:** The `DenyKMSAccess` statement ensures Superviber can never access encryption keys directly. S3 and DynamoDB use envelope encryption - they decrypt data using the KMS key, but the key itself never leaves KMS.

### Trust Policy (Client Creates)

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Principal": {
        "AWS": "arn:aws:iam::SUPERVIBER_ACCOUNT_ID:role/BlueMCPServiceRole"
      },
      "Action": "sts:AssumeRole",
      "Condition": {
        "StringEquals": {
          "sts:ExternalId": "${client_external_id}"
        }
      }
    }
  ]
}
```

**Revocation:** Client removes or modifies this trust policy → immediate access termination.

---

## Infisical Integration

```mermaid
flowchart LR
    subgraph CLIENT_INF["Client's Infisical Workspace"]
        direction TB
        SEC1["AWS_ROLE_ARN"]
        SEC2["EXTERNAL_ID"]
        SEC3["API_ENDPOINT"]
        SEC4["ANTHROPIC_API_KEY"]
    end

    subgraph SV_INF["Superviber Infisical"]
        direction TB
        SVC["Service Token<br/>(Read-Only)"]
    end

    subgraph MCP["Blue MCP"]
        ENV["Runtime Env"]
    end

    SVC -->|"Membership"| CLIENT_INF
    CLIENT_INF -->|"Inject"| ENV

    style CLIENT_INF fill:#e8f5e9
    style SV_INF fill:#e3f2fd
```

**Client onboarding:**
1. Client creates Infisical workspace
2. Client adds required secrets (role ARN, endpoint, etc.)
3. Client invites Superviber service account (read-only)
4. Client can revoke by removing membership

---

## Data Sovereignty Guarantees (Updated)

| Guarantee | Previous (Client Infra) | New (Superviber Infra) |
|-----------|-------------------------|------------------------|
| Data at rest | Client S3/KMS | Client S3/KMS (unchanged) |
| Data in flight | TLS 1.3 | TLS 1.3 (unchanged) |
| Encryption keys | Client KMS | Client KMS (unchanged) |
| Compute location | Client account | Superviber account |
| Data in memory | Client account | Superviber account (ephemeral) |
| Revocation | IAM + Infisical | IAM + Infisical (unchanged) |
| Audit trail | Client CloudTrail | Client CloudTrail + Superviber logs |

**New consideration:** Data passes through Superviber memory during processing. Mitigations:
- No persistence - data only held during request lifecycle
- Memory encryption at rest (EKS with encrypted nodes)
- SOC 2 attestation for Superviber infrastructure
- Option for dedicated/isolated compute per client

---

## Client Onboarding Flow

```mermaid
flowchart TB
    A["1. Client signs agreement"] --> B["2. Client creates<br/>Infisical workspace"]
    B --> C["3. Client provisions<br/>IAM role with trust policy"]
    C --> D["4. Client creates<br/>S3 bucket + DynamoDB"]
    D --> E["5. Client adds secrets<br/>to Infisical"]
    E --> F["6. Client invites<br/>Superviber to workspace"]
    F --> G["7. Superviber configures<br/>MCP for client"]
    G --> H["8. Client connects<br/>Claude Code to MCP"]

    style A fill:#ffecb3
    style H fill:#c8e6c9
```

**Estimated onboarding time:** 30 minutes with Terraform/CDK templates provided.

---

## Open Questions

1. **Multi-tenancy:** Single MCP cluster serving all clients, or isolated per client?
   - Single cluster: Cost efficient, simpler ops
   - Isolated: Stronger security boundary, client preference for finance

2. **Latency:** Cross-account API calls add ~50-100ms per request. Acceptable?
   - Most MCP operations are not latency-sensitive
   - Dialogue runs are already async

3. **Compliance:** Does data-in-memory on Superviber infra affect client's compliance posture?
   - May need to add SOC 2 Type II for Superviber
   - Some clients may still require fully client-hosted

4. **Failover:** If Superviber MCP is down, clients have no access
   - Consider multi-region deployment
   - Or provide fallback to client-hosted MCP

---

## Recommendation

Proceed with **Option A (Proxy Model)** with the following implementation:

1. Deploy Blue MCP on EKS in Superviber AWS account
2. Use Infisical for per-client credential management
3. Provide Terraform/CDK module for client-side infrastructure
4. Offer "dedicated compute" tier for compliance-sensitive clients
5. Document the memory-processing caveat in security docs

**Next steps:**
- [ ] Create RFC for this architecture
- [ ] Build Terraform module for client infrastructure
- [ ] Add multi-tenant support to Blue MCP
- [ ] Draft updated security/compliance documentation

---

*Investigation by Blue*
