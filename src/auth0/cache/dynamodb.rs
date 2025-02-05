use std::{error::Error, time::Duration};

pub use aws_sdk_dynamodb;
use aws_sdk_dynamodb::{
    client::Waiters,
    operation::describe_table::DescribeTableError,
    types::{AttributeDefinition, AttributeValue, TimeToLiveSpecification},
};

use crate::auth0::Token;

use super::{Cache, CacheError};

#[derive(thiserror::Error, Debug)]
pub enum DynamoDBCacheError {
    #[error("AWS error when interacting with dynamo cache: {0}")]
    Aws(Box<dyn Error>),
    #[error("Data in database is wrong. Key: {0}")]
    SchemaError(String),
}

impl From<DynamoDBCacheError> for super::CacheError {
    fn from(val: DynamoDBCacheError) -> Self {
        CacheError(Box::new(val))
    }
}

#[derive(Debug)]
pub struct DynamoDBCache {
    table_name: String,
    client: aws_sdk_dynamodb::Client,
}

impl DynamoDBCache {
    /// Construct a DynamoDBCache instance which uses a given table name and client
    ///
    /// Note: this method doesn't currectly check whether a table with the given name exists during creation.
    pub fn new(client: aws_sdk_dynamodb::Client, table_name: String) -> Self {
        Self { client, table_name }
    }

    pub async fn create_table_if_not_exists(&self) -> Result<(), DynamoDBCacheError> {
        match self
            .client
            .describe_table()
            .table_name(&self.table_name)
            .send()
            .await
            .map_err(|e| e.into_service_error())
        {
            Ok(_) => return Ok(()),
            Err(DescribeTableError::ResourceNotFoundException(_)) => (),
            Err(e) => return Err(DynamoDBCacheError::Aws(Box::new(e))),
        };

        self.client
            .create_table()
            .table_name(self.table_name.clone())
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("key".to_string())
                    .attribute_type(aws_sdk_dynamodb::types::ScalarAttributeType::S)
                    .build()
                    // Unwraps here are fine, will be caught by tests
                    .unwrap(),
            )
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("expiration".to_string())
                    .attribute_type(aws_sdk_dynamodb::types::ScalarAttributeType::N)
                    .build()
                    .unwrap(),
            )
            .send()
            .await
            .map_err(|e| DynamoDBCacheError::Aws(Box::new(e)))?;

        self.client
            .wait_until_table_exists()
            .table_name(&self.table_name)
            .wait(Duration::from_secs(5))
            .await
            .map_err(|e| DynamoDBCacheError::Aws(Box::new(e)))?;

        self.client
            .update_time_to_live()
            .table_name(self.table_name.clone())
            .time_to_live_specification(
                TimeToLiveSpecification::builder()
                    .enabled(true)
                    .attribute_name("expiration")
                    .build()
                    .unwrap(),
            )
            .send()
            .await
            .map_err(|e| DynamoDBCacheError::Aws(Box::new(e)))?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Cache for DynamoDBCache {
    async fn get_token(&self, client_id: &str, aud: &str) -> Result<Option<Token>, CacheError> {
        let key = super::token_key(client_id, aud);
        let Some(attrs) = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("key", AttributeValue::S(key.clone()))
            .send()
            .await
            .map_err(|e| DynamoDBCacheError::Aws(Box::new(e)))?
            .item
        else {
            return Ok(None);
        };

        let token = attrs
            .get("token")
            .and_then(|t| t.as_s().ok())
            .ok_or(DynamoDBCacheError::SchemaError(key.clone()))?;

        let token: Token = serde_json::from_str(token).unwrap();

        Ok(Some(token))
    }

    async fn put_token(&self, client_id: &str, aud: &str, token: &Token) -> Result<(), CacheError> {
        let key = super::token_key(client_id, aud);
        let encoded = serde_json::to_string(token).unwrap();
        self.client
            .put_item()
            .table_name(&self.table_name)
            .item("key", AttributeValue::S(key))
            .item("token", AttributeValue::S(encoded))
            .item(
                "expiration",
                AttributeValue::N(token.expire_date().timestamp().to_string()),
            )
            .send()
            .await
            .map_err(|e| DynamoDBCacheError::Aws(Box::new(e)))?;

        Ok(())
    }
}
