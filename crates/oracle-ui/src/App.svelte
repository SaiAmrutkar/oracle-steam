<script>
  import { onMount } from 'svelte';
  
  let overlayVisible = false;
  let fps = 60;
  
  onMount(() => {
    console.log('[Oracle Steam UI] Svelte app mounted');
    
    // Start FPS counter
    let frameCount = 0;
    let lastTime = performance.now();
    
    function updateFPS() {
      frameCount++;
      const currentTime = performance.now();
      const elapsed = currentTime - lastTime;
      
      if (elapsed >= 1000) {
        fps = Math.round((frameCount * 1000) / elapsed);
        frameCount = 0;
        lastTime = currentTime;
      }
      
      requestAnimationFrame(updateFPS);
    }
    
    updateFPS();
    
    // Listen for overlay toggle
    document.addEventListener('keydown', (e) => {
      if (e.shiftKey && e.key === 'Tab') {
        e.preventDefault();
        overlayVisible = !overlayVisible;
      }
    });
  });
</script>

<div class="oracle-overlay" class:active={overlayVisible}>
  <div class="fps-counter">
    {fps} FPS
  </div>
  
  {#if overlayVisible}
    <div class="overlay-content">
      <h1>Oracle Steam Overlay</h1>
      <p>Press Shift+Tab to toggle</p>
    </div>
  {/if}
</div>

<style>
  .oracle-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    pointer-events: none;
    z-index: 999999;
  }
  
  .oracle-overlay.active {
    pointer-events: all;
  }
  
  .fps-counter {
    position: absolute;
    top: 10px;
    left: 10px;
    background: rgba(0, 0, 0, 0.8);
    color: #90ba3c;
    padding: 6px 12px;
    border-radius: 4px;
    font-family: monospace;
    font-weight: bold;
    pointer-events: all;
  }
  
  .overlay-content {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    background: rgba(27, 40, 56, 0.95);
    padding: 40px;
    border-radius: 8px;
    color: white;
    text-align: center;
  }
</style>