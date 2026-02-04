# Networking Outputs
output "vpc_id" {
  description = "VPC ID"
  value       = module.networking.vpc_id
}

output "public_subnet_ids" {
  description = "Public subnet IDs"
  value       = module.networking.public_subnet_ids
}

output "private_subnet_ids" {
  description = "Private subnet IDs"
  value       = module.networking.private_subnet_ids
}

# EKS Outputs
output "eks_cluster_name" {
  description = "EKS cluster name"
  value       = module.eks.cluster_name
}

output "eks_cluster_endpoint" {
  description = "EKS cluster endpoint"
  value       = module.eks.cluster_endpoint
}

output "eks_cluster_certificate_authority_data" {
  description = "Base64 encoded certificate data for cluster auth"
  value       = module.eks.cluster_certificate_authority_data
  sensitive   = true
}

output "eks_cluster_arn" {
  description = "EKS cluster ARN"
  value       = module.eks.cluster_arn
}

# RDS Outputs (conditional)
output "rds_endpoint" {
  description = "RDS endpoint (if enabled)"
  value       = var.enable_rds ? module.rds[0].db_endpoint : null
}

output "rds_address" {
  description = "RDS hostname (if enabled)"
  value       = var.enable_rds ? module.rds[0].db_address : null
}

output "rds_port" {
  description = "RDS port (if enabled)"
  value       = var.enable_rds ? module.rds[0].db_port : null
}

output "rds_db_name" {
  description = "Database name (if enabled)"
  value       = var.enable_rds ? module.rds[0].db_name : null
}

# Kubectl configuration command
output "configure_kubectl" {
  description = "Command to configure kubectl"
  value       = "aws eks update-kubeconfig --region ${var.aws_region} --name ${module.eks.cluster_name}"
}
