use super::{MCTSTree, Node, PolicyModel, PreferenceModel, CodeVerifier};
use uuid::Uuid;

pub struct MCTSSearch<P, R, C>
where
    P: PolicyModel,
    R: PreferenceModel,
    C: CodeVerifier,
{
    tree: MCTSTree,
    policy: P,
    ppm: R,
    verifier: C,
    n_candidates: usize,
    n_rollouts: u32,
}

impl<P, R, C> MCTSSearch<P, R, C>
where
    P: PolicyModel,
    R: PreferenceModel,
    C: CodeVerifier,
{
    pub fn new(
        question: String,
        policy: P,
        ppm: R,
        verifier: C,
        max_depth: u32,
        exploration_constant: f32,
        n_candidates: usize,
        n_rollouts: u32,
    ) -> Self {
        Self {
            tree: MCTSTree::new(question, max_depth, exploration_constant),
            policy,
            ppm,
            verifier,
            n_candidates,
            n_rollouts,
        }
    }

    /// Run MCTS search to find the best solution trajectory
    pub fn search(&mut self) -> Vec<Uuid> {
        for _ in 0..self.n_rollouts {
            // Selection phase
            let selected = self.select();

            // Expansion phase
            if let Some(expanded) = self.expand(&selected) {
                // Simulation phase
                let reward = self.simulate(&expanded);

                // Backpropagation phase
                self.tree.backpropagate(&expanded, reward);
            }
        }

        // Return best trajectory
        self.get_best_trajectory()
    }

    /// Select a promising node to expand using UCT
    fn select(&self) -> Uuid {
        let mut current = self.tree.root;

        while let Some(node) = self.tree.get_node(&current) {
            if node.children.is_empty() {
                return current;
            }

            let parent_visits = node.visits;
            current = node.children
                .iter()
                .max_by(|a, b| {
                    let node_a = self.tree.get_node(a).unwrap();
                    let node_b = self.tree.get_node(b).unwrap();
                    node_a.uct(parent_visits, self.tree.exploration_constant)
                        .partial_cmp(&node_b.uct(parent_visits, self.tree.exploration_constant))
                        .unwrap()
                })
                .copied()
                .unwrap();
        }
        current
    }

    /// Expand a node by generating and verifying candidate steps
    fn expand(&mut self, node_id: &Uuid) -> Option<Uuid> {
        let trajectory = self.get_trajectory_to_node(node_id);
        let candidates = self.policy.generate_candidates(&trajectory, self.n_candidates);

        for (reasoning, code) in candidates {
            if self.verifier.verify_code(&code) {
                if let Some(child_id) = self.tree.add_child(node_id, reasoning, code) {
                    if let Some(child) = self.tree.get_node_mut(&child_id) {
                        child.code_verified = true;
                        child.ppm_score = Some(self.ppm.score_step(&trajectory, child));
                        return Some(child_id);
                    }
                }
            }
        }
        None
    }

    /// Simulate from a node to estimate its value
    fn simulate(&self, node_id: &Uuid) -> f32 {
        // For now, we'll use the PPM score directly as the simulation result
        // In practice, you might want to do multiple rollouts or use more sophisticated simulation
        if let Some(node) = self.tree.get_node(node_id) {
            node.ppm_score.unwrap_or(0.0)
        } else {
            0.0
        }
    }

    /// Get the trajectory of nodes leading to a given node
    fn get_trajectory_to_node(&self, node_id: &Uuid) -> Vec<&Node> {
        let mut trajectory = Vec::new();
        let mut current_id = Some(*node_id);

        while let Some(id) = current_id {
            if let Some(node) = self.tree.get_node(&id) {
                trajectory.push(node);
                current_id = node.parent;
            } else {
                break;
            }
        }

        trajectory.reverse();
        trajectory
    }

    /// Get the best trajectory after search is complete
    fn get_best_trajectory(&self) -> Vec<Uuid> {
        let mut trajectory = Vec::new();
        let mut current = self.tree.root;

        while let Some(node) = self.tree.get_node(&current) {
            trajectory.push(current);

            if node.children.is_empty() {
                break;
            }

            // Choose child with highest Q-value
            current = node.children
                .iter()
                .max_by(|a, b| {
                    let node_a = self.tree.get_node(a).unwrap();
                    let node_b = self.tree.get_node(b).unwrap();
                    node_a.q_value.partial_cmp(&node_b.q_value).unwrap()
                })
                .copied()
                .unwrap();
        }

        trajectory
    }
}
