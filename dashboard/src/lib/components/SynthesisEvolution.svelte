<script lang="ts">
  import { fade, slide } from 'svelte/transition';
  import { ChevronRight, TrendingUp, Lightbulb, Target } from 'lucide-svelte';
  import type { SynthesisState } from '$lib/types';

  export let states: SynthesisState[] = [];

  let selectedVersion: number | null = null;
  
  $: currentState = states[states.length - 1];
  $: selectedState = selectedVersion !== null 
    ? states.find(s => s.version === selectedVersion) 
    : currentState;
</script>

<div class="space-y-4">
  <!-- Version Timeline -->
  <div class="flex items-center space-x-1 overflow-x-auto pb-2">
    {#each states as state, i}
      <button
        class="flex-shrink-0 w-10 h-10 rounded-full flex items-center justify-center text-xs font-medium transition-all
          {selectedVersion === state.version || (!selectedVersion && i === states.length - 1)
            ? 'bg-blue-600 text-white' 
            : 'bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600'}"
        on:click={() => selectedVersion = state.version}
      >
        v{state.version}
      </button>
      {#if i < states.length - 1}
        <ChevronRight class="w-4 h-4 text-gray-400" />
      {/if}
    {/each}
  </div>

  {#if selectedState}
    <div transition:fade={{ duration: 200 }}>
      <!-- Confidence & Clarity Meters -->
      <div class="grid grid-cols-2 gap-3 mb-4">
        <div>
          <div class="flex justify-between text-sm mb-1">
            <span class="text-gray-600 dark:text-gray-400">Confidence</span>
            <span class="font-medium">{Math.round(selectedState.confidence_score * 100)}%</span>
          </div>
          <div class="w-full bg-gray-200 rounded-full h-2">
            <div 
              class="h-2 rounded-full transition-all duration-500
                {selectedState.confidence_score >= 0.7 ? 'bg-green-500' : 
                 selectedState.confidence_score >= 0.4 ? 'bg-yellow-500' : 'bg-red-500'}"
              style="width: {selectedState.confidence_score * 100}%"
            ></div>
          </div>
        </div>
        
        <div>
          <div class="flex justify-between text-sm mb-1">
            <span class="text-gray-600 dark:text-gray-400">Clarity</span>
            <span class="font-medium">{Math.round(selectedState.clarity_score * 100)}%</span>
          </div>
          <div class="w-full bg-gray-200 rounded-full h-2">
            <div 
              class="h-2 rounded-full transition-all duration-500
                {selectedState.clarity_score >= 0.7 ? 'bg-blue-500' : 
                 selectedState.clarity_score >= 0.4 ? 'bg-indigo-500' : 'bg-purple-500'}"
              style="width: {selectedState.clarity_score * 100}%"
            ></div>
          </div>
        </div>
      </div>

      <!-- Current Understanding -->
      {#if selectedState.current_understanding}
        <div class="mb-4">
          <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-1 flex items-center">
            <TrendingUp class="w-4 h-4 mr-1" />
            Understanding
          </h4>
          <p class="text-sm text-gray-600 dark:text-gray-400 leading-relaxed">
            {selectedState.current_understanding}
          </p>
        </div>
      {/if}

      <!-- Key Insights -->
      {#if selectedState.insights?.length > 0}
        <div class="mb-4">
          <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2 flex items-center">
            <Lightbulb class="w-4 h-4 mr-1" />
            Key Insights ({selectedState.insights.length})
          </h4>
          <ul class="space-y-1">
            {#each selectedState.insights as insight}
              <li class="text-sm text-gray-600 dark:text-gray-400 flex items-start">
                <span class="inline-block w-1.5 h-1.5 rounded-full bg-blue-500 mt-1.5 mr-2 flex-shrink-0"></span>
                <span>{insight.insight_text}</span>
                {#if insight.confidence}
                  <span class="ml-auto text-xs text-gray-500">
                    {Math.round(insight.confidence * 100)}%
                  </span>
                {/if}
              </li>
            {/each}
          </ul>
        </div>
      {/if}

      <!-- Action Items -->
      {#if selectedState.actionItems?.length > 0}
        <div>
          <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2 flex items-center">
            <Target class="w-4 h-4 mr-1" />
            Recommended Actions
          </h4>
          <ul class="space-y-2">
            {#each selectedState.actionItems as action}
              <li class="text-sm">
                <div class="flex items-start">
                  <span class="inline-block w-1.5 h-1.5 rounded-full mt-1.5 mr-2 flex-shrink-0
                    {action.priority === 'high' ? 'bg-red-500' : 
                     action.priority === 'medium' ? 'bg-yellow-500' : 'bg-green-500'}">
                  </span>
                  <div class="flex-1">
                    <span class="text-gray-700 dark:text-gray-300">{action.action_text}</span>
                    {#if action.rationale}
                      <p class="text-xs text-gray-500 mt-0.5">{action.rationale}</p>
                    {/if}
                  </div>
                </div>
              </li>
            {/each}
          </ul>
        </div>
      {/if}

      <!-- Version Diff (if not latest) -->
      {#if selectedVersion !== null && selectedVersion < states[states.length - 1].version}
        <div class="mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
          <p class="text-xs text-gray-500 text-center">
            Viewing version {selectedVersion} of {states[states.length - 1].version}
          </p>
        </div>
      {/if}
    </div>
  {:else}
    <p class="text-sm text-gray-500 text-center py-4">
      No synthesis data available yet
    </p>
  {/if}
</div>