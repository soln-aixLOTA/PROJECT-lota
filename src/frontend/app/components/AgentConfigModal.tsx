'use client';

import { useAgentStore } from '@/lib/stores/agentStore';
import { zodResolver } from '@hookform/resolvers/zod';
import * as Dialog from '@radix-ui/react-dialog';
import { useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { z } from 'zod';

const configSchema = z.object({
  name: z.string().min(1, 'Name is required'),
  config: z.record(z.any()).optional(),
});

type ConfigFormData = z.infer<typeof configSchema>;

export default function AgentConfigModal() {
  const { isModalOpen, setModalOpen, selectedAgent } = useAgentStore();
  
  const form = useForm<ConfigFormData>({
    resolver: zodResolver(configSchema),
    defaultValues: {
      name: '',
      config: {},
    },
  });

  useEffect(() => {
    if (selectedAgent) {
      form.reset({
        name: selectedAgent.name,
        config: selectedAgent.config,
      });
    }
  }, [selectedAgent, form]);

  const onSubmit = async (data: ConfigFormData) => {
    try {
      const response = await fetch('/api/agents/config', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(data),
      });

      if (!response.ok) {
        throw new Error('Failed to update agent configuration');
      }

      setModalOpen(false);
    } catch (error) {
      console.error('Error updating agent config:', error);
    }
  };

  return (
    <Dialog.Root open={isModalOpen} onOpenChange={setModalOpen}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/50 backdrop-blur-sm" />
        <Dialog.Content className="fixed left-[50%] top-[50%] translate-x-[-50%] translate-y-[-50%] w-full max-w-[425px] bg-background rounded-lg p-6 shadow-lg">
          <Dialog.Title className="text-xl font-semibold mb-4">
            Agent Configuration
          </Dialog.Title>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <div>
              <label
                htmlFor="name"
                className="block text-sm font-medium text-foreground"
              >
                Name
              </label>
              <input
                {...form.register('name')}
                className="mt-1 block w-full rounded-md border border-input bg-background px-3 py-2"
              />
              {form.formState.errors.name && (
                <p className="mt-1 text-sm text-red-500">
                  {form.formState.errors.name.message}
                </p>
              )}
            </div>
            <div className="flex justify-end space-x-2">
              <button
                type="button"
                onClick={() => setModalOpen(false)}
                className="px-4 py-2 border rounded-md hover:bg-secondary"
              >
                Cancel
              </button>
              <button
                type="submit"
                className="px-4 py-2 bg-accent text-accent-foreground rounded-md hover:bg-accent/90"
              >
                Save
              </button>
            </div>
          </form>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
} 