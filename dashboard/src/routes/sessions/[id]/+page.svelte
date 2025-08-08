<script lang="ts">
  import { page } from '$app/stores';
  import { onMount, onDestroy } from 'svelte';
  import { createQuery } from '@tanstack/svelte-query';
  import ReasoningStep from '$lib/components/ReasoningStep.svelte';
  import SynthesisEvolution from '$lib/components/SynthesisEvolution.svelte';
  import BiasDetections from '$lib/components/BiasDetections.svelte';
  import SessionHeader from '$lib/components/SessionHeader.svelte';
  import { fetchSession, subscribeToSession } from '$lib/api';
  import type { Session, ReasoningStep as Step, SynthesisState } from '$lib/types';

  let sessionId = $page.params.id;
  let ws: WebSocket | null = null;
  let liveSteps: Step[] = [];
  let liveSynthesis: SynthesisState[] = [];

  // Query for initial session data
  const sessionQuery = createQuery({
    queryKey: ['session', sessionId],
    queryFn: () => fetchSession(sessionId),
  });

  // Set up WebSocket for live updates
  onMount(() => {
    ws = subscribeToSession(sessionId, {
      onStep: (step) => {
        liveSteps = [...liveSteps, step];
      },
      onSynthesis: (synthesis) => {
        liveSynthesis = [...liveSynthesis, synthesis];
      },
      onComplete: () => {
        // Refetch to get final state
        $sessionQuery.refetch();
      }
    });
  });

  onDestroy(() => {
    ws?.close();
  });

  $: steps = [...($sessionQuery.data?.steps || []), ...liveSteps];
  $: synthesis = [...($sessionQuery.data?.synthesis || []), ...liveSynthesis];
</script>

<div class="min-h-screen bg-gray-50 dark:bg-gray-900">
  {#if $sessionQuery.isLoading}
    <div class="flex items-center justify-center h-screen">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
    </div>
  {:else if $sessionQuery.error}
    <div class="container mx-auto p-6">
      <div class="bg-red-50 border border-red-200 rounded-lg p-4">
        <p class="text-red-800">Error loading session: {$sessionQuery.error.message}</p>
      </div>
    </div>
  {:else if $sessionQuery.data}
    <div class="container mx-auto p-6 max-w-7xl">
      <!-- Session Header -->
      <SessionHeader session={$sessionQuery.data.session} />

      <!-- Main Content -->
      <div class="grid grid-cols-1 lg:grid-cols-3 gap-6 mt-6">
        <!-- Left Column: Reasoning Steps -->
        <div class="lg:col-span-2 space-y-4">
          <h2 class="text-xl font-semibold mb-4">Reasoning Process</h2>
          
          {#each steps as step, i}
            <ReasoningStep {step} index={i} />
          {/each}

          {#if $sessionQuery.data.session.status === 'active'}
            <div class="text-center py-4">
              <div class="inline-flex items-center space-x-2">
                <div class="animate-pulse w-2 h-2 bg-blue-600 rounded-full"></div>
                <span class="text-sm text-gray-600 dark:text-gray-400">
                  Reasoning in progress...
                </span>
              </div>
            </div>
          {/if}
        </div>

        <!-- Right Column: Synthesis & Analytics -->
        <div class="space-y-6">
          <!-- Synthesis Evolution -->
          <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-4">
            <h3 class="font-semibold mb-3">Synthesis Evolution</h3>
            <SynthesisEvolution states={synthesis} />
          </div>

          <!-- Bias Detections -->
          <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-4">
            <h3 class="font-semibold mb-3">Bias Analysis</h3>
            <BiasDetections detections={$sessionQuery.data.biasDetections} />
          </div>

          <!-- Session Metrics -->
          <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-4">
            <h3 class="font-semibold mb-3">Session Metrics</h3>
            <dl class="space-y-2">
              <div class="flex justify-between">
                <dt class="text-sm text-gray-600 dark:text-gray-400">Total Steps</dt>
                <dd class="font-medium">{steps.length}</dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-sm text-gray-600 dark:text-gray-400">Avg Confidence</dt>
                <dd class="font-medium">
                  {Math.round(
                    steps.reduce((acc, s) => acc + (s.confidence_score || 0), 0) / 
                    steps.length * 100
                  )}%
                </dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-sm text-gray-600 dark:text-gray-400">Biases Detected</dt>
                <dd class="font-medium">
                  {$sessionQuery.data.biasDetections.filter(b => b.has_bias).length}
                </dd>
              </div>
            </dl>
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>