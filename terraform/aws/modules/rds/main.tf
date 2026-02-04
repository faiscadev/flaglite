locals {
  name = "${var.project_name}-${var.environment}"

  common_tags = merge(var.tags, {
    Project     = var.project_name
    Environment = var.environment
    ManagedBy   = "terraform"
  })
}

# DB Subnet Group
resource "aws_db_subnet_group" "main" {
  name       = "${local.name}-db-subnet-group"
  subnet_ids = var.subnet_ids

  tags = merge(local.common_tags, {
    Name = "${local.name}-db-subnet-group"
  })
}

# RDS PostgreSQL Instance
resource "aws_db_instance" "main" {
  identifier = "${local.name}-postgres"

  # Engine
  engine                = "postgres"
  engine_version        = "15.4"
  instance_class        = var.instance_class
  allocated_storage     = var.allocated_storage
  max_allocated_storage = var.allocated_storage * 2 # Auto-scaling up to 2x

  # Database
  db_name  = var.db_name
  username = var.username
  password = var.password

  # Network
  db_subnet_group_name   = aws_db_subnet_group.main.name
  vpc_security_group_ids = var.security_group_ids
  publicly_accessible    = false
  port                   = 5432

  # Storage
  storage_type      = "gp3"
  storage_encrypted = true

  # Maintenance
  auto_minor_version_upgrade = true
  maintenance_window         = "Mon:00:00-Mon:03:00"
  backup_window              = "03:00-06:00"
  backup_retention_period    = 7
  delete_automated_backups   = true
  deletion_protection        = false # Set to true for production
  skip_final_snapshot        = true  # Set to false for production
  final_snapshot_identifier  = "${local.name}-final-snapshot"

  # Performance Insights (disabled to save costs on small instances)
  performance_insights_enabled = false

  # Monitoring
  monitoring_interval = 0 # Disabled (set to 60 for enhanced monitoring)

  tags = merge(local.common_tags, {
    Name = "${local.name}-postgres"
  })

  lifecycle {
    prevent_destroy = false # Set to true for production
  }
}
