export interface Model {
  id: string;
  name: string;
  version: string;
  task_type: string;
  status: string;
  created_at: string;
  updated_at: string;
  metrics?: ModelMetrics;
  description?: string;
  parameters?: ModelParameters;
}

export interface ModelMetrics {
  accuracy?: number;
  precision?: number;
  recall?: number;
  f1_score?: number;
  loss?: number;
  validation_loss?: number;
  training_time?: number;
  inference_latency?: number;
}

export interface ModelParameters {
  batch_size?: number;
  learning_rate?: number;
  epochs?: number;
  optimizer?: string;
  architecture?: string;
  input_shape?: number[];
  output_shape?: number[];
  layers?: LayerConfig[];
}

export interface LayerConfig {
  type: string;
  units?: number;
  activation?: string;
  dropout?: number;
  filters?: number;
  kernel_size?: number[];
  pool_size?: number[];
  padding?: string;
}

export interface PredictionRequest {
  model_id: string;
  inputs: any;
  parameters?: {
    max_length?: number;
    temperature?: number;
    top_p?: number;
    top_k?: number;
    num_beams?: number;
    do_sample?: boolean;
  };
}

export interface PredictionResponse {
  model_id: string;
  outputs: any;
  metrics: {
    latency_ms: number;
    input_tokens: number;
    output_tokens: number;
    total_tokens: number;
  };
}

export interface TrainingJob {
  job_id: string;
  model_id: string;
  status: 'queued' | 'running' | 'completed' | 'failed';
  progress: number;
  created_at: string;
  started_at?: string;
  completed_at?: string;
  error?: string;
  metrics?: TrainingMetrics;
}

export interface TrainingMetrics {
  current_epoch: number;
  total_epochs: number;
  current_loss: number;
  validation_loss: number;
  learning_rate: number;
  training_speed: number; // examples per second
  eta_seconds: number;
  gpu_utilization?: number;
  memory_utilization?: number;
} 