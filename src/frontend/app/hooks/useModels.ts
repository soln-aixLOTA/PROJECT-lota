import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Model, TrainingJob } from '../types/models';
import { api } from '../utils/api';

export function useModels() {
  const queryClient = useQueryClient();

  const {
    data: models,
    isLoading,
    error,
    refetch,
  } = useQuery<Model[]>({
    queryKey: ['models'],
    queryFn: () => api.get('/api/models').then((res) => res.data),
  });

  const createModel = useMutation({
    mutationFn: (modelData: Partial<Model>) =>
      api.post('/api/models', modelData).then((res) => res.data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['models'] });
    },
  });

  const updateModel = useMutation({
    mutationFn: ({ id, data }: { id: string; data: Partial<Model> }) =>
      api.put(`/api/models/${id}`, data).then((res) => res.data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['models'] });
    },
  });

  const deleteModel = useMutation({
    mutationFn: (id: string) =>
      api.delete(`/api/models/${id}`).then((res) => res.data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['models'] });
    },
  });

  const trainModel = useMutation({
    mutationFn: ({ id, config }: { id: string; config: any }) =>
      api.post(`/api/models/${id}/train`, config).then((res) => res.data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['models'] });
      queryClient.invalidateQueries({ queryKey: ['training-jobs'] });
    },
  });

  const useTrainingJobs = () => {
    return useQuery<TrainingJob[]>({
      queryKey: ['training-jobs'],
      queryFn: () => api.get('/api/training-jobs').then((res) => res.data),
      refetchInterval: 5000, // Poll every 5 seconds for updates
    });
  };

  return {
    models,
    isLoading,
    error,
    refetch,
    createModel,
    updateModel,
    deleteModel,
    trainModel,
    useTrainingJobs,
  };
} 