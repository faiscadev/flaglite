# FlagLite Terraform Infrastructure

Terraform infrastructure-as-code for deploying FlagLite on AWS.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                          AWS VPC                             │
│  ┌─────────────────────┐    ┌─────────────────────┐        │
│  │   Public Subnet     │    │   Public Subnet     │        │
│  │   (us-east-1a)      │    │   (us-east-1b)      │        │
│  │   ┌───────────┐     │    │                     │        │
│  │   │ NAT GW    │     │    │                     │        │
│  │   └───────────┘     │    │                     │        │
│  └─────────────────────┘    └─────────────────────┘        │
│                                                              │
│  ┌─────────────────────┐    ┌─────────────────────┐        │
│  │   Private Subnet    │    │   Private Subnet    │        │
│  │   (us-east-1a)      │    │   (us-east-1b)      │        │
│  │   ┌───────────┐     │    │   ┌───────────┐     │        │
│  │   │ EKS Node  │     │    │   │ EKS Node  │     │        │
│  │   └───────────┘     │    │   └───────────┘     │        │
│  │   ┌───────────┐     │    │                     │        │
│  │   │ RDS (opt) │     │    │                     │        │
│  │   └───────────┘     │    │                     │        │
│  └─────────────────────┘    └─────────────────────┘        │
└─────────────────────────────────────────────────────────────┘
```

## Prerequisites

1. **AWS CLI** configured with credentials
   ```bash
   aws configure
   ```

2. **Terraform** >= 1.5.0
   ```bash
   brew install terraform  # macOS
   ```

3. **kubectl** for Kubernetes management
   ```bash
   brew install kubectl    # macOS
   ```

## Quick Start

### 1. Navigate to the example directory

```bash
cd examples/aws-minimal
```

### 2. Create your variables file

```bash
cp terraform.tfvars.example terraform.tfvars
# Edit terraform.tfvars with your settings
```

### 3. Initialize Terraform

```bash
terraform init
```

### 4. Review the plan

```bash
terraform plan
```

### 5. Apply the infrastructure

```bash
terraform apply
```

### 6. Configure kubectl

```bash
# The command is provided in the output
aws eks update-kubeconfig --region us-east-1 --name flaglite-dev-cluster
```

### 7. Verify

```bash
kubectl get nodes
```

## Directory Structure

```
terraform/
├── aws/                    # AWS infrastructure
│   ├── main.tf            # Provider config, module calls
│   ├── variables.tf       # Input variables
│   ├── outputs.tf         # Outputs
│   ├── versions.tf        # Version constraints
│   └── modules/
│       ├── eks/           # EKS cluster module
│       ├── rds/           # RDS PostgreSQL module
│       └── networking/    # VPC, subnets, security groups
├── examples/
│   └── aws-minimal/       # Minimal working example
└── README.md
```

## Configuration

### Required Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `project_name` | Project name for resource naming | `flaglite` |
| `environment` | Environment (dev/staging/prod) | `dev` |
| `aws_region` | AWS region | `us-east-1` |

### Optional Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `enable_rds` | Create RDS PostgreSQL instance | `false` |
| `eks_node_instance_type` | EC2 instance type for nodes | `t3.medium` |
| `eks_node_desired_size` | Number of EKS nodes | `2` |

See `aws/variables.tf` for all options.

## Cost Estimates

Estimated monthly costs for `us-east-1` (February 2024):

### Minimal Setup (EKS only)

| Resource | Cost/Month |
|----------|------------|
| NAT Gateway | ~$32 |
| EKS Control Plane | ~$73 |
| t3.medium nodes (x2) | ~$60 |
| CloudWatch Logs | ~$5 |
| **Total** | **~$170** |

### With RDS PostgreSQL

| Resource | Cost/Month |
|----------|------------|
| Minimal Setup | ~$170 |
| db.t3.micro | ~$13 |
| **Total** | **~$183** |

### Cost-Saving Tips

1. **Use Spot instances** for non-production
   - Modify EKS module: `capacity_type = "SPOT"`
   - Saves up to 70% on compute

2. **Reduce node count** in dev
   - Set `eks_node_desired_size = 1`
   - Saves ~$30/month

3. **Use smaller instances**
   - `t3.small` = ~$15/month per node (vs $30 for t3.medium)

4. **Disable CloudWatch logs** in dev
   - Remove `enabled_cluster_log_types` in EKS module

5. **Use in-cluster PostgreSQL** instead of RDS
   - Deploy PostgreSQL as a pod
   - Saves ~$13/month (but less reliable)

## Enabling RDS

To enable RDS PostgreSQL:

```hcl
enable_rds   = true
rds_password = "your-secure-password"  # Use TF_VAR_rds_password env var!
```

**Security Note:** Never commit passwords to version control. Use:
```bash
export TF_VAR_rds_password="your-secure-password"
terraform apply
```

Or better, use AWS Secrets Manager (not included in this starter setup).

## Outputs

After `terraform apply`, you'll get:

- `eks_cluster_name` - Cluster name
- `eks_cluster_endpoint` - API endpoint
- `configure_kubectl` - Command to configure kubectl
- `rds_endpoint` - Database endpoint (if enabled)

## Customization

### Adding Production Hardening

For production, consider:

1. **Multi-AZ NAT Gateways** - Edit `networking/main.tf`
2. **Enable deletion protection** - Set in RDS module
3. **Private EKS endpoint only** - Edit EKS module
4. **AWS Secrets Manager** - For database credentials
5. **S3 backend for state** - Add backend configuration
6. **Cluster autoscaler** - Deploy to EKS

### Using S3 Backend

Uncomment the backend block in `examples/aws-minimal/main.tf`:

```hcl
backend "s3" {
  bucket         = "your-terraform-state-bucket"
  key            = "flaglite/terraform.tfstate"
  region         = "us-east-1"
  dynamodb_table = "terraform-locks"
  encrypt        = true
}
```

## Cleanup

To destroy all resources:

```bash
terraform destroy
```

⚠️ This will delete everything including data. Make backups first!

## Troubleshooting

### "Error: error configuring Terraform AWS Provider"

Check your AWS credentials:
```bash
aws sts get-caller-identity
```

### "Error: Kubernetes cluster unreachable"

Update your kubeconfig:
```bash
aws eks update-kubeconfig --region us-east-1 --name flaglite-dev-cluster
```

### "Error: creating EKS Node Group: ResourceInUseException"

Wait a few minutes and retry. Node groups can take time to delete.

## Contributing

1. Make changes to modules
2. Run `terraform fmt -recursive`
3. Run `terraform validate`
4. Test with `terraform plan`
5. Submit PR

## License

Internal use only - Faísca Engineering
