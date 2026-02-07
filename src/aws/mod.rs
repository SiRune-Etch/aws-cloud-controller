//! AWS SDK wrapper for EC2 and Lambda operations

use anyhow::{Context, Result};
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_ec2::Client as Ec2Client;
use aws_sdk_lambda::Client as LambdaClient;
use chrono::{DateTime, Utc};

/// Represents an EC2 instance with relevant metadata
#[derive(Debug, Clone)]
pub struct Ec2Instance {
    pub id: String,
    pub name: String,
    pub instance_type: String,
    pub state: String,
    pub public_ip: Option<String>,
    #[allow(dead_code)] // Reserved for future detailed view
    pub private_ip: Option<String>,
    pub launch_time: Option<DateTime<Utc>>,
    #[allow(dead_code)] // Managed by scheduler
    pub auto_stop_scheduled: Option<DateTime<Utc>>,
}

/// Represents a Lambda function
#[derive(Debug, Clone)]
pub struct LambdaFunction {
    pub name: String,
    #[allow(dead_code)] // Reserved for detailed view
    pub runtime: String,
    #[allow(dead_code)] // Reserved for detailed view
    pub memory: i32,
    #[allow(dead_code)] // Reserved for detailed view
    pub last_modified: String,
    #[allow(dead_code)] // Reserved for detailed view
    pub description: String,
}

/// AWS Client wrapper
#[derive(Debug, Clone)]
pub struct AwsClient {
    ec2: Ec2Client,
    lambda: LambdaClient,
    pub region: String,
}

impl AwsClient {
    /// Create a new AWS client using the default credential chain
    pub async fn new(region_override: Option<&str>) -> Result<Self> {
        let region_provider = RegionProviderChain::first_try(region_override.map(|r| aws_config::Region::new(r.to_string())))
            .or_default_provider()
            .or_else("us-east-1");

        let config = aws_config::from_env()
            .region(region_provider)
            .load()
            .await;

        let region = config.region().map(|r| r.to_string()).unwrap_or_else(|| "us-east-1".to_string());

        Ok(Self {
            ec2: Ec2Client::new(&config),
            lambda: LambdaClient::new(&config),
            region,
        })
    }

    // ─────────────────────────────────────────────────────────────────────────
    // EC2 Operations
    // ─────────────────────────────────────────────────────────────────────────

    /// List all EC2 instances
    pub async fn list_ec2_instances(&self) -> Result<Vec<Ec2Instance>> {
        let response = self.ec2
            .describe_instances()
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to describe EC2 instances: {:?}", e))?;

        let mut instances = Vec::new();

        for reservation in response.reservations() {
            for instance in reservation.instances() {
                let id = instance.instance_id().unwrap_or("N/A").to_string();
                
                // Extract name from tags
                let name = instance
                    .tags()
                    .iter()
                    .find(|t| t.key() == Some("Name"))
                    .and_then(|t| t.value())
                    .unwrap_or(&id)
                    .to_string();

                let instance_type = instance
                    .instance_type()
                    .map(|t| t.as_str().to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                let state = instance
                    .state()
                    .and_then(|s| s.name())
                    .map(|s| s.as_str().to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                let public_ip = instance.public_ip_address().map(|s| s.to_string());
                let private_ip = instance.private_ip_address().map(|s| s.to_string());

                let launch_time = instance
                    .launch_time()
                    .and_then(|t| DateTime::from_timestamp(t.secs(), t.subsec_nanos()));

                instances.push(Ec2Instance {
                    id,
                    name,
                    instance_type,
                    state,
                    public_ip,
                    private_ip,
                    launch_time,
                    auto_stop_scheduled: None, // Will be managed by scheduler
                });
            }
        }

        Ok(instances)
    }

    /// Start an EC2 instance
    pub async fn start_instance(&self, instance_id: &str) -> Result<()> {
        self.ec2
            .start_instances()
            .instance_ids(instance_id)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to start instance {}: {:?}", instance_id, e))?;
        Ok(())
    }

    /// Stop an EC2 instance
    pub async fn stop_instance(&self, instance_id: &str) -> Result<()> {
        self.ec2
            .stop_instances()
            .instance_ids(instance_id)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to stop instance {}: {:?}", instance_id, e))?;
        Ok(())
    }

    /// Terminate an EC2 instance
    pub async fn terminate_instance(&self, instance_id: &str) -> Result<()> {
        self.ec2
            .terminate_instances()
            .instance_ids(instance_id)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to terminate instance {}: {:?}", instance_id, e))?;
        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Lambda Operations
    // ─────────────────────────────────────────────────────────────────────────

    /// List all Lambda functions
    pub async fn list_lambda_functions(&self) -> Result<Vec<LambdaFunction>> {
        let response = self.lambda
            .list_functions()
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list Lambda functions: {:?}", e))?;

        let functions = response
            .functions()
            .iter()
            .map(|f| LambdaFunction {
                name: f.function_name().unwrap_or("N/A").to_string(),
                runtime: f.runtime().map(|r| r.as_str()).unwrap_or("N/A").to_string(),
                memory: f.memory_size().unwrap_or(0),
                last_modified: f.last_modified().unwrap_or("N/A").to_string(),
                description: f.description().unwrap_or("").to_string(),
            })
            .collect();

        Ok(functions)
    }

    /// Invoke a Lambda function
    #[allow(dead_code)] // Reserved for future Lambda invocation feature
    pub async fn invoke_lambda(&self, function_name: &str) -> Result<String> {
        let response = self.lambda
            .invoke()
            .function_name(function_name)
            .send()
            .await
            .context(format!("Failed to invoke Lambda function {}", function_name))?;

        let payload = response.payload()
            .map(|p| String::from_utf8_lossy(p.as_ref()).to_string())
            .unwrap_or_else(|| "No response payload".to_string());

        Ok(payload)
    }
}

/// List available AWS profiles from ~/.aws/config
pub fn list_aws_profiles() -> Result<Vec<String>> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    let config_path = home.join(".aws").join("config");
    
    if !config_path.exists() {
        return Ok(Vec::new());
    }
    
    let content = std::fs::read_to_string(config_path)?;
    let mut profiles = Vec::new();
    
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("[profile ") && line.ends_with(']') {
            let profile_name = line.trim_start_matches("[profile ").trim_end_matches(']');
            profiles.push(profile_name.to_string());
        } else if line == "[default]" {
            profiles.push("default".to_string());
        }
    }
    
    Ok(profiles)
}
