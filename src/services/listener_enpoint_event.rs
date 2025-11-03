use crate::models::interface_retrieval::SwaggerParseRequest;
use crate::services::interface_retrieval_service::InterfaceRetrievalService;
use crate::services::EndpointService;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info};

pub type ProjectId = String;

pub enum EndpointEvent {
    Created(ProjectId),
    DELETE(ProjectId),
    UPDATE(ProjectId),
}

/// 监听Endpoint增删改, 对应操作向量数据库数据
pub struct EndpointListener {
    pub retrieval: Arc<InterfaceRetrievalService>,
    pub endpoint_service: Arc<EndpointService>,
    pub update_sender: mpsc::Sender<EndpointEvent>,
}

impl EndpointListener {
    pub fn new(
        retrieval: Arc<InterfaceRetrievalService>,
        endpoint_service: Arc<EndpointService>,
        update_sender: mpsc::Sender<EndpointEvent>,
    ) -> EndpointListener {
        Self {
            retrieval,
            endpoint_service,
            update_sender,
        }
    }

    pub async fn find_endpoint_to_spr(
        &self,
        project_id: &ProjectId,
    ) -> Option<SwaggerParseRequest> {
        match self
            .endpoint_service
            .get_endpoint_by_name(project_id.to_string())
            .await
        {
            Ok(endpoint) => {
                match serde_json::from_str::<serde_json::Value>(&endpoint.swagger_content) {
                    Ok(swagger_json) => Some(SwaggerParseRequest {
                        project_id: endpoint.name.clone(),
                        swagger_json,
                        version: Some("1.0.0".to_string()),
                        generate_embeddings: Some(true),
                    }),
                    Err(_) => None,
                }
            }
            Err(_) => None,
        }
    }

    pub fn run(self, mut receive: mpsc::Receiver<EndpointEvent>) {
        tokio::task::spawn(async move {
            loop {
                match receive.recv().await {
                    Some(EndpointEvent::Created(project_id)) => {
                        match self.find_endpoint_to_spr(&project_id).await {
                            None => {}
                            Some(parse_request) => {
                                match self.retrieval.parse_and_store_swagger(parse_request).await {
                                    Ok(_) => {
                                        info!("Successfully re-parsed and stored swagger data for endpoint: {}", project_id);
                                    }
                                    Err(e) => {
                                        error!(
                                            "Failed to re-parse swagger data for endpoint {}: {}",
                                            project_id, e
                                        );
                                    }
                                }
                            }
                        };
                    }
                    Some(EndpointEvent::DELETE(project_id)) => {
                        let d = self
                            .retrieval
                            .delete_project_data(project_id.as_str())
                            .await;
                        info!("delete project: {:?}, result: {:?}", project_id, d);
                    }
                    Some(EndpointEvent::UPDATE(project_id)) => {
                        self.update_sender
                            .send(EndpointEvent::DELETE(project_id.clone()))
                            .await
                            .unwrap();
                        self.update_sender
                            .send(EndpointEvent::Created(project_id))
                            .await
                            .unwrap();
                    }
                    None => {}
                }
            }
        });
        info!("listener enpoint event loop running!");
    }
}
