<script lang="ts">
import Agents from "$lib/components/Agents.svelte"
import Bar from "$lib/components/Bar.svelte"
import Glitch from "$lib/components/Glitch.svelte"
import { fade } from "svelte/transition"

let isPlaying: boolean = $state(false)
let isLoading: boolean = $state(false)
let video: HTMLVideoElement | null = $state(null)
let overlay: boolean = $state(true)

function handleTouch() {
  if (!isPlaying) {
    startVideo()
  }
}

async function startVideo() {
  if (video) {
    isLoading = true
    // Reset video state
    video.muted = true
    video.currentTime = 0
    video.playsInline = true
    video.load()

    try {
      await video.play()
      setTimeout(() => {
        if (video) {
          video.muted = false
          isPlaying = true
          overlay = false
          isLoading = false
        }
      }, 100)
    } catch (error) {
      console.error("Error playing video:", error)
      isPlaying = false
      overlay = true
      isLoading = false
    }
  }
}
</script>

<div
  role="presentation"
  ontouchstart={handleTouch}
  class="w-full h-full"
>
  <video
    bind:this={video}
    id="glitch-video"
    loop
    playsinline
    muted
    preload="auto"
    class="fixed inset-0 w-full h-full object-cover -z-10"
    data-video="glitch"
  >
    <source
      src="https://pub-32dd1494f0fa423cb1013941269ecce9.r2.dev/zkgm-v1.mp4"
      type="video/mp4"
    />
  </video>

  {#if overlay}
    <div class="fixed inset-0 bg-black flex items-center justify-center">
      <button
        class="text-union-accent-500 relative"
        onclick={startVideo}
        aria-label="Play video"
        disabled={isLoading}
      >
        {#if isLoading}
          <svg
            class="animate-spin size-24"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
          >
            <circle
              class="opacity-25"
              cx="12"
              cy="12"
              r="10"
              stroke="currentColor"
              stroke-width="4"
            />
            <path
              class="opacity-75"
              fill="currentColor"
              d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
            />
          </svg>
        {:else}
          <svg
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke-width="1.5"
            stroke="currentColor"
            class="size-24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z"
            />
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M15.91 11.672a.375.375 0 0 1 0 .656l-5.603 3.113a.375.375 0 0 1-.557-.328V8.887c0-.286.307-.466.557-.327l5.603 3.112Z"
            />
          </svg>
        {/if}
      </button>
    </div>
  {/if}

  {#if isPlaying}
    <div
      class="h-svh w-full flex flex-col justify-between items-center relative"
      in:fade
    >
      <Bar />
      <div class="flex-grow flex items-center">
        <Glitch text="ZKGM" />
      </div>
      <div class="h-24 w-full flex-shrink-0">
        <Agents />
      </div>
    </div>
  {/if}
</div>
