# FlagLite Minimal AWS Example
# This example provisions the minimum infrastructure for FlagLite

terraform {
  required_version = ">= 1.5.0"

  # Uncomment to use S3 backend for state storage
  # backend "s3" {
  #   bucket         = "your-terraform-state-bucket"
  #   key            = "flaglite/terraform.tfstate"
  #   region         = "us-east-1"
  #   dynamodb_table = "terraform-locks"
  #   encrypt        = true
  # }
}

module "flaglite" {
  source = "../../aws"

  # Project settings
  project_name = var.project_name
  environment  = var.environment
  aws_region   = var.aws_region

  # Networking
  vpc_cidr           = var.vpc_cidr
  availability_zones = var.availability_zones

  # EKS
  eks_cluster_version    = var.eks_cluster_version
  eks_node_instance_type = var.eks_node_instance_type
  eks_node_desired_size  = var.eks_node_desired_size
  eks_node_min_size      = var.eks_node_min_size
  eks_node_max_size      = var.eks_node_max_size

  # RDS (optional)
  enable_rds            = var.enable_rds
  rds_instance_class    = var.rds_instance_class
  rds_allocated_storage = var.rds_allocated_storage
  rds_db_name           = var.rds_db_name
  rds_username          = var.rds_username
  rds_password          = var.rds_password

  tags = var.tags
}

# Variables
variable "project_name" {
  default = "flaglite"
}

variable "environment" {
  default = "dev"
}

variable "aws_region" {
  default = "us-east-1"
}

variable "vpc_cidr" {
  default = "10.0.0.0/16"
}

variable "availability_zones" {
  default = ["us-east-1a", "us-east-1b"]
}

variable "eks_cluster_version" {
  default = "1.29"
}

variable "eks_node_instance_type" {
  default = "t3.medium"
}

variable "eks_node_desired_size" {
  default = 2
}

variable "eks_node_min_size" {
  default = 1
}

variable "eks_node_max_size" {
  default = 4
}

variable "enable_rds" {
  default = false
}

variable "rds_instance_class" {
  default = "db.t3.micro"
}

variable "rds_allocated_storage" {
  default = 20
}

variable "rds_db_name" {
  default = "flaglite"
}

variable "rds_username" {
  default = "flaglite_admin"
}

variable "rds_password" {
  default   = null
  sensitive = true
}

variable "tags" {
  default = {}
}

# Outputs
output "eks_cluster_name" {
  value = module.flaglite.eks_cluster_name
}

output "eks_cluster_endpoint" {
  value = module.flaglite.eks_cluster_endpoint
}

output "configure_kubectl" {
  value = module.flaglite.configure_kubectl
}

output "rds_endpoint" {
  value = module.flaglite.rds_endpoint
}
