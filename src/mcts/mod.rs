use std::collections::HashMap;
use uuid::Uuid;

/// Represents a node in the MCTS search tree
#[derive(Debug)]
pub struct Node {
    /// Unique identifier for the node
    pub id: Uuid,
    /// The step's natural language reasoning
    pub reasoning: String,
    /// The step's Python code
    pub code: String,
    /// Number of times this node has been visited
    pub visits: u32,
    /// Accumulated Q-value from rollouts
    pub q_value: f32,
    /// PPM score for this step
    pub ppm_score: Option<f32>,
    /// Parent node ID
    pub parent: Option<Uuid>,
    /// Child node IDs
    pub children: Vec<Uuid>,
    /// Whether code execution was successful
    pub code_verified: bool,
}

impl Node {
    pub fn new(reasoning: String, code: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            reasoning,
            code,
            visits: 0,
            q_value: 0.0,
            ppm_score: None,
            parent: None,
            children: Vec::new(),
            code_verified: false,
        }
    }

    /// Calculate UCT value for node selection
    pub fn uct(&self, parent_visits: u32, exploration_constant: f32) -> f32 {
        if self.visits == 0 {
            return f32::INFINITY;
        }

        let exploitation = self.q_value / self.visits as f32;
        let exploration = exploration_constant * ((parent_visits as f32).ln() / self.visits as f32).sqrt();

        exploitation + exploration
    }
}

/// Represents the MCTS search tree
pub struct MCTSTree {
    /// Maps node IDs to nodes
    nodes: HashMap<Uuid, Node>,
    /// Root node ID
    root: Uuid,
    /// Maximum depth for search
    max_depth: u32,
    /// Exploration constant for UCT
    exploration_constant: f32,
}

impl MCTSTree {
    pub fn new(question: String, max_depth: u32, exploration_constant: f32) -> Self {
        let root = Node::new(question, String::new());
        let root_id = root.id;
        let mut nodes = HashMap::new();
        nodes.insert(root_id, root);

        Self {
            nodes,
            root: root_id,
            max_depth,
            exploration_constant,
        }
    }

    /// Get a reference to a node by ID
    pub fn get_node(&self, id: &Uuid) -> Option<&Node> {
        self.nodes.get(id)
    }

    /// Get a mutable reference to a node by ID
    pub fn get_node_mut(&mut self, id: &Uuid) -> Option<&mut Node> {
        self.nodes.get_mut(id)
    }

    /// Add a child node to a parent node
    pub fn add_child(&mut self, parent_id: &Uuid, reasoning: String, code: String) -> Option<Uuid> {
        let mut child = Node::new(reasoning, code);
        child.parent = Some(*parent_id);
        let child_id = child.id;

        if let Some(parent) = self.nodes.get_mut(parent_id) {
            parent.children.push(child_id);
            self.nodes.insert(child_id, child);
            Some(child_id)
        } else {
            None
        }
    }

    /// Update node statistics after a rollout
    pub fn backpropagate(&mut self, node_id: &Uuid, reward: f32) {
        let mut current_id = Some(*node_id);

        while let Some(id) = current_id {
            if let Some(node) = self.nodes.get_mut(&id) {
                node.visits += 1;
                node.q_value += reward;
                current_id = node.parent;
            } else {
                break;
            }
        }
    }
}

/// Trait for policy models that generate candidate steps
pub trait PolicyModel {
    /// Generate candidate next steps given the current trajectory
    fn generate_candidates(&self, trajectory: &[&Node], n_candidates: usize) -> Vec<(String, String)>;
}

/// Trait for process preference models that score steps
pub trait PreferenceModel {
    /// Score a step given the trajectory leading to it
    fn score_step(&self, trajectory: &[&Node], step: &Node) -> f32;
}

/// Trait for code verification
pub trait CodeVerifier {
    /// Verify if the Python code executes successfully
    fn verify_code(&self, code: &str) -> bool;
}
