use tensorflow::{Graph, Session, SessionOptions, Tensor};
use ndarray::{Array, Array2};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AIModule {
    graph: Arc<Graph>,
    session: Arc<Mutex<Session>>,
}

impl AIModule {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut graph = Graph::new();
        let session = Session::new(&SessionOptions::new(), &graph)?;
        
        // Load pre-trained model
        let model_bytes = include_bytes!("../models/message_priority_model.pb");
        graph.import_graph_def(&model_bytes, &ImportGraphDefOptions::new())?;
        
        Ok(AIModule { 
            graph: Arc::new(graph),
            session: Arc::new(Mutex::new(session)),
        })
    }

    pub async fn predict_message_priority(&self, message: &str) -> Result<f32, Box<dyn std::error::Error>> {
        let input = Tensor::new(&[1]).with_values(&[message.as_bytes()])?;
        let mut output = Tensor::new(&[1]);
        
        let mut session = self.session.lock().await;
        session.run(
            &[("input", &input)],
            &mut [("output", &mut output)],
            &[],
            None,
        )?;
        
        Ok(output.data()[0])
    }

    pub async fn optimize_cluster_load(&self, node_loads: &[f32]) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
        let input = Tensor::new(&[node_loads.len() as u64]).with_values(node_loads)?;
        let mut output = Tensor::new(&[node_loads.len() as u64]);

        let mut session = self.session.lock().await;
        session.run(
            &[("load_input", &input)],
            &mut [("load_output", &mut output)],
            &[],
            None,
        )?;

        Ok(output.data().iter().enumerate().map(|(i, &v)| (i, v)).sorted_by(|a, b| b.1.partial_cmp(&a.1).unwrap()).map(|(i, _)| i).collect())
    }

    pub async fn update_model(&self, performance_data: &[f32]) -> Result<(), Box<dyn std::error::Error>> {
        let input = Tensor::new(&[performance_data.len() as u64]).with_values(performance_data)?;
        
        let mut session = self.session.lock().await;
        session.run(
            &[("performance_input", &input)],
            &mut [],
            &["update_weights"],
            None,
        )?;

        println!("AI model updated with performance data");
        Ok(())
    }
}