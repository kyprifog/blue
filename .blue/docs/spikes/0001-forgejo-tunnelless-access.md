# Spike: Forgejo Access Without Tunnel

**Date:** 2026-01-24
**Status:** Complete

## Problem

Currently accessing Forgejo requires `kubectl port-forward`. We want direct HTTPS access at `git.beyondtheuniverse.superviber.com`.

## Current State

The infrastructure in `coherence-mcp/infra` is **designed but not fully wired**:

| Component | Status | Blocker |
|-----------|--------|---------|
| EKS Cluster | Unknown | Need to verify deployment |
| Forgejo Deployment | Designed | Depends on cluster |
| AWS ALB Ingress | Template | `${ACM_CERT_ARN}` placeholder |
| cert-manager | Template | `${ACME_EMAIL}`, `${DOMAIN}` placeholders |
| AWS LB Controller IAM | Designed | Controller not installed |
| DNS | Configured in PowerDNS | PowerDNS may not be deployed |

## Root Cause

The ingress at `kubernetes/ingress/core-services.yaml` uses:
```yaml
alb.ingress.kubernetes.io/certificate-arn: ${ACM_CERT_ARN}
```

This placeholder is never substituted. Additionally, the AWS Load Balancer Controller may not be installed.

## Options

### Option A: ACM + AWS ALB (Current Design)

**Pros:** Native AWS, managed TLS, WAF integration possible
**Cons:** Vendor lock-in, requires ACM setup, more moving parts

Steps:
1. Create ACM wildcard certificate for `*.beyondtheuniverse.superviber.com`
2. Install AWS Load Balancer Controller via Helm
3. Substitute `${ACM_CERT_ARN}` with actual ARN
4. Apply ingress
5. Point DNS to ALB

### Option B: NGINX Ingress + cert-manager + Let's Encrypt

**Pros:** Portable, auto-renewing certs, no ACM dependency
**Cons:** Different from current design, requires NGINX controller

Steps:
1. Install NGINX Ingress Controller
2. Configure cert-manager with Let's Encrypt
3. Create Certificate resources for domains
4. Update ingress to use NGINX class
5. Point DNS to NGINX LoadBalancer

### Option C: NLB + Pod TLS (Simplest)

**Pros:** Uses existing NLB, minimal changes, works today
**Cons:** TLS at pod level, can't share certs across services

Steps:
1. Add HTTPS (443) listener to existing NLB
2. Point to Forgejo on port 3000 (or configure Forgejo for 443)
3. Use cert-manager to provision TLS cert for Forgejo
4. Mount cert in Forgejo pod
5. Configure Forgejo for TLS termination

### Option D: Tailscale/Cloudflare Tunnel (Zero Infrastructure)

**Pros:** Works without public IP, easy setup, free tier
**Cons:** External dependency, not self-hosted

## Recommendation

**Option A** for production alignment with existing design. But first, verify cluster state.

## Verification Steps

```bash
# 1. Check if cluster exists and accessible
aws eks describe-cluster --name alignment-production --region us-east-1

# 2. Check if kubectl works
kubectl get nodes

# 3. Check if Forgejo is deployed
kubectl get pods -n forgejo

# 4. Check if AWS LB Controller is installed
kubectl get pods -n kube-system | grep aws-load-balancer

# 5. Check if cert-manager is installed
kubectl get pods -n cert-manager

# 6. Check existing load balancers
aws elbv2 describe-load-balancers --region us-east-1
```

## Quick Fix (If Cluster Exists)

If the cluster is running but just missing the ALB setup:

```bash
# 1. Create ACM certificate
aws acm request-certificate \
  --domain-name "*.beyondtheuniverse.superviber.com" \
  --validation-method DNS \
  --region us-east-1

# 2. Install AWS Load Balancer Controller
helm repo add eks https://aws.github.io/eks-charts
helm install aws-load-balancer-controller eks/aws-load-balancer-controller \
  -n kube-system \
  --set clusterName=alignment-production \
  --set serviceAccount.create=false \
  --set serviceAccount.name=aws-load-balancer-controller

# 3. Apply ingress with correct ARN
export ACM_CERT_ARN="arn:aws:acm:us-east-1:ACCOUNT:certificate/CERT_ID"
envsubst < kubernetes/ingress/core-services.yaml | kubectl apply -f -

# 4. Get ALB DNS name
kubectl get ingress -n ingress core-services -o jsonpath='{.status.loadBalancer.ingress[0].hostname}'

# 5. Point DNS (in PowerDNS or external DNS)
# Create CNAME: git.beyondtheuniverse.superviber.com -> ALB_DNS_NAME
```

## Questions for User

1. Is the EKS cluster currently deployed and running?
2. Do you have Route53 managing `superviber.com` or is it external DNS?
3. Is PowerDNS deployed and authoritative for the subdomain?
4. Do you prefer ACM (AWS managed) or Let's Encrypt (self-managed) for TLS?

## Next Steps

1. Run verification steps above
2. Choose option based on current cluster state
3. Implement chosen option
4. Update runbook to remove port-forward requirement
